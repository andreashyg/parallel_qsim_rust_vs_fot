use clap::{Parser, ValueEnum};
use rust_qsim::simulation::config::{
    Config, Logging, Network, OverwriteFiles, Population, Vehicles, WriteEvents, parse_key_val,
};
use rust_qsim::simulation::controller::controller::ControllerBuilder;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use rust_qsim::simulation::scenario::Scenario;
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
    /// directory to store the output
    #[arg(long, short)]
    output_dir: String,
    /// optionally, these key-val pairs can be used to override specific fields in the config
    #[arg(long= "set", value_parser = parse_key_val)]
    overrides: Vec<(String, String)>,
    /// if set, the output directory will be deleted if it already exists
    #[arg(long)]
    delete_output_dir_if_existing: bool,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    let resource_folder = PathBuf::from("./../runs-svn/braess/refinement/no_spillback_scenario/");

    let args = CommandLineArgs::parse();
    info!("Started with args: {:?}", args);

    // represents vehicle size and time step
    let beta = args.beta;
    // which of the random runs from java to read (1-20)
    let random_java = args.read_from_random;

    let seed_rust = args.use_random_seed;
    let output_dir = PathBuf::from(args.output_dir);

    // Construct config
    let mut config = Config::default();
    config.set_vehicles(Vehicles {
        path: Some(resource_folder.join(format!("no_spillback_beta{beta}_vehicles.xml"))),
    });
    config.set_population(Population {
        path: Some(resource_folder.join(format!(
            "{}/beta{beta}/random{random_java}/beta{beta}random{random_java}.output_plans.xml.gz",
            args.replanning_variant.get_folder_name()
        ))),
    });
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

    config.apply_overrides(&args.overrides);

    let config = Arc::new(config);

    // Load and adapt mod
    let scenario = Scenario::load(config);

    // Create and run simulation
    let controller = ControllerBuilder::default_with_scenario(scenario)
        .build()
        .unwrap();

    controller.run()
}
