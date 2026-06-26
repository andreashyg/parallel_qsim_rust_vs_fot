use nohash_hasher::IntMap;
use polars::df;
use polars::prelude::*;
use rust_qsim::simulation::events::{
    EventHandlerRegisterFn, EventsManager, LinkEnterEvent, VehicleEntersTrafficEvent,
    VehicleLeavesTrafficEvent,
};
use rust_qsim::simulation::id::Id;
use rust_qsim::simulation::scenario::network::Link;
use rust_qsim::simulation::scenario::vehicles::InternalVehicle;
use rust_qsim::simulation::time::SimTime;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;
use tracing::info;

/// The status of a vehicle that has been encountered in the events, either because it entered
/// traffic, because it entered a link that indicates a certain path or because it left traffic.
/// This is used and updated by the `TravelTimePerPathCSVWriter` event handler when processing
/// events, with the first two variants as a sort of cache, and the last state later being used
/// to extract the travel times.
enum VehicleStatus {
    HasDeparted(SimTime),                 // departure time
    IsOnPath(usize, SimTime),             // path index, departure time
    HasArrived(usize, SimTime, Duration), // path index, departure time, travel time
}

/// An events handler that writes travel times, grouped by path taken, and averaged across all
/// vehicles with the same departure time,into a csv file.
/// Expected use case is scenarios where paths can be uniquely determined by a single link and every
/// vehicle is only used once.
pub struct TravelTimePerPathCSVWriter {
    /// data about departure time, travel time and path are stored here once found in the events
    vehicle_data_cache: IntMap<Id<InternalVehicle>, VehicleStatus>,
    /// vector of links, which the link at index `i` being interpreted as an indicator for path `i`
    link_to_path_lookup: IntMap<Id<Link>, usize>,
    /// path to the csv file that is to be written
    output_csv_path: PathBuf,
}

impl TravelTimePerPathCSVWriter {
    pub fn new(link_to_path_map: IntMap<Id<Link>, usize>, csv_path: impl AsRef<Path>) -> Self {
        Self {
            vehicle_data_cache: IntMap::default(),
            link_to_path_lookup: link_to_path_map,
            output_csv_path: csv_path.as_ref().to_owned(),
        }
    }

    /// when processing vehicle enters traffic event, store that the vehicle has departed
    pub fn on_vet(&mut self, e: &VehicleEntersTrafficEvent) {
        match self.vehicle_data_cache.entry(e.vehicle.clone()) {
            // if the vehicle is already in the map, something is wrong, panic correspondingly.
            Entry::Occupied(data) => match data.get() {
                VehicleStatus::HasDeparted(dep_time) => {
                    panic!(
                        "Vehicle {} entered traffic again after already doing so at time {}",
                        e.vehicle, dep_time
                    )
                }
                VehicleStatus::IsOnPath(path, dep_time) => {
                    panic!(
                        "Vehicle {} entered traffic again while already on path {}, having entered \
                        traffic at time {}",
                        e.vehicle, path, dep_time
                    )
                }
                VehicleStatus::HasArrived(_path, dep_time, travel_time) => {
                    panic!(
                        "Vehicle {} entered traffic again after leaving at time {}, and having \
                        arrived after a duration of {:?}. Panicking, since otherwise the first \
                        trip would not be recorded.",
                        e.vehicle, dep_time, travel_time
                    )
                }
            },
            // if the vehicle is not yet in the map, all good
            Entry::Vacant(_data) => {}
        }
        self.vehicle_data_cache
            .insert(e.vehicle.clone(), VehicleStatus::HasDeparted(e.time));
    }

    /// when processing vehicle leaves traffic events, verify that the vehicle has entered traffic
    /// and has a path assigned, and then store that it has arrived, with the travel time
    pub fn on_vlt(&mut self, e: &VehicleLeavesTrafficEvent) {
        match self.vehicle_data_cache.entry(e.vehicle.clone()) {
            Entry::Occupied(mut entry) => match entry.get() {
                VehicleStatus::HasDeparted(dep_time) => {
                    panic!(
                        "Vehicle {} entered traffic at time {}, but left traffic before its path was determined",
                        e.vehicle, dep_time
                    )
                }
                // expected current status: vehicle is on some path
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

    /// when processing link enter events, check if the link is an indicator for a path, if yes,
    /// update the vehicle status to being on that path.
    pub fn on_entered_link(&mut self, e: &LinkEnterEvent) {
        // check if the entered link is one that is mapped to a path index
        // (e.g. center, top, bottom in Braess)
        match self.link_to_path_lookup.get(&e.link.clone()) {
            Some(index) => match self.vehicle_data_cache.entry(e.vehicle.clone()) {
                Entry::Occupied(mut veh_entry) => match veh_entry.get() {
                    // update the vehicle status to IsOnPath with the path index
                    VehicleStatus::HasDeparted(dep_time) => {
                        veh_entry.insert(VehicleStatus::IsOnPath(*index, dep_time.clone()));
                    }
                    VehicleStatus::IsOnPath(path, _dep_time) => {
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
        let successful_vehicle_data = self
            .vehicle_data_cache
            .iter()
            .filter(|(vehicle, status)| match status {
                VehicleStatus::HasDeparted(time) => {
                    info!(
                        "Vehicle {} departed at time {} but did not arrive at any named path. \
                        Ignored in travel time extraction.",
                        vehicle, time
                    );
                    false
                }
                VehicleStatus::IsOnPath(path, time) => {
                    info!(
                        "Vehicle {} departed at time {} and entered path {} but did not arrive. \
                        Ignored in travel time extraction.",
                        vehicle, time, path
                    );
                    false
                }
                VehicleStatus::HasArrived(_, _, _) => true,
            })
            .collect::<IntMap<&Id<InternalVehicle>, &VehicleStatus>>();

        let veh_df: DataFrame = df!(
            "vehicle_id" => successful_vehicle_data
                .keys()
                .map(|id| id.to_string())
                .collect::<Vec<_>>(),
            "departure_time" => successful_vehicle_data
                .values()
                .map(|status| match status {
                    VehicleStatus::HasArrived(_path, departure_time, _travel_time) => {
                        departure_time.as_duration().as_secs_f64()
                    },
                    _ => panic!("Unexpected vehicle status, only arrived vehicles expected"),
                })
                .collect::<Vec<_>>(),
            "travel_time" => successful_vehicle_data
                .values()
                .map(|status| match status {
                    VehicleStatus::HasArrived(_path, _departure_time, travel_time) => {
                        travel_time.as_secs_f64()
                    },
                    _ => panic!("Unexpected vehicle status, only arrived vehicles expected"),
                })
                .collect::<Vec<_>>(),
            "path_index" => successful_vehicle_data
                .values()
                .map(|status| match status {
                    VehicleStatus::HasArrived(path, _departure_time, _travel_time) => {
                        *path as u32
                    },
                    _ => panic!("Unexpected vehicle status, only arrived vehicles expected"),
                })
                .collect::<Vec<_>>(),
        )
        .expect("Failed to create vehicle DataFrame");

        let unique_path_indices = {
            // get all path indices
            let mut path_indices: Vec<_> = self.link_to_path_lookup.values().collect();
            // sort and then remove duplicates
            path_indices.sort_unstable();
            path_indices.dedup();
            path_indices
        };

        // define how to aggregate the data for all vehicles with the same departure time
        let aggs = unique_path_indices // go through all path indices
            .into_iter()
            .map(|path_index| {
                // for each path index, aggregate mean travel times
                col("travel_time")
                    .filter(col("path_index").eq(lit(*path_index as u32)))
                    .mean()
                    .alias(format!("avg_travel_time_path_{}", path_index))
            })
            // then also aggregate the mean travel time for all vehicles, regardless of path
            .chain(std::iter::once(
                col("travel_time").mean().alias("avg_travel_time"),
            ))
            .collect::<Vec<_>>();

        let result = veh_df
            .clone()
            .lazy()
            .group_by([col("departure_time")])
            .agg(aggs)
            .sort(["departure_time"], SortMultipleOptions::default());

        let mut file = File::create(&self.output_csv_path).expect("Failed to create csv file");
        CsvWriter::new(&mut file)
            .finish(&mut result.collect().expect("Failed to collect result"))
            .expect("Failed to write csv file");
    }

    pub fn register_fn(
        link_to_path_map: IntMap<Id<Link>, usize>,
        csv_path: impl AsRef<Path> + Send + 'static,
    ) -> Box<EventHandlerRegisterFn> {
        // register the function to extract event times from the simulation
        Box::new(move |events_mgr: &mut EventsManager| {
            let event_time_extractor = Rc::new(RefCell::new(TravelTimePerPathCSVWriter::new(
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
    use crate::event_extraction::TravelTimePerPathCSVWriter;
    use nohash_hasher::IntMap;
    use rust_qsim::simulation::events::EventsManager;
    use rust_qsim::simulation::events::utils::read_events;
    use rust_qsim::simulation::id::Id;
    use std::fs::create_dir_all;
    use std::path::PathBuf;

    #[test]
    fn test_event_extractor() {
        let input_path = PathBuf::from(
            "./../runs_tmp/260605-cmp_braess_to_java/reroute_proba_10_until_08it_logitmu_1_proba09msa_from_08it/beta1/random1/output/events/events.0.xml.gz",
        );

        let output_path = PathBuf::from("./test_output/io/event_time_extraction")
            .join("test_event_extractor.csv");
        create_dir_all(output_path.parent().unwrap()).expect("Failed to create output directory");
        let mut event_mgr = EventsManager::new();
        let register_fn = TravelTimePerPathCSVWriter::register_fn(
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
