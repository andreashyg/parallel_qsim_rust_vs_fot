use clap::Parser;
use experiment_machine::config::GlobalConfig;
use matsim_rust::simulation::events::EventsManager;
use matsim_rust::simulation::events::utils::{read_events, read_partitioned_events};
use matsim_rust::simulation::logging::init_std_out_logging_thread_local;
use pre_postprocessing::avg_travel_time_extraction::AvgTravelTimeExtractor;
use pre_postprocessing::utils::{
    replace_file_name_end_placeholder_in_path_pattern, replace_placeholders_in_path_pattern,
};
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// Path to the output CSV file with travel times over link entry times
    #[arg(
        long,
        help = "Path to the output CSV file with travel times over link entry times"
    )]
    tts_over_le_time_csv_path_pattern: String,
    /// Path to the output CSV file for the average travel time (over all times)
    #[arg(
        long,
        help = "Path to the output CSV file for the average travel time (over all times)"
    )]
    avg_tt_output_path_pattern: String,
    /// Path to the global config
    #[arg(long, help = "Path to the global config")]
    global_config_path: PathBuf,
    /// Input file stem pattern (with placeholders) for the input file
    #[arg(
        long,
        help = "Input file stem pattern (with placeholders) for the input file"
    )]
    input_file_stem_pattern: String,
    /// Base output directory for the experiment set
    #[arg(long, help = "Base output directory for the experiment set")]
    base_output_dir: String,
    /// Experiment set name
    #[arg(long, help = "Experiment set name")]
    experiment_set_name: String,
    /// Beta value for the experiment. Only used for path replacement.
    #[arg(long, help = "Beta value for the experiment.")]
    beta: usize,
    /// read_from_random value for the experiment. Optional, not all experiment types use this.
    /// Only used for path replacement.
    #[arg(
        long,
        help = "read_from_random value for the experiment. Optional, not all experiment types use this"
    )]
    read_from_random: Option<usize>,
    /// The random seed to use for the experiment.Optional, not all experiment types use this.
    /// Only used for path replacement
    #[arg(long, help = "The random seed to use for the experiment.")]
    use_random_seed: Option<usize>,
    /// number of partitions, i.e., number of files to be read. Will read files of the format
    /// `{input_file_stem}.{i}.{input_file_format}` for i=0..num_parts. If only a single file is to
    /// be read, that doesn't have the ".{i}" part in the file name, set num_parts=0
    #[arg(
        long,
        help = "Number of parts for partitioned input files. If 0, the input file is assumed to be a single file, without .{i} part in the file name."
    )]
    num_parts: u32,
    /// file format of the input file(s). Note: is *not* identical to the extension as given by
    /// `path.extension()`, since here, the entire file type such as `xml.gz` is required, while
    /// `extension()` only considers the part after the last dot to be the extension.
    #[arg(
        long,
        help = "Input file format (extension) for the input file(s). For example, 'xml.gz' or 'xml.zst'."
    )]
    input_file_format: String,
    /// bin size for link enter time aggregation. All link enter times will be aggregated into bins
    /// of this size (in seconds), labeled after the bin start time for the average travel time calculation.
    #[arg(
        long,
        help = "Bin size for link enter time aggregation. All link enter times will be aggregated into bins of this size (in seconds), labeled after the bin start time for the average travel time calculation."
    )]
    bin_size: f64,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();

    info!("Starting average travel time extractor.");

    let args = InputArgs::parse();
    let global_config: GlobalConfig = serde_yaml::from_reader(
        std::fs::File::open(&args.global_config_path).expect("Failed to open global config file"),
    )
    .expect("Failed to parse global config file");

    info!(
        "Global config loaded from {}",
        args.global_config_path.display()
    );

    let input_path_stem_pattern = &args.input_file_stem_pattern;

    let input_path_stem: PathBuf = replace_placeholders_in_path_pattern(
        input_path_stem_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        args.beta.into(),
        args.read_from_random.into(),
        args.use_random_seed,
    )
    .into();

    let tts_over_le_time_path_pattern_with_filename_end =
        replace_file_name_end_placeholder_in_path_pattern(
            &args.tts_over_le_time_csv_path_pattern,
            &global_config,
            args.use_random_seed,
        );

    let tts_over_le_time_output_path: PathBuf = replace_placeholders_in_path_pattern(
        &tts_over_le_time_path_pattern_with_filename_end,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        args.beta.into(),
        args.read_from_random.into(),
        args.use_random_seed,
    )
    .into();

    let avg_tt_path_pattern_with_filename_end = replace_file_name_end_placeholder_in_path_pattern(
        &args.avg_tt_output_path_pattern,
        &global_config,
        args.use_random_seed,
    );
    let avg_tt_output_path: PathBuf = replace_placeholders_in_path_pattern(
        &avg_tt_path_pattern_with_filename_end,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        args.beta.into(),
        args.read_from_random,
        args.use_random_seed,
    )
    .into();

    let mut events_mgr = EventsManager::new();

    let avgtt_register_fn = AvgTravelTimeExtractor::register_fn(
        tts_over_le_time_output_path.clone(),
        avg_tt_output_path.clone(),
        Duration::from_secs_f64(args.bin_size),
    );

    // register the average travel time extractor with the events manager, so that it will receive
    // the events and process them
    avgtt_register_fn(&mut events_mgr);

    // read the events from the input file and process them with the events manager, which will
    // publish them to the avg travel time csv writer
    match args.num_parts {
        0u32 => {
            read_events(
                &mut events_mgr,
                input_path_stem.with_added_extension(args.input_file_format),
            )
            .expect("Failed to read events from input file");
        }
        n if n > 0u32 => {
            read_partitioned_events(
                &mut events_mgr,
                input_path_stem
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

    // finishing will trigger the avg travel time csv writer to write the results to the output files
    events_mgr.finish();

    info!(
        "Event data extractor finished writing event data to files {},\n{}",
        tts_over_le_time_output_path.display(),
        avg_tt_output_path.display()
    );
}
