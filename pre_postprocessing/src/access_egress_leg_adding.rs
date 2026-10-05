use rust_qsim::simulation::InternalAttributes;
use rust_qsim::simulation::id::Id;
use rust_qsim::simulation::scenario::Coordinate;
use rust_qsim::simulation::scenario::population::{
    InternalActivity, InternalGenericRoute, InternalLeg, InternalPlanElement, InternalRoute,
    Population,
};
use rust_qsim::simulation::time::SimTime;
use std::ops::Add;
use std::time::Duration;

pub fn add_access_egress_legs_if_necessary(
    pop_to_be_changed: &mut Population,
    access_egress_mode: &str,
) {
    // for every person in the pop to be changed...
    for (person_id, person) in pop_to_be_changed.persons.iter_mut() {
        // ...go through all their plans...
        for plan in person.plans_mut() {
            // first check the start: get mode of the first leg and its start link.
            // If the mode is not the access egress mode, add a leg at the beginning of the plan
            // (but after the first activity) with the access egress mode, from and to the start link
            // of the first leg.
            let (first_leg_mode, start_link) = {
                let legs = plan.legs();
                let first_leg = legs.first().expect("Plan must have at least one leg");
                (
                    first_leg.mode.external().to_string(),
                    first_leg
                        .route
                        .as_ref()
                        .expect("First leg must have a route")
                        .start_link()
                        .clone(),
                )
            };

            if first_leg_mode != access_egress_mode {
                let current_planelements = &mut plan.elements;

                assert!(
                    current_planelements.first()
                        .is_some_and(|e| e.as_activity().is_some()),
                    "First plan element must be an activity"
                );
                assert!(
                    current_planelements
                        .get(1)
                        .is_some_and(|e| e.as_leg().is_some()),
                    "Second plan element must be a leg"
                );

                let mut leg_internal_attributes = InternalAttributes::default();
                leg_internal_attributes.add("routingMode", &first_leg_mode);

                // add a leg at the beginning of the plan (but after the first activity) with the access egress mode
                let leg_to_add = InternalPlanElement::Leg(InternalLeg {
                    mode: Id::create(access_egress_mode),
                    routing_mode: Some(Id::create(&first_leg_mode)),
                    dep_time: Some(SimTime::from_secs(0)),
                    trav_time: Some(Duration::from_secs(1)),
                    attributes: leg_internal_attributes,
                    route: Some(
                        // route from start to start link (dummy route)
                        InternalRoute::Generic(InternalGenericRoute::new(
                            start_link.clone(),
                            start_link.clone(),
                            Some(Duration::from_secs(1)),
                            Some(1.0), // distance
                            Some(Id::create(&format!("{}_walk", person_id.external()))),
                        )), // vehicle_id
                    ),
                });
                // we also need an interaction activity
                let activity_to_add = InternalPlanElement::Activity(InternalActivity::new(
                    Some(Coordinate::new_2d(0.0, 0.0)),
                    "car interaction",
                    start_link.clone(),
                    None,
                    None,
                    Some(Duration::from_secs(0)),
                ));

                current_planelements.insert(1, leg_to_add);
                current_planelements.insert(2, activity_to_add);

                // Now shift all times of the activities and legs after the first one by 1 second,
                // since we added a leg at the beginning of the plan, which takes 1 second.

                // get all activities except for the first one, since the first is before the added access egress leg and should not be shifted
                let mut all_acts = plan.acts_mut();

                for act in all_acts.iter_mut().skip(1) {
                    // shift all activity start times by 1 second, since we added the access egress leg
                    // at the beginning of the plan, which takes 1 second

                    act.start_time = act
                        .start_time
                        .map(|start_t| start_t.add(SimTime::from_secs(1)));

                    // same thing for end time
                    act.end_time = act.end_time.map(|end_t| end_t.add(SimTime::from_secs(1)));
                }

                let mut all_legs = plan.legs_mut();
                for leg in all_legs.iter_mut().skip(1) {
                    // shift all leg start times by 1 second, since we added the access egress leg at the beginning of the plan, which takes 1 second
                    leg.dep_time = leg.dep_time.map(|dep_t| dep_t.add(SimTime::from_secs(1)));
                }
            }

            // now check the end: get mode of the last leg and its end link. If the mode is not the
            // access egress mode, add a leg at the end of the plan with the access egress mode,
            // from and to the end link of the last leg.
            let (
                last_leg_mode,
                end_link,
            ) = {
                let legs = plan.legs();
                let last_leg = legs.last().expect("Plan must have at least one leg");
                (
                    last_leg.mode.external().to_string(),
                    last_leg
                        .route
                        .as_ref()
                        .expect("Last leg must have a route")
                        .end_link()
                        .clone(),
                )
            };

            if last_leg_mode != access_egress_mode {
                let current_planelements = &mut plan.elements;

                assert!(
                    current_planelements
                        .last()
                        .is_some_and(|e| e.as_activity().is_some()),
                    "Last plan element must be an activity"
                );
                assert!(
                    current_planelements
                        .get(current_planelements.len() - 2)
                        .is_some_and(|e| e.as_leg().is_some()),
                    "Second to last plan element must be a leg"
                );

                let mut leg_internal_attributes = InternalAttributes::default();
                leg_internal_attributes.add("routingMode", &last_leg_mode);

                // add a leg at the end of the plan (but before the last activity) with the access egress mode
                let leg_to_add = InternalPlanElement::Leg(InternalLeg {
                    mode: Id::create(access_egress_mode),
                    routing_mode: Some(Id::create(&last_leg_mode)),
                    dep_time: None, //Some(new_leg_dep_time),
                    trav_time: Some(Duration::from_secs(1)),
                    attributes: leg_internal_attributes,
                    // route from end link to end link (dummy route)
                    route: Some(InternalRoute::Generic(InternalGenericRoute::new(
                        end_link.clone(),
                        end_link.clone(),
                        Some(Duration::from_secs(1)),
                        Some(1.0), // distance
                        Some(Id::create(&format!("{}_walk", person_id.external()))), // vehicle_id
                    ))),
                    // use access egress mode
                });
                // we also need an interaction activity before the new last leg
                let activity_to_add = InternalPlanElement::Activity(InternalActivity::new(
                    Some(Coordinate::new_2d(0.0, 0.0)),
                    "car interaction",
                    end_link.clone(),
                    None,
                    None,
                    Some(Duration::from_secs(0)), // max_dur
                ));

                // add car interaction activity before the last activity
                current_planelements.insert(current_planelements.len() - 1, activity_to_add);
                // add new access egress leg before the last activity (and after the car interaction activity)
                current_planelements.insert(current_planelements.len() - 1, leg_to_add);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_qsim::simulation::scenario::vehicles::Garage;
    #[test]
    fn test_add_access_egress_legs() {
        // shortened version of braess/refinement/no_spillback_scenario/2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it/beta2/random1/beta2random1.output_plans.xml.gz
        let mut pop_to_be_changed = Population::from_file(
            "./../pre_postprocessing/src/tests/resources/test_access_egress_leg_adding/pop_to_be_changed.xml",
            &mut Garage::new(),
        );
        // Shortened version of braess/refinement/no_spillback_scenario/uniteratedPlans_TimeFormatHHMMSSDOTSS/beta2random1.output_plans.xml.gz
        let pop_with_correct_legs = Population::from_file(
            "./../pre_postprocessing/src/tests/resources/test_access_egress_leg_adding/pop_with_correct_legs.xml",
            &mut Garage::new(),
        );

        add_access_egress_legs_if_necessary(&mut pop_to_be_changed, "walk");

        assert_eq!(
            pop_to_be_changed, pop_with_correct_legs,
            "Population after adding access/egress legs does not match expected population"
        );
    }
}
