use clap::Parser;
use postprocessing::event_extraction::{LinkToPathMap, TravelTimeAndSumDepPerPathCSVWriter};
use rust_qsim::simulation::events::EventsManager;
use rust_qsim::simulation::events::utils::{read_events, read_partitioned_events};
use rust_qsim::simulation::id;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// path and file name of the input file(s). If num_parts=0, should be everything in front of
    /// the file extension (e.g., "events" for `events.xml`)
    /// if num_parts>0, should be everything in front of the part number and file extension (e.g.,
    /// "events" for `events.0.xml` and `events.1.xml`)
    #[arg(long)]
    pub input_file_stem: String,
    /// file format of the input file(s). Note: is *not* identical to the extension as given by
    /// `path.extension()`, since here, the entire file type such as `xml.gz` is required, while
    /// `extension()` only considers the part after the last dot to be the extension.
    #[arg(long)]
    pub input_file_format: String,
    /// complete output path (including extension) for the travel times csv file to be written
    #[arg(long)]
    pub tt_csv_path: String,
    /// complete output path (including extension) for the summed departures csv file to be written
    #[arg(long)]
    pub sd_csv_path: String,
    /// optional complete path to an id store to be loaded (required when reading proto files)
    #[arg(long)]
    pub id_store_path: Option<String>,
    /// number of partitions, i.e., number of files to be read. Will read files of the format
    /// `{input_file_stem}.{i}.{input_file_format}` for i=0..num_parts. If only a single file is to
    /// be read, that doesn't have the ".{i}" part in the file name, set num_parts=0.
    #[arg(long, default_value_t = 1)]
    pub num_parts: u32,
    /// name for a link to path map, i.e., a map between certain network links to integers, that
    /// represent the paths that are analyzed. Currently, only "braess" is implemented.
    #[arg(long)]
    pub link_to_path_map_name: String,
    /// parameter beta used in the simulation, i.e., reciprocal of the square of the vehicle size.
    /// This is used to divide the summed departures count, to report values in PCU's/PCE's.
    #[arg(long)]
    pub beta: usize,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    info!("Starting event data extractor");

    let args = InputArgs::parse();
    let mut event_mgr = EventsManager::new();

    if let Some(id_store_path) = &args.id_store_path {
        info!("Loading Id Store from path {}", id_store_path);
        id::load_from_file(&PathBuf::from(id_store_path));
    }

    let input_path_stem = PathBuf::from(&args.input_file_stem);
    let tt_output_file_path = PathBuf::from(&args.tt_csv_path);
    let sd_output_file_path = PathBuf::from(&args.sd_csv_path);

    let link_to_path_map = LinkToPathMap::named(args.link_to_path_map_name.as_str())
        .expect("Failed to load link to path map");

    let ttppsd_register_fn = TravelTimeAndSumDepPerPathCSVWriter::register_fn(
        link_to_path_map,
        tt_output_file_path,
        sd_output_file_path,
        args.beta,
    );

    ttppsd_register_fn(&mut event_mgr);
    match args.num_parts {
        0u32 => {
            read_events(
                &mut event_mgr,
                &input_path_stem.join(args.input_file_format),
            )
            .expect("Failed to read events from input file");
        }
        n if n > 0u32 => {
            read_partitioned_events(
                &mut event_mgr,
                &input_path_stem
                    .parent()
                    .expect("Input path stem contains no parent directory"),
                input_path_stem.file_name().unwrap().to_str().unwrap(),
                n,
                &args.input_file_format,
            )
            .expect("Failed to read partitioned events from files");
        }
        _ => {
            unreachable!()
        }
    }

    // read the events from the input file and process them with the events manager, which will
    // publish them to the travel time + summed departures csv writer
    // finishing will trigger the travel time csv writer to write the results to the output files
    event_mgr.finish();

    info!(
        "Event data extractor finished writing event data to files {},\n{}",
        args.tt_csv_path, args.sd_csv_path
    );
}
