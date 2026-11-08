use clap::{Parser, ValueEnum};
use matsim_rust::simulation::config::{ActivityParameter, ActivityScoring, AgentParameter, CompressionType, Config, Logging, ModeParameter, Network, OverwriteFiles, Population, ScoringMode, StrategySetting, TeleportedParams, Vehicles, WriteEvents, parse_key_val};
use matsim_rust::simulation::controller::controller::ControllerBuilder;
use matsim_rust::simulation::logging::init_std_out_logging_thread_local;
use matsim_rust::simulation::scenario::Scenario;
use std::fmt::Display;

use experiment_machine::config::GlobalConfig;
use matsim_rust::simulation::config::RoutingMode::UsePlans;
use matsim_rust::simulation::id::Id;
use matsim_rust::simulation::replanning::{RE_ROUTE_STRATEGY_NAME, SELECT_EXP_BETA_STRATEGY_NAME, WORST_SCORE_STRATEGY_NAME};
use pre_postprocessing::utils::replace_placeholders_in_path_pattern;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, ValueEnum)]
enum ReplanningVariant {
    /// selection exponent ("preference for higher scores" of 1, rerouting until 0.5, msa from 0.5
    SelExp1SwitchAt50,
    /// selection exponent ("preference for higher scores" of 1, rerouting until 0.8, msa from 0.8
    SelExp1SwitchAt80,
    /// selection exponent ("preference for higher scores" of 10, rerouting until 0.8, msa from 0.8
    SelExp10SwitchAt80,
    /// selection exponent ("preference for higher scores" of 10, rerouting until 0.5, msa from 0.5
    SelExp10SwitchAt80MyOwn, // This is a custom variant for testing purposes
    SelExp10SwitchAt80MyOwnOnlyLegScoringAndBrain1Only64, // This is a custom variant for testing purposes
}

impl Display for ReplanningVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelExp1SwitchAt50 => write!(f, "sel-exp1-switch-at50"),
            Self::SelExp1SwitchAt80 => write!(f, "sel-exp1-switch-at80"),
            Self::SelExp10SwitchAt80 => write!(f, "sel-exp10-switch-at80"),
            Self::SelExp10SwitchAt80MyOwn => write!(f, "sel-exp10-switch-at80-my-own"),
            Self::SelExp10SwitchAt80MyOwnOnlyLegScoringAndBrain1Only64 => write!(f, "sel-exp10-switch-at80-my-own-only-leg-scoring-and-brain1-only64"),
        }
    }
}

impl ReplanningVariant {
    #[allow(dead_code)]
    fn get_folder_name(&self) -> &'static str {
        match self {
            Self::SelExp1SwitchAt50 => {
                "2026-05-8-12-16-8_500it_reRouteProba0.1until0.5it_selExpBeta1proba0.9_msaFrom0.5it"
            }
            Self::SelExp1SwitchAt80 => {
                "2026-05-10-10-2-21_500it_reRouteProba0.1until0.8it_selExpBeta1proba0.9_msaFrom0.8it"
            }
            Self::SelExp10SwitchAt80 => {
                "2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it"
            }
            Self::SelExp10SwitchAt80MyOwn => {
                "2026-10-14-12-22-10_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it_myFirstReplRun"
            }
            Self::SelExp10SwitchAt80MyOwnOnlyLegScoringAndBrain1Only64 => {
                "2026-10-21-23-11-32_original_settings_but_only_leg_scoring_and_brain_1_only_beta_64"
            }
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct CommandLineArgs {
    /// number of iterations
    #[arg(long, short)]
    iterations: usize,
    /// parameter for the time step and vehicle size, interpreted as 1/beta seconds and 1/beta^2 pce
    #[arg(long, short)]
    beta: usize,
    /// counter for the random runs of the original java experiments. Plans will be taken from the
    #[arg(long)]
    read_from_random: usize,
    /// random seed to use for the new experiments
    #[arg(long)]
    use_random_seed: usize,
    /// name of the current experiment set, used to create the output directory
    #[arg(long)]
    experiment_set_name: String,
    /// directory to store the output
    #[arg(long)]
    base_output_dir: String,
    /// pattern for the actual output directory, which can contain placeholders for the parameters.
    /// the placeholders are base_output_dir, experiment_set_name, replanning_variant, beta, read_from_random, use_random_seed
    #[arg(long)]
    // experiment_output_dir_pattern: String,
    /// optionally, these key-val pairs can be used to override specific fields in the config
    #[arg(long= "set", value_parser = parse_key_val)]
    overrides: Vec<(String, String)>,
    /// if set, the output directory will be deleted if it already exists
    #[arg(long)]
    delete_output_dir_if_existing: bool,
    /// global config file path
    #[arg(long)]
    global_config_path: String,
    /// pattern for the population file to use as input
    #[arg(long)]
    pop_to_use_as_input_pattern: String,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    let args = CommandLineArgs::parse();
    info!("Started with args: {:?}", args);

    let global_config_path =
        PathBuf::from(&args.global_config_path);

    let global_config: GlobalConfig =
        serde_yaml::from_reader(std::fs::File::open(&global_config_path).unwrap())
            .expect("Failed to read global config");

    info!("Loaded global config from {}", global_config_path.display());

    // ------ load parameters from input

    // input population pattern
    let pop_to_use_as_input_pattern = &args.pop_to_use_as_input_pattern;

    // represents vehicle size and time step
    let beta = args.beta;
    // which of the random runs from java to read (1-20)
    let random_java = args.read_from_random;

    let seed_rust = args.use_random_seed;

    // ------ load file and directory patterns from global config
    let experiment_output_dir_pattern = global_config.try_get_global_config_parameter_as_str(
        "common_output_from_runs_pattern").expect("Failed to get common_output_from_runs_pattern from global config");

    let vehicle_file_pattern = global_config.try_get_global_config_parameter_as_str(
        "vehicle_file_pattern").expect("Failed to get vehicle_file_pattern from global config");

    let network_file_pattern = global_config.try_get_global_config_parameter_as_str(
        "network_file_pattern").expect("Failed to get network_file_pattern from global config");

    // ------ replace placeholders in the patterns with actual values

    let vehicle_file_path = replace_placeholders_in_path_pattern(
        // first replace placeholders from the global config (such as {original_dir_to_read_from}),
        // then replace the placeholders for this run
        &global_config.replace_common_pattern(&vehicle_file_pattern)
            .expect("Failed to replace common pattern in vehicle_file_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        // Some(&args.replanning_variant.to_string()),
        Some(beta),
        Some(random_java),
        None, // use_random_seed is not used in the vehicle input path
    ).into();

    let network_file_path = replace_placeholders_in_path_pattern(
        // first replace placeholders from the global config (such as {original_dir_to_read_from}),
        // then replace the placeholders for this run
        &global_config.replace_common_pattern(&network_file_pattern)
            .expect("Failed to replace common pattern in network_file_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        // Some(&args.replanning_variant.to_string()),
        Some(beta),
        Some(random_java),
        None, // use_random_seed is not used in the network input path
    ).into();

    let output_dir = replace_placeholders_in_path_pattern(
        experiment_output_dir_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(beta),
        Some(random_java),
        Some(seed_rust),
    ).into();

    let pop_to_use_as_input_path = replace_placeholders_in_path_pattern(
        // first replace common patterns from the global config, such as {original_dir_to_read_from}
        &global_config.replace_common_pattern(&pop_to_use_as_input_pattern)
            .expect("Failed to replace common pattern in pop_to_use_as_input_pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(beta),
        Some(random_java),
        None, // use_random_seed is not needed for the input population path
    ).into();

    // ------ Construct config
    let mut config = Config::default();
    config.set_vehicles(Vehicles {
        path: Some(vehicle_file_path),
    });

    config.set_population(Population {
        path: Some(pop_to_use_as_input_path),
    });

    config.set_network(Network {
        path: Some(network_file_path)
    });

    config.computational_setup_mut().random_seed = seed_rust as u64;
    config.qsim_mut().main_modes = vec!["car".to_string()];
    config.qsim_mut().ticks_per_second = beta as u32;
    config.qsim_mut().stuck_threshold = config.qsim().end_time;
    config.output_mut().output_dir = output_dir;
    config.output_mut().logging = Logging::Info;
    config.output_mut().write_events = WriteEvents::File;
    // write xml.zst instead of binbp (is human-readable)
    config.output_mut().compression_type = CompressionType::Zst;

    config.scoring_mut().mode = ScoringMode::Enabled;

    config.scoring_mut().agent_params = vec![AgentParameter::default()];
    config.scoring_mut().activity_params = vec![
        ActivityParameter { activity_type: "dummy".to_string(), typical_duration_s: 43200.0, activity_scoring: ActivityScoring::Ignore }];
    config.scoring_mut().mode_params = vec![
        ModeParameter::default_for_mode("car"),
        ModeParameter::default_for_mode("walk")];
    config.scoring_mut().fraction_of_iterations_to_start_score_msa = Some(0.8);

    // config.
    if args.delete_output_dir_if_existing {
        config.output_mut().overwrite_files = OverwriteFiles::DeleteDirectoryIfExists;
    }
    config.controller_mut().last_iteration = args.iterations as u32;

    config.routing_mut().mode = UsePlans;
    config.routing_mut().access_egress_mode = "walk".to_string();
    config.routing_mut().network_modes = vec!["car".to_string()];
    config.routing_mut().teleported_mode_params = vec![TeleportedParams {
        mode: "walk".to_string(),
        teleported_mode_speed: 0.8333333333333333,
        beeline_distance_factor: 1.3,
    }];

    config.replanning_mut().fraction_of_iterations_to_disable_innovation = 0.8;
    config.replanning_mut().max_agent_plan_memory = 5;
    config.replanning_mut().plan_selector_for_removal = WORST_SCORE_STRATEGY_NAME.to_string();
    config.replanning_mut().strategy_settings = vec![
        StrategySetting::new(RE_ROUTE_STRATEGY_NAME.to_string(), 0.1, "person".to_string()),
        StrategySetting::new(SELECT_EXP_BETA_STRATEGY_NAME.to_string(), 0.9, "person".to_string())];


    config.travel_time_calculator_mut().bin_size = 1.0 / args.beta as f64;


    config.apply_overrides(&args.overrides);

    let config = Arc::new(config);

    // Load and adapt mod
    let mut scenario = Scenario::load(config);

    // Re-key every vehicle by the new external ID and keep the vehicle's own id field in sync.
    let renamed_vehicles = scenario
        .garage
        .vehicles
        .drain()
        .map(|(old_id, mut vehicle)| {
            let new_id = Id::create(&format!("{old_id}_car"));
            vehicle.id = new_id.clone();
            (new_id, vehicle)
        })
        .collect();
    scenario.garage.vehicles = renamed_vehicles;


    // Create and run simulation
    let controller = ControllerBuilder::default_with_scenario(scenario)
        .build()
        .unwrap();

    controller.run()
}
