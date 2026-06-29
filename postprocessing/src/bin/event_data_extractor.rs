use clap::Parser;
use postprocessing::event_extraction::{LinkToPathMap, TravelTimePerPathCSVWriter};
use rust_qsim::simulation::events::EventsManager;
use rust_qsim::simulation::events::utils::read_events;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use std::path::PathBuf;
use tracing::info;

// TODO do I need the ID store? maybe only for proto? How do I handle this?

#[derive(Parser, Debug)]
struct InputArgs {
    pub input_path: String,
    #[arg(long)]
    pub csv_path: String,
    #[arg(long)]
    pub id_store: String,
    #[arg(long, default_value_t = 1)]
    pub num_parts: u32,
    pub link_to_path_map_name: String,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    info!("Starting event data extractor");

    let args = InputArgs::parse();

    let mut event_mgr = EventsManager::new();

    let input_path = PathBuf::from(&args.input_path);
    let output_file_path = PathBuf::from(&args.csv_path);

    let link_to_path_map = LinkToPathMap::named(args.link_to_path_map_name.as_str())
        .expect("Failed to load link to path map");

    let register_fn = TravelTimePerPathCSVWriter::register_fn(link_to_path_map, output_file_path);

    register_fn(&mut event_mgr);

    // read the events from the input file and process them with the events manager, which will
    // publish them to the travel time csv writer
    read_events(&mut event_mgr, &input_path).expect("Failed to read events from input file");
    // finishing will trigger the travel time csv writer to write the results to the output file
    event_mgr.finish();

    info!(
        "Event data extractor finished writing event data to file {}",
        args.csv_path
    );
}
