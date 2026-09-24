use clap::Parser;
use pre_postprocessing::event_extraction::{LinkToPathMap, TravelTimeAndSumDepPerPathCSVWriter};
use pre_postprocessing::utils::{
    replace_file_name_end_placeholder_in_path_pattern, replace_placeholders_in_path_pattern,
};
use rust_qsim::simulation::events::EventsManager;
use rust_qsim::simulation::events::utils::{read_events, read_partitioned_events};
use rust_qsim::simulation::id;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// base output directory, e.g., "compare_braess_to_java"
    #[arg(long)]
    pub base_output_dir: String,
    /// name of the experiment set, e.g., "varying_rust_seeds"
    #[arg(long)]
    pub experiment_set_name: String,
    /// replanning variant of the original java experiment to use, such as "sel-exp10-switch-at-80"
    #[arg(long)]
    pub replanning_variant: String,
    /// java seed for the random run to read from, e.g., 1-20
    #[arg(long)]
    pub read_from_random: usize,
    /// rust seed that was used in the random run for which the events are to be read, e.g., 42-61
    #[arg(long, conflicts_with = "no_random_seed")]
    pub use_random_seed: Option<usize>,
    /// if set, there is no use_random_seed value to use
    #[arg(long)]
    pub no_random_seed: bool,
    /// path and file name of the input file(s). If num_parts=0, should be everything in front of
    /// the file extension (e.g., "events" for `events.xml`)
    /// if num_parts>0, should be everything in front of the part number and file extension (e.g.,
    /// "events" for `events.0.xml` and `events.1.xml`)
    /// Note: *can* contain placeholders {base_output_dir}, {experiment_set_name},
    /// {replanning_variant}, {beta}, {read_from_random}, {use_random_seed} that will be replaced.
    #[arg(long)]
    pub input_file_stem_pattern: String,
    /// file format of the input file(s). Note: is *not* identical to the extension as given by
    /// `path.extension()`, since here, the entire file type such as `xml.gz` is required, while
    /// `extension()` only considers the part after the last dot to be the extension.
    #[arg(long)]
    pub input_file_format: String,
    /// complete output path (including extension) for the travel times csv file to be written.
    /// Note: *can* contain placeholders {base_output_dir}, {experiment_set_name}, {file_name_end},
    /// {replanning_variant}, {beta}, {read_from_random}, {use_random_seed} that will be replaced.
    #[arg(long)]
    pub tt_csv_path_pattern: String,
    /// complete output path (including extension) for the summed departures csv file to be written
    /// Note: *can* contain placeholders {base_output_dir}, {experiment_set_name}, {file_name_end},
    /// {replanning_variant}, {beta}, {read_from_random}, {use_random_seed} that will be replaced.
    #[arg(long)]
    pub sd_csv_path_pattern: String,
    /// optional complete path to an id store to be loaded (required when reading proto files)
    #[arg(long)]
    pub id_store_path_pattern: Option<String>,
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

    let global_config_path =
        PathBuf::from("./experiments/compare_braess_to_java/experiment_sets/global_config.yaml");
    let global_config: experiment_machine::config::GlobalConfig =
        serde_yaml::from_reader(std::fs::File::open(&global_config_path).unwrap())
            .expect("Failed to read global config");

    info!("Loaded global config from {}", global_config_path.display());

    // New version: if the input file format is binpb, we need to load the id store, which is
    // required for reading the events from the proto files. The path to the id store is read from
    // the global config, and can contain placeholders that are replaced with the actual values.
    if args.input_file_format == "binpb" {
        info!(
            "Input file format is binpb, loading id store is required. Reading from global config to get experiment output path pattern."
        );

        let id_store_path_pattern = PathBuf::from(
            global_config
                .global_parameters
                .get("common_output_from_runs_pattern")
                .expect("Failed to get common_output_from_runs_pattern from global config")
                .as_str()
                .expect("common_output_from_runs_pattern must be a string"),
        )
        .join("output_ids.binpb");
        let id_store_path = replace_placeholders_in_path_pattern(
            id_store_path_pattern.to_str().unwrap(),
            Some(&args.base_output_dir),
            Some(&args.experiment_set_name),
            Some(&args.replanning_variant),
            args.beta.into(),
            args.read_from_random.into(),
            args.use_random_seed,
        );
        info!("Loading Id Store from path {}", id_store_path);
        id::load_from_file(&PathBuf::from(id_store_path));
    }
    // if let Some(id_store_path_pattern) = &args.id_store_path_pattern {
    //     let id_store_path = replace_placeholders_in_path_pattern(
    //         id_store_path_pattern,
    //         Some(&args.base_output_dir),
    //         Some(&args.experiment_set_name),
    //         Some(&args.replanning_variant),
    //         args.beta.into(),
    //         args.read_from_random.into(),
    //         args.use_random_seed.into(),
    //     );
    //     info!("Loading Id Store from path {}", id_store_path);
    //     id::load_from_file(&PathBuf::from(id_store_path));
    // }

    let input_path_stem_pattern = &args.input_file_stem_pattern;

    let input_path_stem: PathBuf = replace_placeholders_in_path_pattern(
        input_path_stem_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant),
        args.beta.into(),
        args.read_from_random.into(),
        args.use_random_seed.into(),
    )
    .into();

    let tt_output_file_path_with_file_name_end = replace_file_name_end_placeholder_in_path_pattern(
        &args.tt_csv_path_pattern,
        &global_config,
        args.use_random_seed,
    );

    let tt_output_file_path: PathBuf = replace_placeholders_in_path_pattern(
        &tt_output_file_path_with_file_name_end,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant),
        args.beta.into(),
        args.read_from_random.into(),
        args.use_random_seed.into(),
    )
    .into();

    // let tt_output_file_path = PathBuf::from(&args.tt_csv_path);

    let sd_output_file_path_with_file_name_end = replace_file_name_end_placeholder_in_path_pattern(
        &args.sd_csv_path_pattern,
        &global_config,
        args.use_random_seed,
    );
    let sd_output_file_path: PathBuf = replace_placeholders_in_path_pattern(
        &sd_output_file_path_with_file_name_end,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant),
        args.beta.into(),
        args.read_from_random.into(),
        args.use_random_seed.into(),
    )
    .into();

    let link_to_path_map = LinkToPathMap::named(args.link_to_path_map_name.as_str())
        .expect("Failed to load link to path map");

    let ttppsd_register_fn = TravelTimeAndSumDepPerPathCSVWriter::register_fn(
        link_to_path_map,
        tt_output_file_path.clone(),
        sd_output_file_path.clone(),
        args.beta,
    );

    ttppsd_register_fn(&mut event_mgr);
    match args.num_parts {
        0u32 => {
            read_events(
                &mut event_mgr,
                &input_path_stem.with_added_extension(args.input_file_format),
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
        tt_output_file_path.display(),
        sd_output_file_path.display()
    );
}
