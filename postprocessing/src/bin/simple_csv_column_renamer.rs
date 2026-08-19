use clap::Parser;
use postprocessing::csv_column_renaming::rename_csv_columns;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// Path to input CSV file
    #[arg(long)]
    pub input_file: String,
    /// Path to output CSV file
    #[arg(long)]
    pub output_file: String,
    /// New column names in order. Provide this argument multiple times, once per column.
    #[arg(long = "column-name", required = true)]
    pub column_names: Vec<String>,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    let args = InputArgs::parse();

    rename_csv_columns(
        &PathBuf::from(args.input_file),
        &PathBuf::from(args.output_file),
        &args.column_names,
    )
    .expect("Failed to rename CSV columns");

    info!("Successfully wrote renamed CSV file");
}
