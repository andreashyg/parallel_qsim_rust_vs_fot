use crate::simulation::Identifiable;
use crate::simulation::events::{
    ActivityEndEvent, ActivityStartEvent, LinkEnterEvent, PersonArrivalEvent, PersonDepartureEvent,
    PersonEntersVehicleEvent, PersonLeavesVehicleEvent, PersonStuckEvent,
    PtTeleportationArrivalEvent, TeleportationArrivalEvent, VehicleEntersTrafficEvent,
    VehicleLeavesTrafficEvent,
};
use crate::simulation::id::Id;
use crate::simulation::messaging::partition_change::PartitionChangeEntity;
use crate::simulation::scenario::population::InternalPerson;
use crate::simulation::scenario::vehicles::InternalVehicle;
use crate::simulation::scoring::backpacking::backpack::{Backpack, PersonExperience};
use crate::simulation::scoring::partial_plans::PartialPlan;
use nohash_hasher::IntMap;
use std::collections::hash_map::Entry;

pub(crate) struct BackpackingAttachment {
    backpacks: Vec<Backpack>,
}

pub struct BackpackingDataCollector {
    home_person_ids: Vec<Id<InternalPerson>>,
    person_id2backpack: IntMap<Id<InternalPerson>, Backpack>,
    vehicle_id2person_ids: IntMap<Id<InternalVehicle>, VehicleOccupants>,
}

/// The persons in a vehicle. The first one is stored inline, so that a vehicle with only a driver,
/// the common case, needs no allocation of its own. Looking up the occupants of a vehicle, which
/// happens for every link a vehicle enters, then also doesn't follow a pointer into another table.
#[derive(Debug)]
struct VehicleOccupants {
    first: Id<InternalPerson>,
    others: Vec<Id<InternalPerson>>,
}

impl VehicleOccupants {
    fn new(person: Id<InternalPerson>) -> Self {
        Self {
            first: person,
            others: Vec::new(),
        }
    }

    /// Creates the occupants from the persons of a vehicle, which always has a driver.
    fn from_persons(persons: impl IntoIterator<Item = Id<InternalPerson>>) -> Self {
        let mut persons = persons.into_iter();
        let mut occupants = Self::new(persons.next().expect("A vehicle without persons."));
        persons.for_each(|person| occupants.insert(person));
        occupants
    }

    fn iter(&self) -> impl Iterator<Item = &Id<InternalPerson>> {
        std::iter::once(&self.first).chain(&self.others)
    }

    /// Adds the person unless it is an occupant already.
    fn insert(&mut self, person: Id<InternalPerson>) {
        if !self.iter().any(|occupant| *occupant == person) {
            self.others.push(person);
        }
    }

    /// Removes the person, if it is an occupant, and returns whether the vehicle is empty then.
    fn remove(&mut self, person: &Id<InternalPerson>) -> bool {
        if self.first == *person {
            match self.others.pop() {
                Some(next) => self.first = next,
                None => return true,
            }
        } else {
            self.others.retain(|occupant| occupant != person);
        }
        false
    }
}

impl BackpackingDataCollector {
    pub fn new(home_person_ids: Vec<Id<InternalPerson>>) -> Self {
        Self {
            home_person_ids,
            person_id2backpack: Default::default(),
            vehicle_id2person_ids: Default::default(),
        }
    }

    #[hotpath::measure]
    pub(crate) fn reset_iteration(&mut self) {
        self.person_id2backpack.clear();
        for person in &self.home_person_ids {
            self.person_id2backpack
                .insert(person.clone(), Backpack::new(person.clone()));
        }
        self.vehicle_id2person_ids.clear();
    }

    /// Records the person as an occupant of the vehicle and forwards the event to its backpack.
    #[hotpath::measure]
    pub(crate) fn person_enters_vehicle(&mut self, event: &PersonEntersVehicleEvent) {
        match self.vehicle_id2person_ids.entry(event.vehicle.clone()) {
            Entry::Occupied(mut occupants) => occupants.get_mut().insert(event.person.clone()),
            Entry::Vacant(vacant) => {
                vacant.insert(VehicleOccupants::new(event.person.clone()));
            }
        }
        self.partial_plan(&event.person)
            .person_enters_vehicle(event);
    }

    /// Removes the person from the occupants of the vehicle and forwards the event to its backpack.
    #[hotpath::measure]
    pub(crate) fn person_leaves_vehicle(&mut self, event: &PersonLeavesVehicleEvent) {
        let remove_vehicle = self
            .vehicle_id2person_ids
            .get_mut(&event.vehicle)
            .map(|occupants| occupants.remove(&event.person))
            .unwrap_or(false);
        if remove_vehicle {
            self.vehicle_id2person_ids.remove(&event.vehicle);
        }
        self.partial_plan(&event.person)
            .person_leaves_vehicle(event);
    }

    pub(crate) fn person_departure(&mut self, event: &PersonDepartureEvent) {
        self.partial_plan(&event.person).person_departure(event);
    }

    pub(crate) fn person_arrival(&mut self, event: &PersonArrivalEvent) {
        self.partial_plan(&event.person).person_arrival(event);
    }

    pub(crate) fn activity_start(&mut self, event: &ActivityStartEvent) {
        self.partial_plan(&event.person).activity_start(event);
    }

    pub(crate) fn activity_end(&mut self, event: &ActivityEndEvent) {
        self.partial_plan(&event.person).activity_end(event);
    }

    pub(crate) fn teleportation_arrival(&mut self, event: &TeleportationArrivalEvent) {
        self.partial_plan(&event.person)
            .teleportation_arrival(event);
    }

    pub(crate) fn pt_teleportation_arrival(&mut self, event: &PtTeleportationArrivalEvent) {
        self.partial_plan(&event.person)
            .pt_teleportation_arrival(event);
    }

    pub(crate) fn person_stuck(&mut self, event: &PersonStuckEvent) {
        self.partial_plan(&event.person).person_stuck(event);
    }

    pub(crate) fn link_enter(&mut self, event: &LinkEnterEvent) {
        self.for_each_occupant(&event.vehicle, |plan| plan.link_enter(event));
    }

    pub(crate) fn vehicle_enters_traffic(&mut self, event: &VehicleEntersTrafficEvent) {
        self.for_each_occupant(&event.vehicle, |plan| plan.vehicle_enters_traffic(event));
    }

    pub(crate) fn vehicle_leaves_traffic(&mut self, event: &VehicleLeavesTrafficEvent) {
        self.for_each_occupant(&event.vehicle, |plan| plan.vehicle_leaves_traffic(event));
    }

    /// Passes an event of a vehicle to the backpacks of all its occupants. Events of vehicles
    /// without occupants on this partition are ignored.
    fn for_each_occupant(
        &mut self,
        vehicle: &Id<InternalVehicle>,
        mut handle: impl FnMut(&mut PartialPlan),
    ) {
        let Some(occupants) = self.vehicle_id2person_ids.get(vehicle) else {
            return;
        };
        for person in occupants.iter() {
            handle(Self::backpack(&mut self.person_id2backpack, person).partial_plan_mut());
        }
    }

    fn partial_plan(&mut self, person: &Id<InternalPerson>) -> &mut PartialPlan {
        Self::backpack(&mut self.person_id2backpack, person).partial_plan_mut()
    }

    fn backpack<'a>(
        backpacks: &'a mut IntMap<Id<InternalPerson>, Backpack>,
        person: &Id<InternalPerson>,
    ) -> &'a mut Backpack {
        backpacks.get_mut(person).unwrap_or_else(|| {
            panic!(
                "No backpack is available for person {} while handling an event.",
                person.external()
            )
        })
    }

    #[hotpath::measure]
    pub(crate) fn send(&mut self, entity: PartitionChangeEntity<'_>) -> BackpackingAttachment {
        let person_ids = Self::person_ids(entity);
        if let PartitionChangeEntity::Vehicle(vehicle) = entity {
            self.vehicle_id2person_ids.remove(vehicle.id());
        }

        let backpacks = person_ids
            .into_iter()
            .map(|person_id| {
                self.person_id2backpack
                    .remove(&person_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "No backpack is available for departing person {}.",
                            person_id.external()
                        )
                    })
            })
            .collect();
        BackpackingAttachment { backpacks }
    }

    #[hotpath::measure]
    pub(crate) fn receive(
        &mut self,
        entity: PartitionChangeEntity<'_>,
        attachment: BackpackingAttachment,
    ) {
        let expected_person_ids = Self::person_ids(entity);
        assert_eq!(
            attachment.backpacks.len(),
            expected_person_ids.len(),
            "Backpacking attachment contains the wrong number of backpacks."
        );

        for (expected_id, backpack) in expected_person_ids.iter().zip(attachment.backpacks) {
            assert_eq!(
                backpack.person_id(),
                expected_id,
                "Backpacking attachment contains a backpack for the wrong person."
            );
            let previous = self
                .person_id2backpack
                .insert(expected_id.clone(), backpack);
            assert!(
                previous.is_none(),
                "A backpack for arriving person {} is already present.",
                expected_id.external()
            );
        }

        if let PartitionChangeEntity::Vehicle(vehicle) = entity {
            let occupants = VehicleOccupants::from_persons(expected_person_ids);
            let previous = self
                .vehicle_id2person_ids
                .insert(vehicle.id().clone(), occupants);
            assert!(
                previous.is_none(),
                "A vehicle mapping for arriving vehicle {} is already present.",
                vehicle.id().external()
            );
        }
    }

    fn person_ids(entity: PartitionChangeEntity<'_>) -> Vec<Id<InternalPerson>> {
        match entity {
            PartitionChangeEntity::Vehicle(vehicle) => std::iter::once(vehicle.driver().id())
                .chain(vehicle.passengers().iter().map(Identifiable::id))
                .cloned()
                .collect(),
            PartitionChangeEntity::TeleportationAgent(agent) => vec![agent.id().clone()],
        }
    }

    #[hotpath::measure]
    pub(crate) fn finish(&mut self) -> IntMap<Id<InternalPerson>, PersonExperience> {
        self.person_id2backpack
            .drain()
            .map(|(person_id, backpack)| (person_id, backpack.finish()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::scenario::vehicles::InternalVehicle;
    use crate::simulation::vehicles::SimulationVehicle;
    use crate::test_utils::create_agent;
    use macros::deterministic_id_test;
    use nohash_hasher::IntSet;

    #[deterministic_id_test]
    fn migrates_driver_and_passenger_and_rebuilds_vehicle_mapping() {
        let driver = create_agent(1, vec!["destination"]);
        let passenger = create_agent(2, vec!["destination"]);
        let driver_id = driver.id().clone();
        let passenger_id = passenger.id().clone();
        let vehicle = SimulationVehicle::new(
            InternalVehicle::new(10, 0, 1.0, 1.0),
            Some(driver),
            vec![passenger],
        );
        let mut departing =
            BackpackingDataCollector::new(vec![driver_id.clone(), passenger_id.clone()]);
        departing.reset_iteration();
        departing.vehicle_id2person_ids.insert(
            vehicle.id().clone(),
            VehicleOccupants::from_persons([driver_id.clone(), passenger_id.clone()]),
        );

        let attachment = departing.send(PartitionChangeEntity::Vehicle(&vehicle));
        assert!(departing.person_id2backpack.is_empty());
        assert!(departing.vehicle_id2person_ids.is_empty());

        let mut arriving = BackpackingDataCollector::new(Vec::new());
        arriving.receive(PartitionChangeEntity::Vehicle(&vehicle), attachment);
        assert_eq!(arriving.person_id2backpack.len(), 2);
        let occupants: IntSet<_> = arriving
            .vehicle_id2person_ids
            .get(vehicle.id())
            .unwrap()
            .iter()
            .cloned()
            .collect();
        let expected: IntSet<_> = [driver_id, passenger_id].into_iter().collect();
        assert_eq!(occupants, expected);
    }

    #[deterministic_id_test]
    fn vehicle_occupants_behave_like_a_set() {
        let [a, b, c] = ["a", "b", "c"].map(Id::<InternalPerson>::create);
        let mut occupants = VehicleOccupants::new(a.clone());
        occupants.insert(b.clone());
        occupants.insert(a.clone());
        occupants.insert(c.clone());
        occupants.insert(b.clone());
        let members = |occupants: &VehicleOccupants| {
            let mut members: Vec<_> = occupants.iter().map(|p| p.external().to_string()).collect();
            members.sort();
            members
        };
        assert_eq!(members(&occupants), ["a", "b", "c"]);

        // Removing a person that isn't an occupant changes nothing.
        assert!(!occupants.remove(&Id::create("d")));
        assert_eq!(members(&occupants), ["a", "b", "c"]);
        // Removing the first occupant moves another one to the front.
        assert!(!occupants.remove(&a));
        assert_eq!(members(&occupants), ["b", "c"]);
        assert!(!occupants.remove(&c));
        assert_eq!(members(&occupants), ["b"]);
        assert!(occupants.remove(&b));
    }
}
