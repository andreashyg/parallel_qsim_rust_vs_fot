use clap::Parser;
use experiment_machine::config::GlobalConfig;
use matsim_rust::simulation::logging::init_std_out_logging_thread_local;
use matsim_rust::simulation::scenario::population::Population;
use matsim_rust::simulation::scenario::vehicles::Garage;
use pre_postprocessing::access_egress_leg_adding::add_access_egress_legs_if_necessary;
use pre_postprocessing::activity_time_replacement::replace_activity_times_and_add_dummy_coords_to_acts;
use pre_postprocessing::utils::replace_placeholders_in_path_pattern;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// Base directory for output files.
    #[arg(long)]
    pub base_output_dir: String,
    // /// Replanning variant. The short name used in the experiment. Used in the output file name
    // #[arg(long)]
    // pub replanning_variant: String,
    // /// Replanning variant. The long, original directory name. Needed for the input file names.
    // #[arg(long)]
    // pub replanning_variant_original: String,
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
    #[arg(long)]
    pub skip_if_existing: bool,
    /// global config file path
    #[arg(long)]
    pub global_config_path: String,
    /// Pattern for the population to be changed.
    #[arg(long)]
    pub pop_to_be_changed_pattern: String,
    /// Pattern for the population with correct activity times. If not provided, no activity times will be replaced.
    #[arg(long)]
    pub pop_with_correct_activity_times_pattern: Option<String>,
    /// Pattern for the vehicle file.
    #[arg(long)]
    pub vehicle_file_pattern: String,
    /// Option to add access/egress legs if necessary. If not provided, no access/egress legs will be added.
    #[arg(long)]
    pub add_access_egress_legs_if_necessary: bool,
    /// pattern for the output file.
    #[arg(long)]
    pub output_file_pattern: String,
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    let args = InputArgs::parse();

    let global_config: GlobalConfig = serde_yaml::from_reader(
        std::fs::File::open(
            &args.global_config_path,
        )
            .expect("Failed to open global config file"),
    )
        .expect("Failed to read global config");

    let pop_to_be_changed_pattern = global_config
        .replace_common_pattern(
            &args.pop_to_be_changed_pattern
            // global_config
            //     .global_parameters
            //     .get("original_output_pop_pattern")
            //     .expect("Failed to get original_output_pop_pattern from global config")
            //     .as_str()
            //     .expect("original_output_pop_pattern must be a string"),
        )
        .expect("Failed to replace common pattern in pop_to_be_changed_pattern");

    let pop_to_be_changed_path = replace_placeholders_in_path_pattern(
        &pop_to_be_changed_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(args.beta),
        Some(args.read_from_random),
        None,
    );

    // If pop_with_correct_activity_times_pattern is given, replace common patterns in it.
    let pop_with_correct_times_pattern = &args.pop_with_correct_activity_times_pattern.map(
        |pat|
            global_config
                .replace_common_pattern(
                    &pat
                    // global_config
                    //     .global_parameters
                    //     .get("pop_with_correct_activity_times_pattern")
                    //     .expect("Failed to get pop_with_correct_activity_times_pattern from global config")
                    //     .as_str()
                    //     .expect("pop_with_correct_activity_times_pattern must be a string"),
                )
                .expect("Failed to replace common pattern in pop_with_correct_activity_times_pattern")
    );

    // then also replace standard placeholders
    let pop_with_correct_times_path = pop_with_correct_times_pattern.as_ref().map(
        |pat|
            replace_placeholders_in_path_pattern(
                pat,
                Some(&args.base_output_dir),
                Some(&args.experiment_set_name),
                Some(args.beta),
                Some(args.read_from_random),
                None,
            )
    );

    // the output file of this module will likely be the same pattern used as
    // "pop_to_use_as_input_pattern" in the next module.
    // However, it is no longer a "global" parameter in the global config, for more flexibility,
    // and since this module is no longer needed in the normal experiments anyway.
    let output_file_pattern = global_config
        .replace_common_pattern(
            &args.output_file_pattern
        )
        .expect("Failed to replace common pattern in output_file_pattern");

    let output_file = replace_placeholders_in_path_pattern(
        &output_file_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(args.beta),
        Some(args.read_from_random),
        None,
    );

    let vehicle_file_pattern = global_config
        .replace_common_pattern(
            &args.vehicle_file_pattern
        )
        .expect("Failed to replace common pattern in vehicle_file_pattern");

    let vehicle_file = replace_placeholders_in_path_pattern(
        &vehicle_file_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(args.beta),
        Some(args.read_from_random),
        None,
    );

    if args.skip_if_existing && std::fs::exists(&output_file).expect("Failed to check if output file exists") {
        info!(
            "Output file {} already exists, skipping population preparation since --skip-if-existing is specified.",
            output_file
        );
        return;
    }

    let mut garage = Garage::from_file(vehicle_file.as_ref());

    let mut pop_to_be_changed = Population::from_file(&pop_to_be_changed_path, &mut garage);

    // if a population with correct activity times is provided, replace the activity times in the
    // population to be changed with the correct ones
    if let Some(pop_with_correct_times) = pop_with_correct_times_path
        .map(|path| Population::from_file(&path, &mut garage)) {
        info!("Replacing activity times in population");
        replace_activity_times_and_add_dummy_coords_to_acts(
            &mut pop_to_be_changed,
            pop_with_correct_times,
        );
        info!("Successfully replaced activity times in population");
    }

    if args.add_access_egress_legs_if_necessary {
        info!("Adding access/egress legs to population where necessary");
        // we use hard-coded access egress leg mode "walk" here, since the experiment runner also
        // uses this
        add_access_egress_legs_if_necessary(&mut pop_to_be_changed, "walk");
        info!("Successfully added access/egress legs to population where necessary");
    }

    info!("Writing prepared population to {}", output_file);
    pop_to_be_changed.to_file(output_file.as_ref());

    info!("Successfully wrote prepared population to {}", output_file);
}
