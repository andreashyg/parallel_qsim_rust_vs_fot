use clap::{Parser, ValueEnum};
use rust_qsim::simulation::config::{
    Config, Logging, Network, OverwriteFiles, Population, Vehicles, WriteEvents, parse_key_val,
};
use rust_qsim::simulation::controller::controller::ControllerBuilder;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use rust_qsim::simulation::scenario::Scenario;
use rust_qsim::simulation::scenario::population::Population as ScenarioPopulation;
use std::fmt::Display;

use pre_postprocessing::activity_time_replacement::replace_activity_times_and_add_dummy_coords_to_acts;

use experiment_machine::config::GlobalConfig;
use serde_yaml;
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
}

impl Display for ReplanningVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelExp1SwitchAt50 => write!(f, "sel-exp1-switch-at50"),
            Self::SelExp1SwitchAt80 => write!(f, "sel-exp1-switch-at80"),
            Self::SelExp10SwitchAt80 => write!(f, "sel-exp10-switch-at80"),
        }
    }
}

impl ReplanningVariant {
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
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct CommandLineArgs {
    /// parameter for the time step and vehicle size, interpreted as 1/beta seconds and 1/beta^2 pce
    #[arg(long, short)]
    beta: usize,
    /// counter for the random runs of the original java experiments. Plans will be taken from the
    #[arg(long)]
    read_from_random: usize,
    /// random seed to use for the new experiments
    #[arg(long)]
    use_random_seed: u64,
    /// replanning variant of the original java experiment to use. Available: `SelExp1SwitchAt50`,
    /// `SelExp1SwitchAt80`, `SelExp10SwitchAt80`
    #[arg(long)]
    replanning_variant: ReplanningVariant,
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
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    let global_config_path =
        PathBuf::from("./experiments/compare_braess_to_java/experiment_sets/global_config.yaml");

    let global_config: GlobalConfig =
        serde_yaml::from_reader(std::fs::File::open(&global_config_path).unwrap())
            .expect("Failed to read global config");

    info!("Loaded global config from {}", global_config_path.display());

    let resource_folder = PathBuf::from(
        global_config
            .global_parameters
            .get("original_dir_to_read_from")
            .expect("Failed to get original_dir_to_read_from from global config")
            .as_str()
            .expect("original_dir_to_read_from must be a string"),
    );

    let experiment_output_dir_pattern = global_config
        .global_parameters
        .get("common_output_from_runs_pattern")
        .expect("Failed to get common_output_from_runs_pattern from global config")
        .as_str()
        .expect("common_output_from_runs_pattern must be a string")
        .to_string();

    let corrected_times_pop_pattern = global_config
        .global_parameters
        .get("corrected_output_pop_pattern")
        .expect("Failed to get corrected_output_pop_pattern from global config")
        .as_str()
        .expect("corrected_output_pop_pattern must be a string")
        .to_string();

    // let resource_folder = PathBuf::from("./../runs-svn/braess/refinement/no_spillback_scenario/");

    let args = CommandLineArgs::parse();
    info!("Started with args: {:?}", args);

    // represents vehicle size and time step
    let beta = args.beta;
    // which of the random runs from java to read (1-20)
    let random_java = args.read_from_random;

    let seed_rust = args.use_random_seed;

    // let output_dir = args
    //     .experiment_output_dir_pattern
    let output_dir = experiment_output_dir_pattern // new variant, read directly from global config
        .replace("{base_output_dir}", &args.base_output_dir)
        .replace("{experiment_set_name}", &args.experiment_set_name)
        .replace("{replanning_variant}", &args.replanning_variant.to_string())
        .replace("{beta}", &beta.to_string())
        .replace("{read_from_random}", &random_java.to_string())
        .replace("{use_random_seed}", &seed_rust.to_string())
        .into();

    let corrected_times_pop_path = global_config
        .replace_common_pattern(&corrected_times_pop_pattern)
        .expect("Failed to replace common pattern in pop_with_correct_activity_times_pattern")
        .replace("{base_output_dir}", &args.base_output_dir)
        .replace("{experiment_set_name}", &args.experiment_set_name)
        .replace("{replanning_variant}", &args.replanning_variant.to_string())
        .replace("{beta}", &beta.to_string())
        .replace("{read_from_random}", &random_java.to_string())
        .into();

    // Construct config
    let mut config = Config::default();
    config.set_vehicles(Vehicles {
        path: Some(resource_folder.join(format!("no_spillback_beta{beta}_vehicles.xml"))),
    });

    config.set_population(Population {
        path: Some(corrected_times_pop_path),
    });

    // config.set_population(Population {
    //     path: Some(resource_folder.join(format!(
    //         "{}/beta{beta}/random{random_java}/beta{beta}random{random_java}.output_plans.xml.gz",
    //         args.replanning_variant.get_folder_name()
    //     ))),
    // });
    config.set_network(Network {
        path: Some(resource_folder.join("no_spillback_network.xml")),
    });

    config.computational_setup_mut().random_seed = seed_rust;
    config.qsim_mut().main_modes = vec!["car".to_string()];
    config.qsim_mut().ticks_per_second = beta as u32;
    config.qsim_mut().stuck_threshold = config.qsim().end_time;
    config.output_mut().output_dir = output_dir;
    config.output_mut().logging = Logging::Info;
    config.output_mut().write_events = WriteEvents::File;
    if args.delete_output_dir_if_existing {
        config.output_mut().overwrite_files = OverwriteFiles::DeleteDirectoryIfExists;
    }
    config.controller_mut().last_iteration = 0;

    config.apply_overrides(&args.overrides);

    let config = Arc::new(config);

    // Load and adapt mod
    let scenario = Scenario::load(config);
    // let mut scenario = Scenario::load(config);
    //
    // let mut scenario_garage_clone = scenario.garage.clone();
    //
    // // Population with more accurate activity times (decimals) to replace the activity times in the
    // // population loaded from the java output
    // let ref_pop_with_correct_times = ScenarioPopulation::from_file(
    //     PathBuf::from(resource_folder).join(format!(
    //         "uniteratedPlans_TimeFormatHHMMSSDOTSS/beta{beta}random1.output_plans.xml.gz"
    //     )),
    //     &mut scenario_garage_clone,
    // );
    //
    // replace_activity_times_and_add_dummy_coords_to_acts(
    //     &mut scenario.population,
    //     ref_pop_with_correct_times,
    // );
    //

    // Create and run simulation
    let controller = ControllerBuilder::default_with_scenario(scenario)
        .build()
        .unwrap();

    controller.run()
}
