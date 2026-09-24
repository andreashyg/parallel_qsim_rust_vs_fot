use std::error::Error;
use std::fs::create_dir_all;
use std::path::PathBuf;

use clap::Parser;
use pre_postprocessing::activity_dummy_coordinates::add_dummy_coordinates_to_file;
use pre_postprocessing::utils::replace_placeholders_in_path_pattern;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// Path to input XML events file
    /// Note: *can* contain placeholders {base_output_dir}, {replanning_variant}, {beta},
    /// {read_from_random} that will be replaced.
    #[arg(long)]
    pub input_file_pattern: String,
    #[arg(long)]
    pub output_file_pattern: String,
    #[arg(long)]
    pub base_output_dir: String,
    #[arg(long)]
    pub replanning_variant: String,
    #[arg(long)]
    pub experiment_set_name: String,
    #[arg(long)]
    pub beta: usize,
    #[arg(long)]
    pub read_from_random: usize,
    #[arg(long, conflicts_with = "no_random_seed")]
    pub use_random_seed: Option<usize>,
    #[arg(long)]
    pub no_random_seed: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let _guard = init_std_out_logging_thread_local();

    println!("Starting simple_dummy_coordinate_adder...");
    let args = InputArgs::parse();

    let input_file: PathBuf = replace_placeholders_in_path_pattern(
        &args.input_file_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant),
        args.beta.into(),
        args.read_from_random.into(),
        args.use_random_seed,
    )
    .into();

    println!("Input file: {}", input_file.display());

    let output_file: PathBuf = replace_placeholders_in_path_pattern(
        &args.output_file_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant),
        args.beta.into(),
        args.read_from_random.into(),
        args.use_random_seed,
    )
    .into();

    println!("Output file: {}", output_file.display());

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
