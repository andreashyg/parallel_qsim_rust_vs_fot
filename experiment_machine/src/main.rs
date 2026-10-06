use clap::Parser;
use experiment_machine::config;
use experiment_machine::logging::init_tracing;
use std::fs::File;
use std::path::PathBuf;
use tracing::info;

// pub mod logging;
// pub mod bash_utils;
// pub mod config;

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
    /// if true, gnu parallel is called with --resume, so that it will resume the run of the experiment set if it was interrupted.
    #[arg(long, default_value = "false")]
    pub resume: bool,
    /// if true, gnu parallel is called with --resume-failed, so that it will resume the run of the experiment set if it was interrupted or for failed jobs.
    #[arg(long, default_value = "false", conflicts_with = "resume")]
    pub resume_failed: bool,
    /// if true, the output logs will be appended instead of overwritten, and the output configs
    /// will saved with a timestamp instead of overwriting the previous ones.
    /// This is useful to rerun parts of an experiment set but keep the output logs from the
    /// untouched parts
    #[arg(
        long,
        default_value = "false",
        conflicts_with = "resume",
        conflicts_with = "resume_failed"
    )]
    pub overwrite_and_append: bool,
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

    // check that the experiment set name in the config file matches the file name of the config
    // file.
    // If this is not the case, it is likely that the output will be saved in the wrong directory,
    // possibly overwriting previous runs. This is a safety check to prevent that from happening.
    assert_eq!(
        experiment_set_config.get_expset_config_globals_parameter("experiment_set_name")
            .expect("Missing experiment_set_name in globals"),
        args.experiment_set_config_file.file_stem()
            .expect("Failed to get file stem of experiment set config file cla")
            .to_str()
            .expect("Failed to convert file stem of experiment set config to string"),
        "Experiment set name in config file does not match the file name"
    );

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
            .expect("global_log_file_pattern must be a string")
            .replace("{base_output_dir}", base_output_dir)
            .replace("{experiment_set_name}", expset_name_str),
    );

    let global_error_log_file_path = PathBuf::from(
        &global_config
            .get_global_config_parameter("global_error_log_file_pattern")
            .expect("Missing global_error_log_file_pattern in global config")
            .as_str()
            .expect("global_error_log_file_pattern must be a string")
            .replace("{base_output_dir}", base_output_dir)
            .replace("{experiment_set_name}", expset_name_str),
    );

    // initialize logging to the global log file and stdout/stderr
    let (_main_guard, _error_guard) = init_tracing(
        global_log_file_path,
        global_error_log_file_path,
        args.resume, // this will decide whether to append to the log files or overwrite them
        args.resume_failed,
        args.overwrite_and_append,
    );

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
        .run_all_modules(&global_config, args.resume, args.resume_failed)
        .expect("Failed to run experiment set");

    info!("********** Finished run of experiment set **********");

    // Save a copy of the experiment set config and the global config to the output directory

    let output_expset_config_pattern = global_config
        .get_global_config_parameter("output_expset_config_pattern")
        .expect("Missing output_expset_config_pattern in global config")
        .as_str()
        .expect("output_expset_config_pattern must be a string");

    let output_experiment_set_config_path = PathBuf::from(
        global_config
            .replace_common_pattern(
                &output_expset_config_pattern
                    .replace("{base_output_dir}", base_output_dir)
                    .replace("{experiment_set_name}", expset_name_str),
            )
            .expect("Failed to replace common pattern in output_expset_config_pattern"),
    );

    // when overwrite_and_append is true, we want to save the output config with an index so that
    // we don't overwrite previous runs.
    let output_experiment_set_config_path = if args.overwrite_and_append {
        // get smallest free index for the output config file, so that we don't overwrite previous runs
        let mut index = 1;
        while get_output_path_with_index(&output_experiment_set_config_path, index)
            .exists()
        {
            index += 1;
        }
        get_output_path_with_index(&output_experiment_set_config_path, index)
    } else {
        // if we are not resuming (and not in overwrite_and_append mode), we want to delete any old
        // config files. The config file with no index is the one that will be overwritten, so we
        // don't need to delete it.But we need to delete the ones with an index.
        if !args.resume && !args.resume_failed {
            for index in 1.. {
                let path = get_output_path_with_index(&output_experiment_set_config_path, index);
                if !path.exists() {
                    break;
                }
                std::fs::remove_file(&path).expect("Failed to remove old config file");
            }
        }

        output_experiment_set_config_path
    };

    let output_global_config_pattern = global_config
        .get_global_config_parameter("output_global_config_pattern")
        .expect("Missing output_global_config_pattern in global config")
        .as_str()
        .expect("output_global_config_pattern must be a string");

    let output_global_config_path = PathBuf::from(
        global_config
            .replace_common_pattern(
                &output_global_config_pattern
                    .replace("{base_output_dir}", base_output_dir)
                    .replace("{experiment_set_name}", expset_name_str),
            )
            .expect("Failed to replace common pattern in output_global_config_pattern"),
    );
    let output_global_config_path = if args.overwrite_and_append {
        // get smallest free index for the output config file, so that we don't overwrite previous runs
        let mut index = 1;
        while get_output_path_with_index(&output_global_config_path, index)
            .exists()
        {
            index += 1;
        }
        get_output_path_with_index(&output_global_config_path, index)
    } else {
        // if we are not resuming (and not in overwrite_and_append mode), we want to delete any old
        // config files. The config file with no index is the one that will be overwritten, so we
        // don't need to delete it.But we need to delete the ones with an index.
        if !args.resume && !args.resume_failed {
            for index in 1.. {
                let path = get_output_path_with_index(&output_experiment_set_config_path, index);
                if !path.exists() {
                    break;
                }
                std::fs::remove_file(&path).expect("Failed to remove old config file");
            }
        }

        // since we are not in append mode, it is fine/desired to overwrite the existing file
        output_global_config_path
    };

    info!(
        "Saving copy of experiment set config to {}",
        output_experiment_set_config_path.display()
    );

    serde_yaml::to_writer(
        File::create(output_experiment_set_config_path)
            .expect("Failed to create output experiment set config file"),
        &experiment_set_config,
    )
        .expect("Failed to write output_experiment_set_config file");

    info!(
        "Saving copy of global config to {}",
        output_global_config_path.display()
    );

    serde_yaml::to_writer(
        File::create(output_global_config_path)
            .expect("Failed to create output global config file"),
        &global_config,
    )
        .expect("Failed to write output global config file");
}

/// Returns a new PathBuf with the same path as `path`, but with the file stem modified to include
/// the given index.
fn get_output_path_with_index(path: &PathBuf, index: i32) -> PathBuf {
    path
        .with_file_name(
            format!(
                "{}_{}.{}",
                path
                    .file_stem()
                    .expect(&format!("Failed to get file stem of {}", path.display()))
                    .to_str()
                    .expect(
                        &format!("Failed to convert file stem of {} to str", path.display())
                    ),
                index,
                path.extension()
                    .expect(&format!("Failed to get extension of {}", path.display()))
                    .to_str()
                    .expect(
                        &format!("Failed to convert extension of {} to str", path.display())
                    )
            )
        )
}
