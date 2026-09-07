use std::error::Error;
use std::fs::create_dir_all;
use std::path::PathBuf;

use clap::Parser;
use pre_postprocessing::activity_dummy_coordinates::add_dummy_coordinates_to_file;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// Path to input XML events file
    #[arg(long)]
    pub input_file: String,
    #[arg(long)]
    pub output_file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let _guard = init_std_out_logging_thread_local();
    let args = InputArgs::parse();
    let input = PathBuf::from(&args.input_file);
    let output = PathBuf::from(&args.output_file);
    create_dir_all(
        output
            .parent()
            .ok_or("Output file has no parent directory")?,
    )?;

    let changed_lines = add_dummy_coordinates_to_file(&input, &output)?;
    info!(
        "Wrote {} changed event line(s) to {}",
        changed_lines,
        output.display()
    );

    Ok(())
}
