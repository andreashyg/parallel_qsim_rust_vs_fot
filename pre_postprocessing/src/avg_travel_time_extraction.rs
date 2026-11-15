use matsim_rust::simulation::events::{
    EventHandlerRegisterFn, LinkEnterEvent, LinkLeaveEvent, VehicleEntersTrafficEvent,
    VehicleLeavesTrafficEvent,
};
use matsim_rust::simulation::id::Id;
use matsim_rust::simulation::scenario::network::Link;
use matsim_rust::simulation::scenario::vehicles::InternalVehicle;
use matsim_rust::simulation::time::SimTime;
use nohash_hasher::IntMap;
use polars::df;
use polars::prelude::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::btree_map;
use std::collections::hash_map;
use std::fs::{File, create_dir_all};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

// Average duration together with the number of measurements that were used to calculate it.
// This allows to calculate a new average when a new measurement is added, without having to store
// all measurements.
// It also allows to calculate an average of averages, by using the count to weight the averages.
#[derive(Clone)]
struct CountedAverageDuration {
    avg_dur_in_secs: f64,
    count: usize,
}

impl CountedAverageDuration {
    #[allow(dead_code)]
    fn new() -> Self {
        CountedAverageDuration {
            avg_dur_in_secs: 0.0,
            count: 0,
        }
    }

    pub fn new_from_initial_measurement(dur: Duration) -> Self {
        CountedAverageDuration {
            avg_dur_in_secs: dur.as_secs_f64(),
            count: 1,
        }
    }

    // Add a new measurement, update the average and the count.
    fn add_measurement_and_update_avg(&mut self, dur_in_secs: f64) {
        // Update the average using the formula for a running average.
        // self.avg_dur = (self.avg_dur * self.count as u32 + dur) / (self.count as u32 + 1);

        // Avoid overflow by reshuffling the formula above to:
        // new_avg = old_avg + (new_value - old_avg) / (count + 1)
        self.avg_dur_in_secs += (dur_in_secs - self.avg_dur_in_secs) / (self.count as f64 + 1.0);

        // update counter
        self.count += 1;
    }

    // Add a new measurement, update the average and the count.
    fn add_other_counted_avg_add_measurement_and_update_avg(
        &mut self,
        other: &CountedAverageDuration,
    ) {
        // Update the average using the formula for a running average.
        // self.avg_dur = (self.avg_dur * self.count as u32 + other.avg_dur * other.count) / (self.count as u32 + other.count);

        // Avoid overflow by reshuffling the formula above to:
        // new_avg = old_avg + other.count * (other.avg_dur - old_avg) / (self.count + other.count)
        self.avg_dur_in_secs += other.count as f64 * (other.avg_dur_in_secs - self.avg_dur_in_secs)
            / (self.count as f64 + other.count as f64);

        // self.avg_dur_in_secs = (self.avg_dur_in_secs * self.count as f64
        //     + other.avg_dur_in_secs * other.count as f64)
        //     / (self.count as f64 + other.count as f64);

        // update counter
        self.count += other.count;
    }

    // Calculate the average of averages, weighted by the count of measurements used to calculate
    // each average.
    fn get_avg_of_averages(averages: &[&Self]) -> f64 {
        if averages.is_empty() {
            panic!(
                "Cannot calculate average of averages for empty slice, which means no measurements."
            );
        }

        let total_weight = averages.iter().map(|a| a.count as f64).sum::<f64>();
        if total_weight == 0.0 {
            panic!("Got slice of averages with total weight of 0, which means no measurements.");
        }

        // averages
        //     .iter()
        //     .map(|a| a.avg_dur_in_secs * a.count as f64)
        //     .sum::<f64>()
        //     / total_weight

        // Avoid overflow by reshuffling the formula above to:
        // weighted_avg = for avg in averages {
        //  weighted_avg += (avg - weighted_avg) * (weight / total_weight)
        // }
        let weighted_avg = averages.iter().fold(0.0, |acc, a| {
            acc + (a.avg_dur_in_secs - acc) * (a.count as f64 / total_weight)
        });
        weighted_avg
    }
}
pub struct AvgTravelTimeExtractor {
    tts_over_le_time_csv_path: PathBuf,
    avg_tt_output_path: PathBuf,
    bin_size: Duration,
    // Cache to store the vehicles after they have entered, until they leave
    vehs_on_link_cache: IntMap<Id<Link>, IntMap<Id<InternalVehicle>, SimTime>>,
    // Storage of every link travel time, per link enter time.
    // This will allow to get average travel times per time, but also to get the average travel time
    // over all, weighted.
    measured_durations_over_time: BTreeMap<SimTime, CountedAverageDuration>,
}

/// Both Link enter events and VehicleEntersTraffic events are used as the "enter" event for a link,
/// (the time when we start counting travel time on the link). So we need to handle both types of
/// events in the same way.
pub enum LinkEnterLikeEvent {
    LinkEnter(LinkEnterEvent),
    VehicleEntersTraffic(VehicleEntersTrafficEvent),
}
impl LinkEnterLikeEvent {
    pub fn get_link(&self) -> &Id<Link> {
        match self {
            LinkEnterLikeEvent::LinkEnter(event) => &event.link,
            LinkEnterLikeEvent::VehicleEntersTraffic(event) => &event.link,
        }
    }

    pub fn get_vehicle(&self) -> &Id<InternalVehicle> {
        match self {
            LinkEnterLikeEvent::LinkEnter(event) => &event.vehicle,
            LinkEnterLikeEvent::VehicleEntersTraffic(event) => &event.vehicle,
        }
    }

    pub fn get_time(&self) -> SimTime {
        match self {
            LinkEnterLikeEvent::LinkEnter(event) => event.time,
            LinkEnterLikeEvent::VehicleEntersTraffic(event) => event.time,
        }
    }
}

pub enum LinkLeaveLikeEvent {
    LinkLeave(LinkLeaveEvent),
    VehicleLeavesTraffic(VehicleLeavesTrafficEvent),
}

impl LinkLeaveLikeEvent {
    pub fn get_link(&self) -> &Id<Link> {
        match self {
            LinkLeaveLikeEvent::LinkLeave(event) => &event.link,
            LinkLeaveLikeEvent::VehicleLeavesTraffic(event) => &event.link,
        }
    }

    pub fn get_vehicle(&self) -> &Id<InternalVehicle> {
        match self {
            LinkLeaveLikeEvent::LinkLeave(event) => &event.vehicle,
            LinkLeaveLikeEvent::VehicleLeavesTraffic(event) => &event.vehicle,
        }
    }

    pub fn get_time(&self) -> SimTime {
        match self {
            LinkLeaveLikeEvent::LinkLeave(event) => event.time,
            LinkLeaveLikeEvent::VehicleLeavesTraffic(event) => event.time,
        }
    }
}

impl AvgTravelTimeExtractor {
    pub fn new(
        tts_over_le_time_csv_path: impl AsRef<Path>,
        avg_tt_output_path: impl AsRef<Path>,
        bin_size: Duration,
    ) -> Self {
        AvgTravelTimeExtractor {
            tts_over_le_time_csv_path: tts_over_le_time_csv_path.as_ref().into(),
            avg_tt_output_path: avg_tt_output_path.as_ref().into(),
            bin_size,
            vehs_on_link_cache: IntMap::default(),
            measured_durations_over_time: BTreeMap::default(),
        }
    }

    fn on_link_enter_or_vehicle_enters_traffic_event(&mut self, event: &LinkEnterLikeEvent) {
        // If the cache already has data for this link, we need to check if it contains the current
        // vehicle.
        match self.vehs_on_link_cache.entry(event.get_link().clone()) {
            // Cache has data (a hashmap of vehicles) for this link.
            // Check if that hashmap contains the current vehicle.
            hash_map::Entry::Occupied(mut vehs_on_link_entry) => {
                match vehs_on_link_entry
                    .get_mut()
                    .entry(event.get_vehicle().clone())
                {
                    // Link hashmap does contain the current vehicle
                    hash_map::Entry::Occupied(_) => {
                        panic!(
                            "Vehicle {} already on link {} when entering at time {}",
                            event.get_vehicle(),
                            event.get_link(),
                            event.get_time()
                        );
                    }
                    // Link hashmap does not contain the current vehicle, insert it with its enter time.
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(event.get_time());
                    }
                }
            }
            // Cache has no data for this link yet, create it and insert the vehicle and its enter time.
            hash_map::Entry::Vacant(entry) => {
                entry.insert(IntMap::from_iter([(
                    event.get_vehicle().clone(),
                    event.get_time(),
                )]));
            }
        }
    }

    fn on_link_leave_or_vehicle_leaves_traffic_event(&mut self, event: &LinkLeaveLikeEvent) {
        match self.vehs_on_link_cache.entry(event.get_link().clone()) {
            // Cache has data (a hashmap of vehicles) for this link.
            hash_map::Entry::Occupied(mut vehs_on_link_entry) => {
                match vehs_on_link_entry
                    .get_mut()
                    .entry(event.get_vehicle().clone())
                {
                    // The link hashmap of vehicles contains the current vehicle with its enter time
                    hash_map::Entry::Occupied(vehicle_entry) => {
                        let enter_time = vehicle_entry.get();
                        let travel_time =
                            Duration::from_secs(event.get_time().as_secs() - enter_time.as_secs());
                        // Update the measured durations over time with the new travel time.
                        match self.measured_durations_over_time.entry(*enter_time) {
                            // There are already travel time measurements for this enter time.
                            // Update the average with the new measurement.
                            btree_map::Entry::Occupied(mut avg_entry) => {
                                avg_entry
                                    .get_mut()
                                    .add_measurement_and_update_avg(travel_time.as_secs_f64());
                            }
                            // There are no travel time measurements for this enter time yet.
                            // Create a new CountedAverageDuration with the current travel time as
                            // the initial measurement (and count = 1).
                            btree_map::Entry::Vacant(entry) => {
                                entry.insert(CountedAverageDuration::new_from_initial_measurement(
                                    travel_time,
                                ));
                            }
                        }
                        // Remove the vehicle from the link hashmap, as it has now left the link.
                        vehicle_entry.remove();
                    }
                    // The link hashmap of vehicles does not contain the current vehicle, which means
                    // that the vehicle is leaving a link it never entered, or that it has already left that link.
                    hash_map::Entry::Vacant(_) => {
                        panic!(
                            "Vehicle {} not found on link {} when leaving at time {}",
                            event.get_vehicle(),
                            event.get_link(),
                            event.get_time()
                        );
                    }
                }
            }
            // Cache has no data for this link, which means that the vehicle is leaving a link it
            // never entered, or that it has already left that link.
            hash_map::Entry::Vacant(_) => {
                panic!(
                    "Link {} not found in cache when vehicle {} is leaving at time {}",
                    event.get_link(),
                    event.get_vehicle(),
                    event.get_time()
                );
            }
        }
    }

    pub fn on_link_enter_event(&mut self, event: &LinkEnterEvent) {
        self.on_link_enter_or_vehicle_enters_traffic_event(&LinkEnterLikeEvent::LinkEnter(
            event.clone(),
        ));
    }

    pub fn on_vehicle_enters_traffic_event(&mut self, event: &VehicleEntersTrafficEvent) {
        self.on_link_enter_or_vehicle_enters_traffic_event(
            &LinkEnterLikeEvent::VehicleEntersTraffic(event.clone()),
        );
    }

    pub fn on_link_leave_event(&mut self, event: &LinkLeaveEvent) {
        self.on_link_leave_or_vehicle_leaves_traffic_event(&LinkLeaveLikeEvent::LinkLeave(
            event.clone(),
        ));
    }

    pub fn on_vehicle_leaves_traffic_event(&mut self, event: &VehicleLeavesTrafficEvent) {
        self.on_link_leave_or_vehicle_leaves_traffic_event(
            &LinkLeaveLikeEvent::VehicleLeavesTraffic(event.clone()),
        );
    }

    pub fn on_finish(&self) {
        // collect average travel time and count of measurements into bins
        let mut binned_measured_durations_over_time: BTreeMap<SimTime, CountedAverageDuration> =
            BTreeMap::new();
        for (enter_time, avg_duration) in &self.measured_durations_over_time {
            let bin_start_time_secs =
                (enter_time.as_duration().as_secs_f64() / self.bin_size.as_secs_f64()).floor()
                    * self.bin_size.as_secs_f64();
            let bin_start_time =
                SimTime::from_duration(Duration::from_secs_f64(bin_start_time_secs));
            match binned_measured_durations_over_time.entry(bin_start_time) {
                btree_map::Entry::Occupied(mut avg_entry) => {
                    avg_entry
                        .get_mut()
                        .add_other_counted_avg_add_measurement_and_update_avg(avg_duration);
                }
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(CountedAverageDuration {
                        avg_dur_in_secs: avg_duration.avg_dur_in_secs,
                        count: avg_duration.count,
                    });
                }
            }
        }

        let mut tt_over_time_df = df!(
            "enter_time" => binned_measured_durations_over_time.keys().map(|t| t.as_duration().as_secs_f64()).collect::<Vec<f64>>(),
            "avg_travel_time_secs" => binned_measured_durations_over_time.values().map(|avg| avg.avg_dur_in_secs).collect::<Vec<f64>>(),
            "counted_link_traversals" => binned_measured_durations_over_time.values().map(|avg| avg.count as u64).collect::<Vec<u64>>(),
        ).expect("Failed to create DataFrame from measured durations over time");

        create_dir_all(&self.tts_over_le_time_csv_path.parent().expect(
            "Failed to get parent directory for travel time over link enter time CSV path",
        ))
        .expect("Failed to create directory for travel time over link enter time CSV");

        let mut tt_over_le_time_csv_file = File::create(&self.tts_over_le_time_csv_path)
            .expect("Failed to create travel time over link enter time CSV file");

        CsvWriter::new(&mut tt_over_le_time_csv_file)
            .finish(&mut tt_over_time_df)
            .expect("Failed to write travel time over link enter time CSV file");

        // Calculate the average travel time over all measured durations, weighted by the count of
        // measurements used to calculate each average.
        // Note that this is a dataframe with only one row and one column.
        let mut avg_tt_df = df!(
            "avg_travel_time_secs" => vec![CountedAverageDuration::get_avg_of_averages(
                &binned_measured_durations_over_time.values().collect::<Vec<&CountedAverageDuration>>()
            )],
        )
        .expect("Failed to create DataFrame for average travel time over all measured durations");

        create_dir_all(
            &self
                .avg_tt_output_path
                .parent()
                .expect("Failed to get parent directory for average travel time CSV path"),
        )
        .expect("Failed to create directory for average travel time CSV");

        let mut avg_tt_csv_file = File::create(&self.avg_tt_output_path)
            .expect("Failed to create average travel time CSV file");

        CsvWriter::new(&mut avg_tt_csv_file)
            .include_header(false) // don't include header, as we only have one row and one column
            .finish(&mut avg_tt_df)
            .expect("Failed to write average travel time CSV file");
    }

    pub fn register_fn(
        tts_over_le_time_csv_path: impl AsRef<Path> + Send + 'static,
        avg_tt_output_path: impl AsRef<Path> + Send + 'static,
        bin_size: Duration,
    ) -> Box<EventHandlerRegisterFn> {
        Box::new(move |events_mgr| {
            let avg_tt_extractor = Rc::new(RefCell::new(AvgTravelTimeExtractor::new(
                tts_over_le_time_csv_path,
                avg_tt_output_path,
                bin_size,
            )));
            let avg_tt_extractor_1 = avg_tt_extractor.clone();
            let avg_tt_extractor_2 = avg_tt_extractor.clone();
            let avg_tt_extractor_3 = avg_tt_extractor.clone();
            let avg_tt_extractor_4 = avg_tt_extractor.clone();
            let avg_tt_extractor_5 = avg_tt_extractor.clone();

            events_mgr.on::<LinkEnterEvent, _>(move |event| {
                avg_tt_extractor_1.borrow_mut().on_link_enter_event(event);
            });
            events_mgr.on::<VehicleEntersTrafficEvent, _>(move |event| {
                avg_tt_extractor_2
                    .borrow_mut()
                    .on_vehicle_enters_traffic_event(event);
            });
            events_mgr.on::<LinkLeaveEvent, _>(move |event| {
                avg_tt_extractor_3.borrow_mut().on_link_leave_event(event);
            });
            events_mgr.on::<VehicleLeavesTrafficEvent, _>(move |event| {
                avg_tt_extractor_4
                    .borrow_mut()
                    .on_vehicle_leaves_traffic_event(event);
            });
            events_mgr.on_finish(move || {
                avg_tt_extractor_5.borrow().on_finish();
            });
        })
    }
}
