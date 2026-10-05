use clap::Parser;
use experiment_machine::config::GlobalConfig;
use pre_postprocessing::access_egress_leg_adding::add_access_egress_legs_if_necessary;
use pre_postprocessing::utils::replace_placeholders_in_path_pattern;
use rust_qsim::simulation::logging::init_std_out_logging_thread_local;
use rust_qsim::simulation::scenario::population::Population;
use rust_qsim::simulation::scenario::vehicles::Garage;
use tracing::info;

#[derive(Parser, Debug)]
struct InputArgs {
    /// Base directory for output files.
    #[arg(long)]
    pub base_output_dir: String,
    /// Replanning variant. The short name used in the experiment. Used in the output file name
    #[arg(long)]
    pub replanning_variant: String,
    /// Replanning variant. The long, original directory name. Needed for the input file names.
    #[arg(long)]
    pub replanning_variant_original: String,
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
}

fn main() {
    let _guard = init_std_out_logging_thread_local();
    let args = InputArgs::parse();

    // TODO this is not *really* global, since it is written to compare_braess_to_java.
    //  Decide if it should be moved to the parent folder, to be valid for all experiments, or if
    //  every such folder should have its own, and then pass it around as a parameter.
    let global_config: GlobalConfig = serde_yaml::from_reader(
        std::fs::File::open(
            "./experiments/compare_braess_to_java/experiment_sets/global_config.yaml",
        )
            .expect("Failed to open global config file"),
    )
        .expect("Failed to read global config");

    let pop_to_be_changed_pattern = global_config
        .replace_common_pattern(
            global_config
                .global_parameters
                .get("original_output_pop_pattern")
                .expect("Failed to get original_output_pop_pattern from global config")
                .as_str()
                .expect("original_output_pop_pattern must be a string"),
        )
        .expect("Failed to replace common pattern in original_output_pop_pattern");

    let pop_to_be_changed_path = replace_placeholders_in_path_pattern(
        &pop_to_be_changed_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant_original),
        Some(args.beta),
        Some(args.read_from_random),
        None,
    );

    let output_file_pattern = global_config
        .replace_common_pattern(
            global_config
                .global_parameters
                .get("corrected_output_pop_pattern")
                .expect("Failed to get corrected_output_pop_pattern from global config")
                .as_str()
                .expect("corrected_output_pop_pattern must be a string"),
        )
        .expect("Failed to replace common pattern in corrected_output_pop_pattern");

    let output_file = replace_placeholders_in_path_pattern(
        &output_file_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant),
        Some(args.beta),
        Some(args.read_from_random),
        None,
    );

    let vehicle_file_pattern = global_config
        .replace_common_pattern(
            global_config
                .global_parameters
                .get("vehicle_file_pattern")
                .expect("Failed to get vehicle_file_pattern from global config")
                .as_str()
                .expect("vehicle_file_pattern must be a string"),
        )
        .expect("Failed to replace common pattern in vehicle_file_pattern");

    let vehicle_file = replace_placeholders_in_path_pattern(
        &vehicle_file_pattern,
        Some(&args.base_output_dir),
        Some(&args.experiment_set_name),
        Some(&args.replanning_variant_original),
        Some(args.beta),
        Some(args.read_from_random),
        None,
    );

    let mut garage = Garage::from_file(vehicle_file.as_ref());

    let mut pop_to_be_changed = Population::from_file(&pop_to_be_changed_path, &mut garage);

    // we use hard-coded access egress leg mode "walk" here, since the experiment runner also uses
    // this
    add_access_egress_legs_if_necessary(&mut pop_to_be_changed, "walk");

    pop_to_be_changed.to_file(output_file.as_ref());

    info!(
        "Successfully wrote population with access egress legs added, where necessary, to {}",
        output_file
    );
}
