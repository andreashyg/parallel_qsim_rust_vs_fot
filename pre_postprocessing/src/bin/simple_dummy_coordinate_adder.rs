use std::error::Error;
use std::fs::create_dir_all;
use std::path::PathBuf;

use clap::Parser;
use experiment_machine::config::GlobalConfig;
use matsim_rust::simulation::logging::init_std_out_logging_thread_local;
use pre_postprocessing::activity_dummy_coordinates::add_dummy_coordinates_to_file;
use pre_postprocessing::utils::replace_placeholders_in_path_pattern;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// Path to input XML events file
    /// Note: *can* contain placeholders {base_output_dir}, {beta},
    /// {read_from_random} that will be replaced.
    #[arg(long)]
    pub input_file_pattern: String,
    #[arg(long)]
    pub output_file_pattern: String,
    #[arg(long)]
    pub base_output_dir: String,
    #[arg(long)]
    pub experiment_set_name: String,
    #[arg(long)]
    pub beta: usize,
    #[arg(long)]
    pub read_from_random: usize,
    #[arg(long, conflicts_with = "no_use_random_seed")]
    pub use_random_seed: Option<usize>,
    #[arg(long)]
    pub no_use_random_seed: bool,
    #[arg(long)]
    pub global_config_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let _guard = init_std_out_logging_thread_local();

    info!("Starting simple_dummy_coordinate_adder...");
    let args = InputArgs::parse();

    let global_config: GlobalConfig = serde_yaml::from_reader(
        std::fs::File::open(
            &args.global_config_path,
        )
            .expect("Failed to open global config file"),
    )
        .expect("Failed to read global config");


    let input_file: PathBuf = replace_placeholders_in_path_pattern(
        // first, replace placholders coming from the global config file, such as {original_dir_to_read_from},
        // then replace the placeholders that are specific to this experiment, such as {base_output_dir}, {beta}, {read_from_random}
        &global_config.replace_common_pattern(&args.input_file_pattern).expect("Failed to replace common placeholders in input file pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        args.beta.into(),
        args.read_from_random.into(),
        args.use_random_seed,
    )
        .into();

    info!("Input file: {}", input_file.display());

    let output_file: PathBuf = replace_placeholders_in_path_pattern(
        &global_config.replace_common_pattern(&args.output_file_pattern).expect("Failed to replace common placeholders in output file pattern"),
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        args.beta.into(),
        args.read_from_random.into(),
        args.use_random_seed,
    )
        .into();

    info!("Output file: {}", output_file.display());

    create_dir_all(
        output_file
            .parent()
            .ok_or("Output file has no parent directory")?,
    )?;

    let changed_lines = add_dummy_coordinates_to_file(&input_file, &output_file)?;
    info!(
        "Wrote {} changed event line(s) to {}",
        changed_lines,
        output_file.display()
    );

    Ok(())
}
