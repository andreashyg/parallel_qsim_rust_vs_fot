use clap::Parser;
use pre_postprocessing::csv_column_renaming::rename_csv_columns;
use pre_postprocessing::utils::replace_placeholders_in_path_pattern;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// Path to input CSV file.
    /// This can be a path pattern with placeholders like {base_output_dir}, {experiment_set_name}, {replanning_variant}, {beta} and {read_from_random}.
    #[arg(long)]
    pub input_file_pattern: String,
    /// Path to output CSV file
    /// This can be a path pattern with placeholders like {base_output_dir}, {experiment_set_name}, {replanning_variant}, {beta} and {read_from_random}.
    #[arg(long)]
    pub output_file_pattern: String,
    /// Base directory for output files.
    #[arg(long)]
    pub base_output_dir: String,
    /// Replanning variant.
    #[arg(long)]
    pub replanning_variant: String,
    /// Name of the experiment set.
    #[arg(long)]
    pub experiment_set_name: String,
    /// Beta value.
    #[arg(long)]
    pub beta: usize,
    /// Java random seed used for reading the input file.
    /// This is used to replace the {read_from_random} placeholder in the input/output file patterns
    #[arg(long)]
    pub read_from_random: usize,
    /// New column names in order. Provide this argument multiple times, once per column.
    #[arg(long = "column-name", required = true)]
    pub column_names: Vec<String>,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    let args = InputArgs::parse();

    let input_file = replace_placeholders_in_path_pattern(
        &args.input_file_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant),
        Some(args.beta),
        Some(args.read_from_random),
        None,
    );
    let output_file = replace_placeholders_in_path_pattern(
        &args.output_file_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant),
        Some(args.beta),
        Some(args.read_from_random),
        None,
    );

    rename_csv_columns(
        &PathBuf::from(input_file),
        &PathBuf::from(output_file),
        &args.column_names,
    )
    .expect("Failed to rename CSV columns");

    info!("Successfully wrote renamed CSV file");
}
