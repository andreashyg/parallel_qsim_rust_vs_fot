use rust_qsim::simulation::scenario::population::Population;

pub fn replace_activity_times(
    pop_to_be_changed: &mut Population,
    pop_with_correct_times: Population,
) {
    // let mut new_pop_persons = pop_to_be_changed.persons.clone();

    // for every person in the pop to be changed...
    for (person_id, person) in pop_to_be_changed.persons.iter_mut() {
        // ...check if the person exists in the pop with correct times...
        if let Some(correct_person) = pop_with_correct_times.persons.get(&person_id) {
            // ...and if so, replace the activity times in the person with the correct times
            for plan in person.plans_mut() {
                for activity in plan.acts_mut() {
                    // there is always one activity with end time (which should be replaced) and one
                    // activity without an end time (where nothing is to be done)
                    if activity.end_time.is_some() {
                        // println!(
                        //     "current activity is: {:?}, for person {}",
                        //     activity, person_id
                        // );
                        // println!(
                        //     "correctly timed activities are: {:?}",
                        //     correct_person.selected_plan().as_ref().unwrap().acts()
                        // );
                        activity.end_time = correct_person
                            .selected_plan()
                            .as_ref()
                            .unwrap()
                            .acts()
                            .iter()
                            .find(|a| a.act_type == activity.act_type && a.end_time.is_some())
                            .unwrap()
                            .end_time
                    }
                }
            }
        }
    }
}

mod tests {
    use super::*;
    use rust_qsim::simulation::scenario::vehicles::Garage;
    use rust_qsim::simulation::time::SimTime;

    #[test]
    fn test_replace_activity_times() {
        // shortened version of braess/refinement/no_spillback_scenario/2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it/beta2/random1/beta2random1.output_plans.xml.gz
        let mut pop_to_be_changed = Population::from_file(
            "./../pre_postprocessing/src/tests/resources/test_replace_activity_times/pop_to_be_changed.xml",
            &mut Garage::new(),
        );
        // Shortened version of braess/refinement/no_spillback_scenario/uniteratedPlans_TimeFormatHHMMSSDOTSS/beta2random1.output_plans.xml.gz
        let pop_with_correct_times = Population::from_file(
            "./../pre_postprocessing/src/tests/resources/test_replace_activity_times/pop_with_correct_times.xml",
            &mut Garage::new(),
        );

        replace_activity_times(&mut pop_to_be_changed, pop_with_correct_times);

        for (person_id, person) in pop_to_be_changed.persons {
            for plan in person.plans() {
                for activity in plan.acts() {
                    if let Some(end_time) = activity.end_time {
                        assert_eq!(
                            end_time,
                            // expected time for, e.g., person 10 is 2.5 seconds, that is, 10 * 0.25 * 1e9 nanoseconds
                            SimTime::from_nanos(
                                (person_id.clone().external().parse::<f64>().unwrap() * 0.25 * 1e9)
                                    as u64
                            ),
                            "Activity end time for person {} is not correct",
                            person_id.clone()
                        );
                    }
                }
            }
        }
    }
}
