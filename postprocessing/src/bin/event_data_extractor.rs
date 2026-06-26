use clap::Parser;
use rust_qsim::simulation::events::EventsManager;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    #[arg(long)]
    pub path: String,
    #[arg(long)]
    pub id_store: String,
    #[arg(long, default_value_t = 1)]
    pub num_parts: u32,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    info!("Starting event data extractor");

    let args = InputArgs::parse();

    let mut event_mgr = EventsManager::new();

    let output_file_path = PathBuf::from(&args.path).join("some_filename.csv");

    // let register_
}
