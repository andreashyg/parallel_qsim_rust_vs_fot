use crate::bash_utils::init_tracing;
use clap::Parser;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::info;

pub mod bash_utils;
pub mod config;

#[derive(Parser, Debug)]
struct InputArgs {
    /// Path to experiment set config file
    #[arg(long)]
    pub experiment_set_config_file: PathBuf,
    /// Path to global config file
    #[arg(
        long,
        default_value = "./experiments/compare_braess_to_java/experiment_sets/global_config.yaml"
    )]
    pub global_config_file: PathBuf,
}

fn main() {
    let args = InputArgs::parse();

    // load global config TODO this is not *really* global, since it is specific to compare_braess_to_java.
    let global_config: config::GlobalConfig = serde_yaml::from_reader(
        File::open(&args.global_config_file).expect("Failed to open global config file"),
    )
    .expect("Failed to parse global config file");

    //  load experiment set config
    let experiment_set_config: config::Config = serde_yaml::from_reader(
        File::open(&args.experiment_set_config_file)
            .expect("Failed to open experiment set config file"),
    )
    .expect("Failed to parse experiment set config file");

    let base_output_dir = experiment_set_config
        .get_expset_config_globals_parameter("base_output_dir")
        .expect("Missing base_output_dir in global config")
        .as_str()
        .expect("base_output_dir must be a string");

    let expset_name_str = experiment_set_config
        .get_expset_config_globals_parameter("experiment_set_name")
        .expect("Missing experiment_set_name in globals")
        .as_str()
        .expect("experiment_set_name must be a string");

    let global_log_file_path = PathBuf::from(
        &global_config
            .get_global_config_parameter("global_log_file_pattern")
            .expect("Missing global_log_file_pattern in global config")
            .as_str()
            .expect("global_log_file must be a string")
            .replace("{base_output_dir}", base_output_dir)
            .replace("{replanning_variant}", "sel-exp10-switch-at80") // FIXME this is a hack, in the future the replanning variant should be a subfolder of the experiment set
            .replace("{experiment_set_name}", expset_name_str),
    );

    // initialize logging to the global log file and stdout/stderr
    let _guard = init_tracing(global_log_file_path);

    info!(
        "Loaded global config from {}",
        args.global_config_file.display()
    );

    info!(
        "Loaded experiment set config from {}",
        args.experiment_set_config_file.display()
    );

    // run the modules specified in the config
    info!("********** Starting run of experiment set **********");
    experiment_set_config
        .run_all_modules(&global_config)
        .expect("Failed to run experiment set");

    info!("********** Finished run of experiment set **********");

    // Save a copy of the experiment set config and the global config to the output directory
    // of each replanning variant.
    let experiment_set_dir_pattern = "{base_output_dir}/{replanning_variant}/{experiment_set_name}";

    for replvar in experiment_set_config
        .get_expset_config_param_sweep_parameter("replanning_variants")
        .expect("Missing replanning_variants in param_sweep")
    {
        let replvar_str = replvar
            .as_str()
            .expect("replanning_variant must be a string");

        let experiment_set_dir = PathBuf::from(
            global_config
                .replace_common_pattern(
                    &experiment_set_dir_pattern
                        .replace("{base_output_dir}", base_output_dir)
                        .replace("{replanning_variant}", replvar_str)
                        .replace("{experiment_set_name}", expset_name_str),
                )
                .expect("Failed to replace common pattern in experiment_set_dir_pattern"),
        );

        info!(
            "Saving copy of experiment set config to {}",
            experiment_set_dir
                .join("output_expset_config.yaml")
                .display()
        );

        serde_yaml::to_writer(
            File::create(experiment_set_dir.join("output_expset_config.yaml"))
                .expect("Failed to create output_expset_config.yaml"),
            &experiment_set_config,
        )
        .expect("Failed to write output_expset_config.yaml");

        info!(
            "Saving copy of global config to {}",
            experiment_set_dir
                .join("output_global_config.yaml")
                .display()
        );

        serde_yaml::to_writer(
            File::create(experiment_set_dir.join("output_global_config.yaml"))
                .expect("Failed to create output_global_config.yaml"),
            &global_config,
        )
        .map_err(|e| format!("Failed to write output_global_config.yaml: {}", e))
        .expect("Failed to write output_global_config.yaml");
    }
}
