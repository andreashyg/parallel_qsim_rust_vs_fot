use nohash_hasher::IntMap;
use polars::df;
use polars::prelude::*;
use rust_qsim::simulation::events::{
    EventHandlerRegisterFn, EventTrait, EventsManager, LinkEnterEvent, PersonDepartureEvent,
    VehicleEntersTrafficEvent, VehicleLeavesTrafficEvent,
};
use rust_qsim::simulation::id::Id;
use rust_qsim::simulation::io::proto::proto_events::ProtoEventsWriter;
use rust_qsim::simulation::scenario::network::Link;
use rust_qsim::simulation::scenario::vehicles::InternalVehicle;
use rust_qsim::simulation::time::SimTime;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;
use tracing::{info, warn};

struct VehiclePathData {
    departure_time: SimTime,
    took_path: usize,
    travel_time: Duration,
}

enum VehicleStatus {
    HasDeparted(SimTime),                 // departure time
    IsOnPath(usize, SimTime),             // path index, departure time
    HasArrived(usize, SimTime, Duration), // path index, departure time, travel time
}

pub struct SomeEventTimeExtractor {
    vehicle_data: IntMap<Id<InternalVehicle>, VehicleStatus>,
    link_to_path_lookup: IntMap<Id<Link>, usize>,
    writer: BufWriter<File>,
    csv_path: PathBuf,
}

impl SomeEventTimeExtractor {
    pub fn new(link_to_path_map: IntMap<Id<Link>, usize>, csv_path: impl AsRef<Path>) -> Self {
        let file = File::create(csv_path.as_ref()).unwrap();
        let writer = BufWriter::new(file);
        Self {
            vehicle_data: IntMap::default(),
            link_to_path_lookup: link_to_path_map,
            writer,
            csv_path: csv_path.as_ref().to_owned(),
        }
    }

    pub fn on_vet(&mut self, e: &VehicleEntersTrafficEvent) {
        match self.vehicle_data.entry(e.vehicle.clone()) {
            Entry::Occupied(data) => match data.get() {
                VehicleStatus::HasDeparted(dep_time) => {
                    panic!(
                        "Vehicle {} entered traffic again after already doing so at time {}",
                        e.vehicle, dep_time
                    )
                }
                VehicleStatus::IsOnPath(path, dep_time) => {
                    panic!(
                        "Vehicle {} entered traffic again while already on path {}, having entered traffic at time {}",
                        e.vehicle, path, dep_time
                    )
                }
                VehicleStatus::HasArrived(_path, dep_time, travel_time) => {
                    warn!(
                        "Vehicle {} entered traffic again after leaving at time {}, and having arrived after a duration of {:?}",
                        e.vehicle, dep_time, travel_time
                    )
                }
            },
            Entry::Vacant(_data) => {}
        }
        self.vehicle_data
            .insert(e.vehicle.clone(), VehicleStatus::HasDeparted(e.time));
    }

    pub fn on_vlt(&mut self, e: &VehicleLeavesTrafficEvent) {
        match self.vehicle_data.entry(e.vehicle.clone()) {
            Entry::Occupied(mut entry) => match entry.get() {
                VehicleStatus::HasDeparted(dep_time) => {
                    panic!(
                        "Vehicle {} entered traffic at time {}, but left traffic before its path was determined",
                        e.vehicle, dep_time
                    )
                }
                VehicleStatus::IsOnPath(path, dep_time) => {
                    entry.insert(VehicleStatus::HasArrived(
                        *path,
                        dep_time.clone(),
                        e.time.saturating_sub(dep_time.as_duration()).as_duration(),
                    ));
                }
                VehicleStatus::HasArrived(_path, dep_time, travel_time) => {
                    panic!(
                        "Vehicle {} arrived again after already arriving at time {}",
                        e.vehicle,
                        dep_time.saturating_add(*travel_time)
                    );
                }
            },
            Entry::Vacant(_entry) => {
                panic!("Vehicle {} left traffic without entering first", e.vehicle)
            }
        }
    }
    pub fn on_entered_link(&mut self, e: &LinkEnterEvent) {
        // check if the entered link is one that is mapped to a path index
        // (e.g. center, top, bottom in Braess)
        match self.link_to_path_lookup.get(&e.link.clone()) {
            Some(index) => match self.vehicle_data.entry(e.vehicle.clone()) {
                Entry::Occupied(mut veh_entry) => match veh_entry.get() {
                    // update the vehicle status to IsOnPath with the path index
                    VehicleStatus::HasDeparted(dep_time) => {
                        veh_entry.insert(VehicleStatus::IsOnPath(*index, dep_time.clone()));
                    }
                    VehicleStatus::IsOnPath(path, dep_time) => {
                        // if vehicle is already on a path, check if the path index matches the one for the entered link
                        if path != index {
                            panic!(
                                "Vehicle {} entered link {} which is mapped to path {}, while already on path {}",
                                e.vehicle, e.link, index, path
                            )
                        }
                    }
                    VehicleStatus::HasArrived(_path, dep_time, travel_time) => {
                        panic!(
                            "Vehicle {} entered link {} after already arriving at time {}",
                            e.vehicle,
                            e.link,
                            dep_time.saturating_add(*travel_time)
                        )
                    }
                },
                Entry::Vacant(_veh_entry) => {
                    panic!(
                        "Vehicle {} entered link {} without entering traffic first",
                        e.vehicle, e.link
                    )
                }
            },
            None => {}
        }
    }

    pub fn on_finish(&mut self) {
        let mut successful_vehicle_data = self.vehicle_data.iter().filter(|(vehicle, status)| match status {
            VehicleStatus::HasDeparted(time) => {
                info!(
                        "Vehicle {} departed at time {} but did not arrive at any named path. Ignored in travel time extraction.",
                        vehicle, time
                    );
                false
            }
            VehicleStatus::IsOnPath(path, time) => {
                info!(
                        "Vehicle {} departed at time {} and entered path {} but did not arrive. Ignored in travel time extraction.",
                        vehicle, time, path
                    );
                false
            }
            VehicleStatus::HasArrived(_, _, _) => true,
        }).collect::<IntMap<&Id<InternalVehicle>, &VehicleStatus>>();

        let veh_df: DataFrame = df!(
            "vehicle_id" => successful_vehicle_data.keys().map(|id| id.to_string()).collect::<Vec<_>>(),
            "departure_time" => successful_vehicle_data.values().map(|status| match status {
                VehicleStatus::HasArrived(_path, departure_time, _travel_time) => departure_time.as_duration().as_secs_f64(),
                _ => panic!("Unexpected vehicle status, only successfully arrived vehicles expected"),
            }).collect::<Vec<_>>(),
            "travel_time" => successful_vehicle_data.values().map(|status| match status {
                VehicleStatus::HasArrived(_path, _departure_time, travel_time) => travel_time.as_secs_f64(),
                _ => panic!("Unexpected vehicle status, only successfully arrived vehicles expected"),
            }).collect::<Vec<_>>(),
        ).expect("Failed to create vehicle DataFrame");

        dbg!(&veh_df);

        let result = veh_df
            .clone()
            .lazy()
            .group_by([col("departure_time")])
            .agg([
                col("travel_time"),
                col("travel_time").mean().alias("avg_travel_time"),
            ]);

        dbg!(&result.collect().expect("Failed to collect result"));
        //TODO continue here: maybe repair the other things and start testing with data.
        // makes it easier to verify correctness

        //
        // //     smth write things to file or print them out
        // let mut writer =
        //     csv::Writer::from_path(&self.csv_path).expect("Failed to create CSV writer");
        // writer
        //     .write_record(&[
        //         "departure time",
        //         "avg tt top",
        //         "avg tt mid",
        //         "avg tt bot",
        //         "avg tt all",
        //     ])
        //     .expect("Failed to write CSV header");
    }

    pub fn register_fn(
        link_to_path_map: IntMap<Id<Link>, usize>,
        csv_path: impl AsRef<Path> + Send + 'static,
    ) -> Box<EventHandlerRegisterFn> {
        // register the function to extract event times from the simulation
        Box::new(move |events_mgr: &mut EventsManager| {
            let event_time_extractor = Rc::new(RefCell::new(SomeEventTimeExtractor::new(
                link_to_path_map,
                csv_path,
            )));
            let event_time_extractor_1 = event_time_extractor.clone();
            let event_time_extractor_2 = event_time_extractor.clone();
            let event_time_extractor_3 = event_time_extractor.clone();
            let event_time_extractor_4 = event_time_extractor.clone();

            events_mgr.on::<VehicleEntersTrafficEvent, _>(move |e| {
                event_time_extractor_1.borrow_mut().on_vet(e);
            });
            events_mgr.on::<VehicleLeavesTrafficEvent, _>(move |e| {
                event_time_extractor_2.borrow_mut().on_vlt(e);
            });
            events_mgr.on::<LinkEnterEvent, _>(move |e| {
                event_time_extractor_3.borrow_mut().on_entered_link(e);
            });

            events_mgr.on_finish(move || {
                event_time_extractor_4.borrow_mut().on_finish();
            });
        })
    }
}

mod test {
    use crate::event_extraction::SomeEventTimeExtractor;
    use nohash_hasher::IntMap;
    use rust_qsim::simulation::events::EventsManager;
    use rust_qsim::simulation::events::utils::read_events;
    use rust_qsim::simulation::id::Id;
    use rust_qsim::simulation::scenario::network::Link;
    use std::fs::create_dir_all;
    use std::path::PathBuf;

    #[test]
    fn test_event_extractor() {
        let input_path = PathBuf::from(
            // "./runs_tmp/260605-cmp_braess_to_java/reroute_proba_10_until_08it_logitmu_1_proba09msa_from_08it/beta1/random1/output/events/events.0.xml.gz",
            "/home/andreas/RustroverProjects/parallel_qsim_rust_vs_fot/runs_tmp/260605-cmp_braess_to_java/reroute_proba_10_until_08it_logitmu_1_proba09msa_from_08it/beta1/random1/output/events/events.0.xml.gz",
        );
        // TODO continue here: why can it not open this file? Only works with the full local path right now

        // TODO and then: why are my dataframes empty?? Is it because they are lazy? probably not. Then something is wrong.

        //TODO !!!! it seems I don't have any vehicle enters, vehicle leaves traffic, linkenter events in the given file.
        // WHY???

        let output_path = PathBuf::from("./test_output/io/event_time_extraction")
            .join("test_event_extractor.csv");
        create_dir_all(output_path.parent().unwrap()).expect("Failed to create output directory");
        let mut event_mgr = EventsManager::new();
        let register_fn = SomeEventTimeExtractor::register_fn(
            IntMap::from_iter([
                (Id::create("3_5"), 0),
                (Id::create("3_4"), 1),
                (Id::create("2_4"), 2),
            ]),
            output_path,
        );
        register_fn(&mut event_mgr);

        read_events(&mut event_mgr, &input_path).expect("Failed to read events from input file");
        event_mgr.finish();
    }
}
