use clap::Parser;
use matsim_rust::simulation::config::{
    CompressionType, Config, Facilities, Logging, MetisOptions, ModalLinkSelection, Network,
    NonMainModeLinks, OverwriteFiles, QSim, Replanning, Routing, Scoring, StrategySetting,
    TeleportedParams, TrafficDynamics, Transit, TransitSimulation, TravelTimeCalculator, Vehicles,
    WriteEvents, parse_key_val,
};
use matsim_rust::simulation::controller::controller::ControllerBuilder;
use matsim_rust::simulation::logging::init_std_out_logging_thread_local;
use matsim_rust::simulation::scenario::Scenario;
use matsim_rust::simulation::scenario::population::Population as ScenarioPopulation;
use std::collections::HashSet;

use experiment_machine::config::GlobalConfig;
use matsim_rust::simulation::config::LinkDynamics::PassingQ;
use matsim_rust::simulation::config::PartitionMethod::Metis;
use matsim_rust::simulation::config::RoutingMode::UsePlans;
use matsim_rust::simulation::config::ScoringMode::Enabled;
use matsim_rust::simulation::config::VehiclesSource::ModeVehicleTypesFromVehiclesData;
use matsim_rust::simulation::id::Id;
use matsim_rust::simulation::io::xml::attributes::IOAttribute;
use matsim_rust::simulation::io::xml::population::{IOPlanElement, IOPopulation};
use matsim_rust::simulation::replanning::{
    RE_ROUTE_STRATEGY_NAME, SELECT_EXP_BETA_STRATEGY_NAME, WORST_SCORE_STRATEGY_NAME,
};
use matsim_rust::simulation::scenario::population::InternalPopulation;
use pre_postprocessing::utils::replace_placeholders_in_path_pattern;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct CommandLineArgs {
    /// number of iterations
    #[arg(long, short)]
    iterations: usize,
    /// parameter for the time step and vehicle size, interpreted as 1/beta seconds and 1/beta^2 pce
    #[arg(long, short)]
    beta: usize,
    /// random seed to use for the new experiments
    #[arg(long)]
    use_random_seed: usize,
    /// name of the current experiment set, used to create the output directory
    #[arg(long)]
    experiment_set_name: String,
    /// directory to store the output
    #[arg(long)]
    base_output_dir: String,
    /// optionally, these key-val pairs can be used to override specific fields in the config
    #[arg(long= "set", value_parser = parse_key_val)]
    overrides: Vec<(String, String)>,
    /// if set, the output directory will be deleted if it already exists
    #[arg(long)]
    delete_output_dir_if_existing: bool,
    /// global config file path
    #[arg(long)]
    global_config_path: String,
    /// sample size for the simulation, between 0.0 and 1.0
    #[arg(long)]
    sample_size: f64,
    /// number of partitions that rust_qsim should use
    #[arg(long)]
    num_parts: usize,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    let args = CommandLineArgs::parse();
    info!("Started with args: {:?}", args);

    let global_config_path = PathBuf::from(&args.global_config_path);

    let global_config: GlobalConfig =
        serde_yaml::from_reader(std::fs::File::open(&global_config_path).unwrap())
            .expect("Failed to read global config");

    info!("Loaded global config from {}", global_config_path.display());

    // ------ load parameters from input

    // represents vehicle size and time step
    let beta = args.beta;

    let seed_rust = args.use_random_seed;

    // ------ load file and directory patterns from global config
    let experiment_output_dir_pattern = global_config
        .try_get_global_config_parameter_as_str("common_output_from_runs_pattern")
        .expect("Failed to get common_output_from_runs_pattern from global config");

    let pop_to_use_as_input_pattern = global_config
        .try_get_global_config_parameter_as_str("pop_to_use_as_input_pattern")
        .expect("Failed to get pop_to_use_as_input_pattern from global config");

    let vehicle_file_pattern = global_config
        .try_get_global_config_parameter_as_str("vehicle_file_pattern")
        .expect("Failed to get vehicle_file_pattern from global config");

    let network_file_pattern = global_config
        .try_get_global_config_parameter_as_str("network_file_pattern")
        .expect("Failed to get network_file_pattern from global config");

    let facilities_file_pattern = global_config
        .try_get_global_config_parameter_as_str("facilities_file_pattern")
        .expect("Failed to get facilities_file_pattern from global config");

    let transit_schedule_file_pattern = global_config
        .try_get_global_config_parameter_as_str("transit_schedule_file_pattern")
        .expect("Failed to get transit_schedule_file_pattern from global config");

    let transit_vehicles_file_pattern = global_config
        .try_get_global_config_parameter_as_str("transit_vehicles_file_pattern")
        .expect("Failed to get transit_vehicles_file_pattern from global config");

    let scoring_parameters_file_pattern = global_config
        .try_get_global_config_parameter_as_str("scoring_parameters_file_pattern")
        .expect("Failed to get scoring_parameters_file_pattern from global config");

    // ------ replace placeholders in the patterns with actual values

    let vehicle_file_path = replace_placeholders_in_path_pattern(
        // first replace placeholders from the global config (such as {original_dir_to_read_from}),
        // then replace the placeholders for this run
        &global_config
            .replace_common_pattern(&vehicle_file_pattern)
            .expect("Failed to replace common pattern in vehicle_file_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        // Some(&args.replanning_variant.to_string()),
        Some(beta),
        None, // random_java is not used in the dresden runner
        None, // use_random_seed is not used in the vehicle input path
    )
    .into();

    let network_file_path = replace_placeholders_in_path_pattern(
        // first replace placeholders from the global config (such as {original_dir_to_read_from}),
        // then replace the placeholders for this run
        &global_config
            .replace_common_pattern(&network_file_pattern)
            .expect("Failed to replace common pattern in network_file_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        // Some(&args.replanning_variant.to_string()),
        Some(beta),
        None, // random_java is not used in the dresden runner
        None, // use_random_seed is not used in the network input path
    )
    .into();

    let facilities_file_path = replace_placeholders_in_path_pattern(
        // first replace placeholders from the global config (such as {original_dir_to_read_from}),
        // then replace the placeholders for this run
        &global_config
            .replace_common_pattern(&facilities_file_pattern)
            .expect("Failed to replace common pattern in facilities_file_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        // Some(&args.replanning_variant.to_string()),
        Some(beta),
        None, // random_java is not used in the dresden runner
        None, // use_random_seed is not used in the facilities input path
    )
    .into();

    let transit_schedule_file_path = replace_placeholders_in_path_pattern(
        // first replace placeholders from the global config (such as {original_dir_to_read_from}),
        // then replace the placeholders for this run
        &global_config
            .replace_common_pattern(&transit_schedule_file_pattern)
            .expect("Failed to replace common pattern in transit_schedule_file_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        // Some(&args.replanning_variant.to_string()),
        Some(beta),
        None, // random_java is not used in the dresden runner
        None, // use_random_seed is not used in the transit schedule input path
    );

    let transit_vehicles_file_path = replace_placeholders_in_path_pattern(
        // first replace placeholders from the global config (such as {original_dir_to_read_from}),
        // then replace the placeholders for this run
        &global_config
            .replace_common_pattern(&transit_vehicles_file_pattern)
            .expect("Failed to replace common pattern in transit_vehicles_file_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        // Some(&args.replanning_variant.to_string()),
        Some(beta),
        None, // random_java is not used in the dresden runner
        None, // use_random_seed is not used in the transit vehicles input path
    );

    let scoring_parameters_file_path = replace_placeholders_in_path_pattern(
        // first replace placeholders from the global config (such as {original_dir_to_read_from}),
        // then replace the placeholders for this run
        &global_config
            .replace_common_pattern(&scoring_parameters_file_pattern)
            .expect("Failed to replace common pattern in scoring_parameters_file_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        // Some(&args.replanning_variant.to_string()),
        Some(beta),
        None, // random_java is not used in the dresden runner
        None, // use_random_seed is not used in the scoring parameters file path
    )
    .into();

    let output_dir = replace_placeholders_in_path_pattern(
        experiment_output_dir_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(beta),
        None, // random_java is not used in the dresden runner
        Some(seed_rust),
    )
    .into();

    let pop_to_use_as_input_path: PathBuf = replace_placeholders_in_path_pattern(
        // first replace common patterns from the global config, such as {original_dir_to_read_from}
        &global_config
            .replace_common_pattern(&pop_to_use_as_input_pattern)
            .expect("Failed to replace common pattern in pop_to_use_as_input_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(beta),
        None, // random_java is not used in the dresden runner
        None, // use_random_seed is not needed for the input population path
    )
    .into();

    // ------ Construct config
    let mut config = Config::default();

    // we read the initial config file from yaml, since it contains too many activity scoring parameters
    // to define them in code.
    // let mut config: Config = serde_yaml::from_reader(
    //     std::fs::File::open("./runners/src/simplified_dresden_config.yaml").unwrap(),
    // )
    // .expect("Failed to read config file");

    config.set_vehicles(Vehicles {
        path: Some(vehicle_file_path),
    });

    // Note: we don't set the population here, since we will modify it before the run (see below).
    // This happens after the scenario has been generated from the config.
    // config.set_population(Population {
    //     path: Some(pop_to_use_as_input_path),
    // });

    config.set_network(Network {
        path: Some(network_file_path),
    });

    config.set_facilities(Facilities {
        path: Some(facilities_file_path),
        modal_link_selection: ModalLinkSelection::BaseLinkFirst, // TODO not sure what they used in the original experiment.
    });

    config.set_qsim(QSim {
        start_time: 0,
        end_time: 36 * 3600,
        ticks_per_second: beta as u32,
        sample_size: args.sample_size,
        stuck_threshold: 30, // Note: this is time in seconds, not ticks.
        main_modes: vec![
            "truck8t".to_string(),
            "truck40t".to_string(),
            "truck18t".to_string(),
            "car".to_string(),
            "bike".to_string(),
        ],
        traffic_dynamics: TrafficDynamics::Queue, // Note: they used kinematic waves, but since we don't know if it works for time steps <1, we use queue
        vehicles_source: ModeVehicleTypesFromVehiclesData, //  TODO they used ModeVehicleTypesFromVehiclesData, but since I have their actual vehicles in the output, that feels more robust
        // vehicles_source: FromVehiclesData, // ModeVehicleTypesFromVehiclesData,  TODO they used ModeVehicleTypesFromVehiclesData, but since I have their actual vehicles in the output, that feels more robust
        link_dynamics: PassingQ,
    });
    // Note: they use linkDynamics PassingQ, which we don't have.

    config.set_replanning(Replanning {
        fraction_of_iterations_to_disable_innovation: 0.9,
        max_agent_plan_memory: 5,
        plan_selector_for_removal: WORST_SCORE_STRATEGY_NAME.to_string(),
        strategy_settings: vec![
            StrategySetting::new(
                RE_ROUTE_STRATEGY_NAME.to_string(),
                0.22222222222222224,
                "person".to_string(),
            ),
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                1.5454545454545452,
                "person".to_string(),
            ),
            // NOTE: we are missing SubTourModeChoice, TimeAllocationMutator
            // NOTE: this is a new subpopulation, containing all persons that have mixed plans, and
            // thus cannot be rerouted.
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                1.0,
                "person_dontroute".to_string(),
            ),
            // TODO: possibly, I could skip rerouting on these, since they should be kind of constant? -> I will keep them, since I throw away their routes
            StrategySetting::new(
                RE_ROUTE_STRATEGY_NAME.to_string(),
                0.1,
                "goodsTraffic".to_string(),
            ),
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                0.85,
                "goodsTraffic".to_string(),
            ),
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                1.0,
                "goodsTraffic_dontroute".to_string(),
            ),
            StrategySetting::new(
                RE_ROUTE_STRATEGY_NAME.to_string(),
                0.1,
                "commercialPersonTraffic".to_string(),
            ),
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                0.85,
                "commercialPersonTraffic".to_string(),
            ),
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                1.0,
                "commercialPersonTraffic_dontroute".to_string(),
            ),
            StrategySetting::new(
                RE_ROUTE_STRATEGY_NAME.to_string(),
                0.1,
                "commercialPersonTraffic_service".to_string(),
            ),
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                0.85,
                "commercialPersonTraffic_service".to_string(),
            ),
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                1.0,
                "commercialPersonTraffic_service_dontroute".to_string(),
            ),
            StrategySetting::new(
                RE_ROUTE_STRATEGY_NAME.to_string(),
                0.05,
                "longDistanceFreight".to_string(),
            ),
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                0.95,
                "longDistanceFreight".to_string(),
            ),
            StrategySetting::new(
                SELECT_EXP_BETA_STRATEGY_NAME.to_string(),
                1.0,
                "longDistanceFreight_dontroute".to_string(),
            ),
        ],
    });

    // Note: we are missing the replanningAnnealer

    config.set_routing(Routing {
        mode: UsePlans,
        access_egress_mode: "walk".to_string(),
        network_modes: vec![
            "car".to_string(),
            "ride".to_string(),
            "bike".to_string(),
            "truck8t".to_string(),
            "truck18t".to_string(),
            "truck40t".to_string(),
        ],
        teleported_mode_params: vec![TeleportedParams {
            mode: "walk".to_string(),
            teleported_mode_speed: 1.0555556,
            beeline_distance_factor: 1.3,
        }],
        // Note: I wasn't able to set the routing randomness, but they kept it at default 3.0, I guess that's what we use in rust as well (or we don't use it at all)
    });

    config.set_scoring(Scoring {
        mode: Enabled,
        write_experienced_plans: true,
        fraction_of_iterations_to_start_score_msa: Some(0.9),
        scoring_parameters_path: Some(scoring_parameters_file_path),
    });

    // Note: scoring is read from the yaml file.

    // Note: we are missing subtourModeChoice, timeAllocationMutator

    config.set_transit(Transit {
        schedule_path: Some(transit_schedule_file_path.into()),
        vehicles_path: Some(transit_vehicles_file_path.into()),
        simulation: TransitSimulation::ScheduleBasedTeleportation,
        passenger_modes: vec!["pt".to_string()],
    });

    // Note: we are missing the transit router, but should be fine, since we don't want to reroute pt

    config.set_travel_time_calculator(TravelTimeCalculator {
        bin_size: 900.0,
        // TODO: how should I handle this? in my braess experiments, I used 1/beta seconds, but in the original java experiments, they used 900 seconds. I could scale this down with beta, but it might lead to crazy slowdown.
        //  I think it is fine to keep it as it is, since it is not dependent on the time step anyway. It was just necessary in braess.
    });

    config.computational_setup_mut().random_seed = seed_rust as u64;

    config.output_mut().output_dir = output_dir;
    config.output_mut().logging = Logging::Info;
    config.output_mut().write_events = WriteEvents::File;
    // write xml.zst instead of binbp (is human-readable)
    config.output_mut().compression_type = CompressionType::Zst;
    config.output_mut().run_id = Some("001".to_string());

    if args.delete_output_dir_if_existing {
        config.output_mut().overwrite_files = OverwriteFiles::DeleteDirectoryIfExists;
    }
    config.controller_mut().last_iteration = args.iterations as u32;

    config.partitioning_mut().method = Metis(MetisOptions::default());
    config.partitioning_mut().num_parts = args.num_parts as u32;
    config.partitioning_mut().non_main_mode_links = NonMainModeLinks::default();

    // config.travel_time_calculator_mut().bin_size = 1.0 / args.beta as f64;

    config.apply_overrides(&args.overrides);

    let config = Arc::new(config);

    // Load and adapt mod
    let mut scenario = Scenario::load(config.clone());

    // ------ prepare for rerun - delete routes etc

    // First, scaling down the vehicle sizes by 1/beta^2
    for (_, vehicle_type) in scenario.garage.vehicle_types.iter_mut() {
        vehicle_type.pce = vehicle_type.pce / (beta * beta) as f64;
        vehicle_type.length = vehicle_type.length / (beta * beta) as f64;
    }
    for (_, vehicle) in scenario.garage.vehicles.iter_mut() {
        vehicle.pce = vehicle.pce / (beta * beta) as f64;
    }

    // We will also need to have beta^2 as many vehicles to keep the same flow, since we scaled down
    // the vehicle sizes. This also means that we need to have beta^2 as many persons, since the
    // persons are the ones that use the vehicles.
    let needed_copies_of_persons_and_vehs = beta * beta - 1; // -1 since one copy already exists

    // Since mutating an internal population is not really possible (not pub interface), we will
    // instead read the population file into an IOPopulation, mutate it, and only then convert it to
    // an internal population.

    // While mutating it, before creating copies of the persons, we will also delete the routes of
    // all legs that have a routing mode that we want to use in this run. And we will reset all
    // scores, since it is not clear if rust scores in the same way as Java (there might be scoring
    // params that we cannot respect here). Lastly, we will set all plans to unselected, to get an
    // unbiased start.

    // these are the network modes used in ROUTING, that is, all network modes, i.e., everything but pt.
    // For these routing modes, we will delete the present routes, so that they are rerouted instantly.
    // For plans containing other routing modes (that is, pt), the routes will be kept, and the
    // persons will be marked as dontroute. Note that all of their trips will keep their routes, even
    // if some of them might be of a routing mode that we support. This is because routing can only
    // be turned on and off for a whole person.
    let routing_modes_to_use = HashSet::from_iter(
        scenario
            .config
            .routing()
            .network_modes
            .iter()
            .map(|s| s.as_str()),
    );

    // read IOPopulation from original population path, since the internal population interface
    // is not public
    let mut population_as_io = IOPopulation::from_file(pop_to_use_as_input_path);

    let mut num_dontroute_persons = 0usize;
    let mut num_deleted_routes = 0usize;

    // prepare the plans, by throwing out routes that I want to route, and only keeping selected plans
    for person in population_as_io.persons.iter_mut() {
        let mut new_plans = Vec::with_capacity(1); // we will only keep the selected plan, so we can preallocate for one plan
        // go through all plans of the person
        for plan in person.plans.iter_mut() {
            if plan.selected == false {
                // don't consider this plan, since we want to keep only the selected plan for all agents.
                continue;
            }

            // reset all scores to None, since the scoring might not be identical in rust
            plan.score = None;

            // get all routing modes of the legs in the plan. This is because we cannot route all
            // routing modes in rust yet, so if the plan contains such a routing mode, we must mark
            // the person as dontroute, so that they keep their initial plan and do not get
            // rerouted.
            let routing_modes_in_plan = plan
                .elements
                .iter()
                .filter_map(|e| {
                    IOPlanElement::get_leg(Some(e))
                        .and_then(|l| l.attributes.as_ref().and_then(|a| a.find("routingMode")))
                })
                .collect::<HashSet<&str>>();

            // if the routing modes in the plan are not a subset of the routing modes we want to use,
            // mark this person as dontroute
            if !routing_modes_to_use.is_superset(&routing_modes_in_plan) {
                if let Some(attrs) = person.attributes.as_mut() {
                    // remove the current subpopulation attribute but save its name
                    let old_subpopulation = attrs.attributes.remove(
                        attrs
                            .attributes
                            .iter()
                            .position(|a| a.name == "subpopulation")
                            .expect("subpopulation attribute not found"),
                    );

                    // set new subpopulation attribute
                    let new_subpopulation = IOAttribute::new_with_class(
                        old_subpopulation.name,
                        old_subpopulation.class,
                        format!("{}_dontroute", old_subpopulation.value),
                    );

                    // put new subpopulation attribute into the person's attributes
                    attrs.attributes.push(new_subpopulation);

                    // update counter
                    num_dontroute_persons += 1;
                }
            } else {
                // If the person has a plan with only routing modes that we can route

                // Delete the routes:
                // go through all plan elements
                for element in plan.elements.iter_mut() {
                    match element {
                        // for legs, ...
                        IOPlanElement::Leg(leg) => {
                            // delete the route
                            leg.route = None;
                            num_deleted_routes += 1;
                        }
                        // for activities, do nothing
                        IOPlanElement::Activity(_) => {}
                    }
                }
            }

            if plan.selected == false {
                panic!("Found unselected plan, but we should only keep selected plans.");
            }
            // add the edited plan to the new plans vector
            new_plans.push(plan.clone());
        }
        // add the new plans (it is just one)
        person.plans = new_plans;
    }

    info!(
        "Prepared population for rerun:\n\
        {} persons marked as dontroute because their selected plan contained routing modes that are\n\
        not routable in rust.¸\n\
        These persons will simply keep their selected plan in this run.\n\
        In those persons that are being routed, {} routes deleted.",
        num_dontroute_persons, num_deleted_routes
    );

    // create beta*beta - 1 copies of all persons and vehicles

    // collect the copies of the persons here
    let mut new_persons = Vec::new();
    // collect the copies of the vehicles here
    let mut new_vehicles = Vec::new();

    for i in 0..needed_copies_of_persons_and_vehs {
        // create copies of all (io) persons in the (io) population
        for person in population_as_io.persons.iter() {
            let mut new_person = person.clone();
            // give the new person a new id, by appending "_copy{i}" to the original id
            new_person.id = format!("{}_copy{}", person.id.clone(), i);

            for plan in new_person.plans.iter_mut() {
                for element in plan.elements.iter_mut() {
                    if let IOPlanElement::Leg(leg) = element {
                        // if the leg contains a route (must be a person in a dontroute subpop),
                        // then we need to fix the vehicle id in the route, since it must fit to the
                        // new person id
                        if let Some(route) = leg.route.as_mut() {
                            if route.vehicle.is_some() {
                                let routing_mode = leg
                                    .attributes
                                    .as_ref()
                                    .unwrap()
                                    .find("routingMode")
                                    .unwrap()
                                    .to_string();
                                route.vehicle = Some(format!("{}_{}", new_person.id, routing_mode));
                            }
                        }
                    }
                }
            }
            new_persons.push(new_person);
        }

        // create copies of all vehicles in the garage
        for (vehicle_id, vehicle) in scenario.garage.vehicles.iter_mut() {
            let mut new_vehicle = vehicle.clone();

            // split the vehicle id into parts, insert copy{i} before the last part (which is the routing mode string)
            let mut vehicle_id_parts = vehicle_id.external().split('_').collect::<Vec<&str>>();
            let second_to_last_part = format!("copy{}", i);
            vehicle_id_parts.insert(vehicle_id_parts.len() - 1, &second_to_last_part);

            new_vehicle.id = Id::create(&vehicle_id_parts.join("_"));

            new_vehicles.push(new_vehicle);
        }
    }

    info!(
        "Created {} copies of every person, i.e., {} new persons.",
        needed_copies_of_persons_and_vehs,
        new_persons.len()
    );
    info!(
        "Created {} copies of every vehicle, i.e., {} new vehicles.",
        needed_copies_of_persons_and_vehs,
        new_vehicles.len()
    );

    // add the new persons to the io population
    population_as_io.persons.extend(new_persons.into_iter());

    // convert the io population back to internal population
    scenario.population = ScenarioPopulation::from_persons(
        InternalPopulation {
            persons: population_as_io
                .persons
                .iter()
                .map(|p| p.clone().into())
                .collect(),
        }
        .persons,
    );

    info!(
        "Successfully created new population with {} persons.",
        scenario.population.persons.len()
    );

    for new_vehicle in new_vehicles.into_iter() {
        scenario.garage.add_veh(new_vehicle);
    }
    info!(
        "Successfully added new vehicles to garage, now has {} vehicles.",
        scenario.garage.vehicles.len()
    );

    // Create and run simulation
    let controller = ControllerBuilder::default_with_scenario(scenario)
        .build()
        .unwrap();

    controller.run()
}
