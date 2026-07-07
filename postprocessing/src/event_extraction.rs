#![allow(ambiguous_glob_imports)]
use nohash_hasher::IntMap;
use polars::df;
use polars::prelude::*;
// this function is technically automatically imported in the prelude above, but it is ambiguous,
// which throws warnings or errors depending on the rust version.
// Unclear if it is even fixed in the newest polars version.as
// So we import the one we want explicitly here, with a custom name.
use polars_lazy::prelude::sum_horizontal as polars_lazy_sum_horizontal;
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
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{File, create_dir_all};
use std::ops::Div;
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

#[derive(Debug)]
pub struct UnimplementedNamedMapError(String);

impl Display for UnimplementedNamedMapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for UnimplementedNamedMapError {}

pub struct LinkToPathMap(IntMap<Id<Link>, usize>);

impl LinkToPathMap {
    pub fn named(name: &str) -> Result<Self, UnimplementedNamedMapError> {
        match name.to_lowercase().as_str() {
            "braess" => Ok(LinkToPathMap(IntMap::from_iter([
                (Id::create("3_5"), 0),
                (Id::create("3_4"), 1),
                (Id::create("2_4"), 2),
            ]))),
            _ => Err(UnimplementedNamedMapError(format!(
                "No link to path map with name {} implemented",
                name,
            ))),
        }
    }
}

/// An events handler that writes travel times, grouped by path taken, and averaged across all
/// vehicles with the same departure time,into a csv file.
/// Expected use case is scenarios where paths can be uniquely determined by a single link and every
/// vehicle is only used once.
pub struct TravelTimeAndSumDepPerPathCSVWriter {
    /// data about departure time, travel time and path are stored here once found in the events
    vehicle_data_cache: IntMap<Id<InternalVehicle>, VehicleStatus>,
    /// map from link ids to an integer representing a path
    link_to_path_lookup: LinkToPathMap, // = IntMap<Id<Link>, usize>
    /// path to the csv file into which travel times are written
    tt_output_csv_path: PathBuf,
    /// path to the csv file into which summed departures are written
    sd_output_csv_path: PathBuf,
    /// parameter `beta` that was used when simulating the scenario; this is used to scale the
    /// summed departures value (we want it to be given in PCU's/PCE's)
    beta: usize,
}

impl TravelTimeAndSumDepPerPathCSVWriter {
    pub fn new(
        link_to_path_map: LinkToPathMap,
        tt_csv_path: impl AsRef<Path>,
        sd_csv_path: impl AsRef<Path>,
        beta: usize,
    ) -> Self {
        Self {
            vehicle_data_cache: IntMap::default(),
            link_to_path_lookup: link_to_path_map,
            tt_output_csv_path: tt_csv_path.as_ref().to_owned(),
            sd_output_csv_path: sd_csv_path.as_ref().to_owned(),
            beta,
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
        match self.link_to_path_lookup.0.get(&e.link.clone()) {
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
            let mut path_indices: Vec<_> = self.link_to_path_lookup.0.values().copied().collect();
            // sort and then remove duplicates
            path_indices.sort_unstable();
            path_indices.dedup();
            path_indices
        };

        // define how to aggregate the data for all vehicles with the same departure time
        let aggs = unique_path_indices // go through all path indices
            .iter()
            .copied()
            .map(|path_index| {
                // for each path index, aggregate mean travel times
                col("travel_time")
                    .filter(col("path_index").eq(lit(path_index as u32)))
                    .mean()
                    .alias(format!("avg_travel_time_path_{}", path_index))
            })
            // then also aggregate the mean travel time for all vehicles, regardless of path
            .chain(std::iter::once(
                col("travel_time").mean().alias("avg_travel_time"),
            ))
            .collect::<Vec<_>>();

        let tt_df = veh_df
            .clone()
            .lazy()
            .group_by([col("departure_time")])
            .agg(aggs)
            .sort(["departure_time"], SortMultipleOptions::default());

        let sd_df = veh_df
            .clone()
            .lazy()
            // call departure time "time" here, since what we want to report is the amount of
            // vehicles that have departed up to that time
            .group_by([col("departure_time").alias("time")])
            .agg(
                unique_path_indices
                    .iter()
                    .copied()
                    .map(|path_index| {
                        col("path_index")
                            .filter(col("path_index").eq(lit(path_index as u32)))
                            .count()
                            .alias(format!("sum_departures_path_{}", path_index))
                    })
                    .collect::<Vec<_>>(),
            )
            .sort(["departure_time"], SortMultipleOptions::default())
            .with_columns(
                unique_path_indices
                    .iter()
                    .copied()
                    .map(|path_index| {
                        col(format!("sum_departures_path_{}", path_index))
                            .cum_sum(false)
                            // divide entire column by beta^2 to get the sum of departures in PCU's/PCE's
                            .div(lit(self.beta.pow(2) as f64))
                            .alias(format!("sum_departures_path_{}", path_index))
                    })
                    .collect::<Vec<_>>(),
            )
            .with_columns([polars_lazy_sum_horizontal(
                unique_path_indices
                    .iter()
                    .copied()
                    .map(|path_index| col(format!("sum_departures_path_{}", path_index)))
                    .collect::<Vec<_>>(),
                true,
            )
            .unwrap()
            .alias("sum_departures_total")]);

        create_dir_all(
            self.tt_output_csv_path
                .parent()
                .expect("Failed to get parent directory of output csv path"),
        )
        .expect("Failed to create output directory");
        let mut file = File::create(&self.tt_output_csv_path).expect("Failed to create csv file");
        CsvWriter::new(&mut file)
            .finish(&mut tt_df.collect().expect("Failed to collect result"))
            .expect("Failed to write csv file");

        create_dir_all(
            self.sd_output_csv_path
                .parent()
                .expect("Failed to get parent directory of output csv path"),
        )
        .expect("Failed to create output directory");
        let mut file = File::create(&self.sd_output_csv_path).expect("Failed to create csv file");
        CsvWriter::new(&mut file)
            .finish(
                &mut sd_df
                    .sort(["departure_time"], SortMultipleOptions::default())
                    .collect()
                    .expect("Failed to collect result"),
            )
            .expect("Failed to write csv file");
    }

    pub fn register_fn(
        link_to_path_map: LinkToPathMap,
        tt_csv_path: impl AsRef<Path> + Send + 'static,
        sd_csv_path: impl AsRef<Path> + Send + 'static,
        beta: usize,
    ) -> Box<EventHandlerRegisterFn> {
        // register the function to extract event times from the simulation
        Box::new(move |events_mgr: &mut EventsManager| {
            let event_time_extractor =
                Rc::new(RefCell::new(TravelTimeAndSumDepPerPathCSVWriter::new(
                    link_to_path_map,
                    tt_csv_path,
                    sd_csv_path,
                    beta,
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

#[cfg(test)]
mod test {
    use crate::event_extraction::{LinkToPathMap, TravelTimeAndSumDepPerPathCSVWriter};
    use polars::prelude::*;
    use rust_qsim::simulation::events::EventsManager;
    use rust_qsim::simulation::events::utils::read_events;
    use std::fs::create_dir_all;
    use std::path::PathBuf;

    /// test the travel time per path extractor.
    /// Reads a simplified/shortened events file based on a run on the braess network, and verifies
    /// that the (avg) travel times (per path) extracted match the expected results.
    #[test]
    fn test_tt_per_path_extractor() {
        // this is an "extract" (shortened version) of an events output file from a run on the
        // braess network, with all events not related to vehicles 0, 1, 2, 3, 16, 28 removed,
        // and also for those vehicles, only events relevant for the traveltime extraction left.
        // This makes it possible to verify paths and travel times by hand
        let input_path =
            PathBuf::from("./../postprocessing/src/tests/resources/simplified_braess_events.xml");

        // the travel time extractor always writes to csv, so we have to test by writing to csv as
        // well
        let tt_output_path = PathBuf::from("./test_output/io/event_time_extraction")
            .join("test_event_extractor_travel_times.csv");
        let sd_output_path = PathBuf::from("./test_output/io/event_time_extraction")
            .join("test_event_extractor_sum_departures.csv");
        create_dir_all(tt_output_path.parent().unwrap())
            .expect("Failed to create output directory");

        // register the travel time extractor with the events manager
        let mut event_mgr = EventsManager::new();
        let register_fn = TravelTimeAndSumDepPerPathCSVWriter::register_fn(
            LinkToPathMap::named("braess").unwrap(),
            tt_output_path.clone(),
            sd_output_path.clone(),
            1,
        );
        register_fn(&mut event_mgr);

        // read the events from the input file and process them with the events manager, which will
        // publish them to the travel time csv writer
        read_events(&mut event_mgr, &input_path).expect("Failed to read events from input file");
        // finishing will trigger the travel time csv writer to write the results to the output file
        event_mgr.finish();

        // read the tt csv file that was just written into a DataFrame, so that we can compare it to
        // the expected results
        let read_tt_csv = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(tt_output_path))
            .expect("Failed to read output csv file")
            .finish()
            .unwrap();

        // we expect:
        // this can be verified by hand by considering the departure and arrival times (i.e., enters
        // traffic and leaves traffic times) of vehicles 0, 1, 2, 3, 16 and 28, respectively in the
        // simplified_braess_events.xml file; and the path the vehicles took
        // ┌────────────────┬─────────────────────┬────────────────────┬────────────────────┬─────────────────┐
        // │ departure_time ┆ avg_travel_time_pat ┆ avg_travel_time_pa ┆ avg_travel_time_pa ┆ avg_travel_time │
        // │ ---            ┆ h_0                 ┆ th_1               ┆ th_2               ┆ ---             │
        // │ f64            ┆ ---                 ┆ ---                ┆ ---                ┆ f64             │
        // │                ┆ f64                 ┆ f64                ┆ f64                ┆                 │
        // ╞════════════════╪═════════════════════╪════════════════════╪════════════════════╪═════════════════╡
        // │ 0.0            ┆ null                ┆ 25.0               ┆ null               ┆ 25.0            │
        // │ 1.0            ┆ null                ┆ 27.0               ┆ null               ┆ 27.0            │
        // │ 2.0            ┆ null                ┆ 29.0               ┆ null               ┆ 29.0            │
        // │ 3.0            ┆ null                ┆ 31.0               ┆ null               ┆ 31.0            │
        // │ 16.0           ┆ null                ┆ null               ┆ 56.0               ┆ 56.0            │
        // │ 28.0           ┆ 67.0                ┆ null               ┆ null               ┆ 67.0            │
        // └────────────────┴─────────────────────┴────────────────────┴────────────────────┴─────────────────┘
        let expected_tt_result = df!(
            "departure_time" => &[0.0, 1.0, 2.0, 3.0, 16.0, 28.0].to_vec(),
            "avg_travel_time_path_0" => &[None, None, None, None, None, Some(67.0)].to_vec(),
            "avg_travel_time_path_1" => &[Some(25.0), Some(27.0), Some(29.0), Some(31.0), None, None].to_vec(),
            "avg_travel_time_path_2" => &[None, None, None, None, Some(56.0), None].to_vec(),
            "avg_travel_time" => &[25.0, 27.0, 29.0, 31.0, 56.0, 67.0].to_vec(),
        )
            .expect("Failed to create expected tt result DataFrame");
        assert_eq!(read_tt_csv, expected_tt_result);

        // for the summed departures same thing:
        let read_sd_csv = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(sd_output_path))
            .expect("Failed to read sd output csv file")
            .finish()
            .unwrap();

        // we expect:
        // (this can also be verified by hand, by considering the departure times of vehicles 0, 1,
        // 2, 3, 16 and 28, and the path they took)
        // ┌────────────────┬────────────────────┬────────────────────┬───────────────────┬───────────────────┐
        // │ departure_time ┆ sum_departures_pat ┆ sum_departures_pat ┆ sum_departures_pa ┆ sum_departures_to │
        // │ ---            ┆ h_0                ┆ h_1                ┆ th_2              ┆ tal               │
        // │ f64            ┆ ---                ┆ ---                ┆ ---               ┆ ---               │
        // │                ┆ f64                ┆ f64                ┆ f64               ┆ f64               │
        // ╞════════════════╪════════════════════╪════════════════════╪═══════════════════╪═══════════════════╡
        // │ 0.0            ┆ 0.0                ┆ 1.0                ┆ 0.0               ┆ 1.0               │
        // │ 1.0            ┆ 0.0                ┆ 2.0                ┆ 0.0               ┆ 2.0               │
        // │ 2.0            ┆ 0.0                ┆ 3.0                ┆ 0.0               ┆ 3.0               │
        // │ 3.0            ┆ 0.0                ┆ 4.0                ┆ 0.0               ┆ 4.0               │
        // │ 16.0           ┆ 0.0                ┆ 4.0                ┆ 1.0               ┆ 5.0               │
        // │ 28.0           ┆ 1.0                ┆ 4.0                ┆ 1.0               ┆ 6.0               │
        // └────────────────┴────────────────────┴────────────────────┴───────────────────┴───────────────────┘
        let expected_sd_result = df!(
            "departure_time" => &[0.0, 1.0, 2.0, 3.0, 16.0, 28.0].to_vec(),
            "sum_departures_path_0" => &[0,0,0,0,0,1].to_vec(),
            "sum_departures_path_1" => &[1,2,3,4,4,4].to_vec(),
            "sum_departures_path_2" => &[0,0,0,0,1,1].to_vec(),
            "sum_departures_total" => &[1,2,3,4,5,6].to_vec(),
        )
        .expect("Failed to create expected sd result DataFrame");

        assert_eq!(read_sd_csv, expected_sd_result);
    }
}
