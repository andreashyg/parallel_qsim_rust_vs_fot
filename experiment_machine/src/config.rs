use crate::bash_utils;
use crate::bash_utils::BashFunction;
use rust_qsim::simulation::config::OverwriteFiles;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub global_parameters: HashMap<String, Value>,
}

impl GlobalConfig {
    pub fn get_global_config_parameter<'a>(&self, key: &str) -> Result<&Value, String> {
        if let Some(value) = self.global_parameters.get(key) {
            return Ok(value);
        }
        Err(format!(
            "Global parameter '{}' not found in global config.",
            key
        ))
    }

    pub fn replace_common_pattern(&self, pattern: &str) -> Result<String, String> {
        if pattern.contains("{common_output_from_runs_pattern}") {
            let actual_pattern = self
                .get_global_config_parameter("common_output_from_runs_pattern")?
                .as_str()
                .ok_or_else(|| "common_output_from_runs_pattern must be a string".to_string())?;

            // Replace the pattern with the actual common output path
            Ok(pattern.replace("{common_output_from_runs_pattern}", actual_pattern))
        } else if pattern.contains("{common_extracted_data_pattern}") {
            let actual_pattern = self
                .get_global_config_parameter("common_extracted_data_pattern")?
                .as_str()
                .ok_or_else(|| "common_extracted_data_pattern must be a string".to_string())?;

            // Replace the pattern with the actual common input path
            Ok(pattern.replace("{common_extracted_data_pattern}", actual_pattern))
        } else if pattern.contains("{common_plots_pattern}") {
            let actual_pattern = self
                .get_global_config_parameter("common_plots_pattern")?
                .as_str()
                .ok_or_else(|| "common_plots_pattern must be a string".to_string())?;

            // Replace the pattern with the actual common input path
            Ok(pattern.replace("{common_plots_pattern}", actual_pattern))
        } else if pattern.contains("{original_dir_to_read_from}") {
            let actual_pattern = self
                .get_global_config_parameter("original_dir_to_read_from")?
                .as_str()
                .ok_or_else(|| "original_dir_to_read_from must be a string".to_string())?;
            Ok(pattern.replace("{original_dir_to_read_from}", actual_pattern))
        } else {
            // If no known pattern is found, return the original pattern
            Ok(pattern.to_string())
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    globals: Option<HashMap<String, Value>>,
    param_sweep: HashMap<String, Vec<Value>>,
    modules: Vec<Box<dyn Module>>,
}

impl Config {
    pub fn get_expset_config_globals_parameter(
        &self,
        key: &str,
    ) -> Result<&Value, String> {
        // if key exists in (per experiment set) globals, return that value
        if let Some(globals) = &self.globals {
            if let Some(value) = globals.get(key) {
                return Ok(value);
            }
        }
        Err(format!("Global parameter '{}' not found in experiment set config.", key))
    }

    pub(crate) fn get_expset_config_param_sweep_parameter(
        &self,
        key: &str,
    ) -> Result<&Vec<Value>, String> {
        self.param_sweep
            .get(key)
            .ok_or_else(|| format!("Missing param_sweep key: {key}"))
    }

    /// Generates a bash array declaration for a given key in config.param_sweep
    pub(crate) fn get_sweep_array_decl(&self, key: &str, var_name: &str) -> Result<String, String> {
        let values = self.get_expset_config_param_sweep_parameter(key)?;

        if values.is_empty() {
            return Err(format!("param_sweep.{key} must not be empty"));
        }

        let atoms: Result<Vec<_>, _> = values
            .iter()
            .map(bash_utils::yaml_value_as_shell_atom)
            .collect();
        Ok(format!("{var_name}=({})", atoms?.join(" ")))
    }

    pub fn run_all_modules(
        &self,
        global_config: &GlobalConfig,
        resume: bool,
        resume_failed: bool,
    ) -> Result<(), String> {
        // Run all modules.
        for module in &self.modules {
            module.run(self, global_config, resume, resume_failed)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SeedsToAvgOver {
    UseRandomSeeds,
    ReadFromRandomSeedIndices,
}

impl SeedsToAvgOver {
    /// Get the name of the param sweep field containing the seeds to avg over/use.
    fn get_seeds_to_use_field_name(&self) -> &str {
        match self {
            SeedsToAvgOver::UseRandomSeeds => { "use_random_seeds" }
            SeedsToAvgOver::ReadFromRandomSeedIndices => { "read_from_random_seed_indices" }
        }
    }

    /// Get the seeds written in the param_sweep field in the config as a space separated string.
    /// Specifically, the seeds written in the field corresponding to the variant of this enum.
    fn get_seeds_to_use_as_str(&self, config: &Config) -> Result<String, String> {
        // this is the name of the param sweep values that will be read and then passed as an extra
        // argument (so that a module can, e.g., average over all those seeds itself)
        let seeds_to_use_field_name = self.get_seeds_to_use_field_name();

        // get the seeds to use, which will be passed as an extra argument
        let seeds_to_use_atoms: Result<Vec<_>, _> = config
            .param_sweep
            .get(seeds_to_use_field_name)
            .ok_or(format!(
                "{} must be set in param_sweep for PlotTtAndSdAvgdOverSeeds module when averaging over use random seeds",
                seeds_to_use_field_name))?
            .iter()
            .map(bash_utils::yaml_value_as_shell_atom)
            .collect();

        Ok(seeds_to_use_atoms?.join(" "))
    }

    /// Get an overwrites HashMap to replace the param sweep field for the seeds to avg over/use
    /// with a single value (given as an argument to this function).
    /// Specifically, it looks like this: {use_random_seeds: use_random_seeds=(avg_over_all)} or
    /// similar for read_from_random.
    /// Motivation: The param sweep seeds are passed as an extra argument, for example,
    /// since a module might average over all those seeds itself. Therefore, we overwrite that
    /// param sweep field with a single value like "avg_over_all" or "use_all", so that the
    /// module is only called once per cartesian product of the other parameters.
    fn get_param_sweep_overwrites(&self, name_for_param_sweep_overwrite: &str) -> HashMap<String, String> {
        // this is the name of the param sweep values that will be replaced
        // by a value such as "avg_over_all" or "use_all" in the actual param sweep
        let seeds_to_use_field_name = self.get_seeds_to_use_field_name();

        // as the seeds to use are passed as an extra argument (for example, since a
        // module might average over all those seeds itself), that module should only be
        // called once per cartesian product of the other parameters. Therefore, we overwrite
        // the param_sweep field for those seeds to a single value like "avg_over_all" or
        // "use_all".
        let seeds_to_use_decl_overwrite = format!(
            "{}=({})",  // overwrite the param sweep field for the seeds that are passed extra with smth like "avg_over_all"
            seeds_to_use_field_name,
            bash_utils::yaml_value_as_shell_atom(&Value::String(
                name_for_param_sweep_overwrite.to_string()
            )).expect("Failed to convert name_for_param_sweep_overwrite to shell atom")
        );
        let overwrites = HashMap::from_iter([(
            seeds_to_use_field_name.to_string(),
            seeds_to_use_decl_overwrite,
        )]);

        overwrites
    }
}

#[typetag::serde(tag = "type")]
pub trait Module: Debug {
    /// Run the module, given the config and global_config.
    /// The default implementation is to run the bash function for the module in parallel for all
    /// combinations of the parameter sweep. This will respect the resume and resume_failed flags,
    /// and will also pass any extra string arguments and param sweep overwrites defined by the
    /// module in the corresponding trait methods.
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        resume: bool,
        resume_failed: bool,
    ) -> Result<(), String> {
        // The function to be run
        let bash_function = self.get_bash_function();

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            resume,
            resume_failed,
            self.get_extra_str_args(config, global_config)?, // is None by default
            self.get_param_sweep_overwrites(),  // also None by default
        )
    }

    /// Get the bash function that should be run for this module.
    fn get_bash_function(&self) -> BashFunction;

    /// get optional extra string arguments to pass to the bash function, which are not part of the
    /// cartesian product of the parameter sweep
    fn get_extra_str_args(&self, _config: &Config, _global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        Ok(None)
    }

    /// Modules can overwrite param sweep overwrite values. For example, a module that only concerns
    /// the input data might not depend on use_random_seed, or a plotting function will be called
    /// only once but iterate through all use_random_seeds itself. Such modules will then overwrite
    /// the param sweep value for use_random_seed to be a single value (e.g. None or avg_over_all),
    /// so that the bash function is only called once per cartesian product of the other parameters.
    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        None
    }

    /// Get a (expset) global parameter, or an overwrite if it exists.
    /// Does *not* read the global config, but just the global parameters of the experiment set
    /// specific config.
    fn get_expset_config_global_parameter_or_overwrite(
        &self,
        config: &Config,
        param_name: &str,
    ) -> Result<Value, String> {
        if let Some(overwrites) = &self.get_expset_config_overwrites() {
            if let Some(value) = overwrites.get(param_name) {
                return Ok(value.clone());
            }
        }
        config
            .get_expset_config_globals_parameter(param_name)
            .cloned()
    }

    /// Some modules may have the possibility to get overwrites for global parameters. The modules
    /// need to implement where they get them from (typically, a field self.overwrites).
    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>>;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunRustBasedOnJavaOutput {
    pub overwrite_mode: OverwriteFiles,
    pub overwrites: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunJavaSingleIterBasedOnJavaOutput {
    pub overwrite_mode: OverwriteFiles,
    pub overwrites: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractMeasurementsFromEvents {
    pub input_file_stem_pattern: Option<String>,
    pub input_file_format: Option<String>,
    // overwrite global parameters by putting them in this hashmap
    pub overwrites: Option<HashMap<String, Value>>,
    pub num_parts: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddDummyCoordinatesToEvents {
    pub input_file_pattern: Option<String>,
    pub output_file_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
}

/// This module is used to prepare the original population for the experiments. It replaces the
/// activity times in the population with times from separate population files, and it adds
/// access/egress legs before and after the main leg.
#[derive(Debug, Deserialize, Serialize)]
pub struct PrepareOriginalPopulation {
    pub overwrites: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddAccessEgressLegsToPopulation {
    pub overwrites: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdPerSeed {
    pub overwrites: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDiffPerSeed {
    pub overwrites: Option<HashMap<String, Value>>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub minus_what: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdAvgdOverSeeds {
    pub overwrites: Option<HashMap<String, Value>>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdAvgdOverSeedsDiff {
    pub overwrites: Option<HashMap<String, Value>>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
    pub minus_what: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToNashBoxplotsOverBeta {
    pub overwrites: Option<HashMap<String, Value>>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToNashScatterplotsOverBeta {
    pub overwrites: Option<HashMap<String, Value>>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta {
    pub overwrites: Option<HashMap<String, Value>>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
    pub which_deviation: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta {
    pub overwrites: Option<HashMap<String, Value>>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
    pub which_deviation: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReformatOriginalExtractedMeasurements {
    pub original_tt_tsv_file_pattern: Option<String>,
    pub original_sd_tsv_file_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
}

fn get_delete_output_dir_if_existing_extra_str_arg(overwrite_mode: OverwriteFiles) -> Option<Vec<String>> {
    let delete_output_dir_if_existing = match overwrite_mode {
        OverwriteFiles::DeleteDirectoryIfExists => "true".to_string(),
        _ => "false".to_string(),
    };
    Some(vec![delete_output_dir_if_existing])
}

fn get_betas_to_use_as_space_sep_str(config: &Config) -> Result<String, String> {
    Ok(config.param_sweep.get("betas")
        .ok_or("betas must be set in param_sweep for PlotTtAndSdDeviationScatterplotsOverBeta module")?
        .iter()
        .map(bash_utils::yaml_value_as_shell_atom)
        .collect::<Result<Vec<_>, _>>()?.join(" "))
}

/// helper function to get the tt and sd csv path pattern from the module's overwrites if existing,
/// otherwise from the *experiment set specific* config.
/// Will replace placeholders {common_output_from_runs_pattern}, {common_extracted_data_pattern},
/// {common_plots_pattern} and {original_dir_to_read_from} in the path patterns, by reading those
/// from the *global* config.
fn get_tt_and_sd_csv_path_patterns(
    config: &Config,
    global_config: &GlobalConfig,
    module: &dyn Module,
) -> Result<(String, String), String> {
    // gets the tt_csv_path_pattern and sd_csv_path_pattern from the expset-configs globals, or
    // from the module's overwrites, if existing.
    // Then, replaces the placeholders {common_output_from_runs_pattern}, {common_extracted_data_pattern},
    // {common_plots_pattern} and {original_dir_to_read_from} in the path patterns, by reading those
    // from the *global* config.
    let tt_csv_output_path_pattern = global_config.replace_common_pattern(
        module
            .get_expset_config_global_parameter_or_overwrite(config, "tt_csv_path_pattern")?
            .as_str()
            .ok_or("tt_csv_path_pattern must be a string")?,
    )?;
    let sd_csv_output_path_pattern = global_config.replace_common_pattern(
        module
            .get_expset_config_global_parameter_or_overwrite(config, "sd_csv_path_pattern")?
            .as_str()
            .ok_or("sd_csv_path_pattern must be a string")?,
    )?;

    Ok((tt_csv_output_path_pattern, sd_csv_output_path_pattern))
}

#[typetag::serde]
impl Module for RunRustBasedOnJavaOutput {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "run_experiment_case",
            "./experiments/compare_braess_to_java/runnable_modules/run_rust_based_on_java_output.sh",
            "rust",
        )
    }
    fn get_extra_str_args(&self, _config: &Config, _global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        Ok(get_delete_output_dir_if_existing_extra_str_arg(self.overwrite_mode))
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for RunJavaSingleIterBasedOnJavaOutput {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "run_java_single_iter_case",
            "./experiments/compare_braess_to_java/runnable_modules/run_java_single_iter_based_on_java_output.sh",
            "java",
        )
    }

    fn get_extra_str_args(&self, _config: &Config, _global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        Ok(get_delete_output_dir_if_existing_extra_str_arg(self.overwrite_mode))
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for ExtractMeasurementsFromEvents {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "extract_travel_time_sum_dep_case",
            "./experiments/compare_braess_to_java/runnable_modules/extract_tt_and_sd_from_events.sh",
            "rust",
        )
    }

    /// The extra string arguments for this module are:
    /// 1. `input_file_stem_pattern` (read from module field)
    /// 2. `input_file_format` (read from module field)
    /// 3. `tt_csv_output_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 4. `sd_csv_output_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 5. `num_parts` (read from module field)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        // these are the csv files to which the measurements will be written.
        // Is read from the expset-configs globals, or from the module's overwrites, if existing.
        let (tt_csv_output_path_pattern, sd_csv_output_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;

        // the input file stem pattern is used to find the input files.
        let input_file_stem_pattern = global_config.replace_common_pattern(
            self.input_file_stem_pattern
                .as_ref()
                .ok_or("input_file_stem_pattern must be set")?,
        )?;

        Ok(Some(vec![
            input_file_stem_pattern,
            self.input_file_format
                .clone()
                .ok_or("input_file_format must be set")?,
            tt_csv_output_path_pattern,
            sd_csv_output_path_pattern,
            self.num_parts.ok_or("num_parts must be set")?.to_string(),
        ]))
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for AddDummyCoordinatesToEvents {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "add_dummy_coordinates_case",
            "./experiments/compare_braess_to_java/runnable_modules/add_dummy_coordinates_to_events.sh",
            "rust",
        )
    }

    /// The extra string arguments for this module are:
    /// 1. `input_file_stem_pattern` (read from module field)
    /// 2. `output_file_stem_pattern` (read from module field)
    fn get_extra_str_args(&self, _config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        // replace common patterns in the input and output file stem patterns, using the global config
        let input_file_stem_pattern = global_config.replace_common_pattern(
            self.input_file_pattern
                .as_ref()
                .ok_or("input_file_stem_pattern must be set")?,
        )?;

        let output_file_stem_pattern = global_config.replace_common_pattern(
            self.output_file_pattern
                .as_ref()
                .ok_or("output_file_stem_pattern must be set")?,
        )?;
        Ok(Some(vec![input_file_stem_pattern, output_file_stem_pattern]))
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PrepareOriginalPopulation {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "prepare_original_population_case",
            "./experiments/compare_braess_to_java/runnable_modules/prepare_original_population.sh",
            "rust",
        )
    }

    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        // this module doesn't use any rust seeds, because it only edits the input population.
        // So we overwrite the rust_seeds variable to contain only one parameter, so that the bash
        // function is only called once per cartesian product of the other parameters
        let rust_seed_decl_overwrite = "rust_seeds=(None)".to_string();
        Some(HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite.clone())]))
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for AddAccessEgressLegsToPopulation {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "add_access_egress_legs_case",
            "./experiments/compare_braess_to_java/runnable_modules/add_access_egress_legs_to_population.sh",
            "rust",
        )
    }
    
    /// The param sweep overwrites for this module are:
    /// 1. `rust_seeds` is overwritten to contain only one parameter, so that the bash function is
    ///     only called once per cartesian product of the other parameters.
    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        // this module doesn't use any rust seeds, because it is only editing the input population.
        // So we overwrite the rust_seeds variable to contain only one parameter, so that the bash
        // function is only called once per cartesian product of the other parameters
        let rust_seed_decl_overwrite = "rust_seeds=(None)".to_string();
        let overwrites =
            HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite.clone())]);
        Some(overwrites)
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdPerSeed {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "plot_per_seed_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_per_seed.sh",
            "python",
        )
    }

    /// The extra string arguments for this module are:
    /// 1. `tt_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 2. `sd_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;
        Ok(Some(vec![tt_csv_input_path_pattern, sd_csv_input_path_pattern]))
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDiffPerSeed {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "plot_diff_per_seed_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_diff_per_seed.sh",
            "python",
        )
    }

    /// The extra string arguments for this module are:
    /// 1. `tt_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 2. `sd_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 3. `secondary_input_tt_csv_path_pattern` (read from module field)
    /// 4. `secondary_input_sd_csv_path_pattern` (read from module field)
    /// 5. `minus_what` (read from module field)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;

        let secondary_input_tt_csv_path_pattern = global_config.replace_common_pattern(
            self.secondary_input_tt_csv_path_pattern
                .as_ref()
                .ok_or("secondary_input_tt_csv_path_pattern must be set")?,
        )?;
        let secondary_input_sd_csv_path_pattern = global_config.replace_common_pattern(
            self.secondary_input_sd_csv_path_pattern
                .as_ref()
                .ok_or("secondary_input_sd_csv_path_pattern must be set")?,
        )?;
        Ok(Some(vec![
            tt_csv_input_path_pattern,
            sd_csv_input_path_pattern,
            secondary_input_tt_csv_path_pattern,
            secondary_input_sd_csv_path_pattern,
            self.minus_what.clone(),
        ]))
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdAvgdOverSeeds {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "plot_avgd_over_seeds_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avgd_over_seeds.sh",
            "python",
        )
    }

    /// The extra string arguments for this module are:
    /// 1. `tt_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 2. `sd_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 3. `seeds_to_use_as_space_sep_str` (read from expset-config param sweep,
    ///     based on the module's `seeds_to_avg_over` field, and converted to a space-separated string)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;
        let seeds_to_use_as_space_sep_str = self.seeds_to_avg_over.get_seeds_to_use_as_str(config)?;
        Ok(Some(vec![
            tt_csv_input_path_pattern,
            sd_csv_input_path_pattern,
            seeds_to_use_as_space_sep_str,
        ]))
    }

    /// The param sweep overwrites for this module are:
    /// 1. Either `use_random_seeds` or `read_from_random_seed_indices` will be overwritten to
    ///     contain only the value "avg_over_all", since the plotting function will average over
    ///     all those seeds itself. The seeds that are overwritten are those that are specified in
    ///     the module's `seeds_to_avg_over` field.
    ///     Note: the get_extra_str_args method provides those seeds as a space-separated string to
    ///     the bash function, so that the plotting fct knows which seeds to average over.s
    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        Some(self.seeds_to_avg_over.get_param_sweep_overwrites("avg_over_all"))
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdAvgdOverSeedsDiff {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "plot_avgd_over_seeds_diff_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avgd_over_seeds_diff.sh",
            "python",
        )
    }

    /// The extra string arguments for this module are:
    /// 1. `tt_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 2. `sd_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 3. `secondary_input_tt_csv_path_pattern` (read from module field)
    /// 4. `secondary_input_sd_csv_path_pattern` (read from module field)
    /// 5. `seeds_to_use_as_space_sep_str` (read from expset-config param sweep,
    ///     based on the module's `seeds_to_avg_over` field, and converted to a space-separated string)
    /// 6. `minus_what` (read from module field)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;

        let secondary_input_tt_csv_path_pattern = global_config.replace_common_pattern(
            self.secondary_input_tt_csv_path_pattern
                .as_ref()
                .ok_or("secondary_input_tt_csv_path_pattern must be set")?,
        )?;
        let secondary_input_sd_csv_path_pattern = global_config.replace_common_pattern(
            self.secondary_input_sd_csv_path_pattern
                .as_ref()
                .ok_or("secondary_input_sd_csv_path_pattern must be set")?,
        )?;

        let seeds_to_use_as_space_sep_str = self.seeds_to_avg_over.get_seeds_to_use_as_str(config)?;

        Ok(Some(vec![
            tt_csv_input_path_pattern,
            sd_csv_input_path_pattern,
            secondary_input_tt_csv_path_pattern,
            secondary_input_sd_csv_path_pattern,
            seeds_to_use_as_space_sep_str,
            self.minus_what.clone(),
        ]))
    }

    /// The param sweep overwrites for this module are:
    /// 1. Either `use_random_seeds` or `read_from_random_seed_indices` will be overwritten to
    ///     contain only the value "avg_over_all", since the plotting function will average over
    ///     all those seeds itself. The seeds that are overwritten are those that are specified in
    ///     the module's `seeds_to_avg_over` field.
    ///     Note: the get_extra_str_args method provides those seeds as a space-separated string to
    ///     the bash function, so that the plotting fct knows which seeds to average over.
    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        Some(self.seeds_to_avg_over.get_param_sweep_overwrites("avg_over_all"))
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToNashBoxplotsOverBeta {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "plot_deviation_boxplots_over_beta_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avg_deviations_to_nash_over_beta.sh",
            "python",
        )
    }

    /// The extra string arguments for this module are:
    /// 1. `tt_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 2. `sd_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 3. `seeds_to_use_as_space_sep_str` (read from expset-config param sweep,
    ///     based on the module's `seeds_to_avg_over` field, and converted to a space-separated string)
    /// 4. `betas_to_use_as_space_sep_str` (read from expset-config param sweep, and converted to a space-separated string)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;
        let seeds_to_use_as_space_sep_str = self.seeds_to_avg_over.get_seeds_to_use_as_str(config)?;
        let betas_to_use_as_space_sep_str = get_betas_to_use_as_space_sep_str(config)?;
        Ok(Some(vec![
            tt_csv_input_path_pattern,
            sd_csv_input_path_pattern,
            seeds_to_use_as_space_sep_str,
            betas_to_use_as_space_sep_str,
        ]))
    }

    /// The param sweep overwrites for this module are:
    /// 1. Either `use_random_seeds` or `read_from_random_seed_indices` will be overwritten to
    ///     contain only the value "use_all", since the plotting function will use all those
    ///     seeds itself. The seeds that are overwritten are those that are specified in the
    ///     module's `seeds_to_avg_over` field.
    /// 2. The `betas` parameter will be overwritten to contain only the value "use_all", since
    ///     the plotting function will use all those betas itself.
    /// Motivation: the plotting function will use all betas and seeds_to_avg_over itself, so the
    /// bash function should only be called once per cartesian product of the other parameters.
    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        let mut overwrites = self.seeds_to_avg_over.get_param_sweep_overwrites("use_all");
        // each plot contains all betas, so replace the beta array by a single value "use_all"
        // (it will never be read, but the important part is that it is only one value)
        overwrites.insert("betas".to_string(), "betas=(use_all)".to_string());
        Some(overwrites)
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToNashScatterplotsOverBeta {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "plot_deviation_scatterplots_over_beta_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avg_deviations_to_nash_over_beta.sh",
            "python",
        )
    }

    /// The extra string arguments for this module are:
    /// 1. `tt_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 2. `sd_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 3. `seeds_to_use_as_space_sep_str` (read from expset-config param sweep,
    ///     based on the module's `seeds_to_avg_over` field, and converted to a space-separated string)
    /// 4. `betas_to_use_as_space_sep_str` (read from expset-config param sweep, and converted to a space-separated string)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;
        let seeds_to_use_as_space_sep_str = self.seeds_to_avg_over.get_seeds_to_use_as_str(config)?;
        let betas_to_use_as_space_sep_str = get_betas_to_use_as_space_sep_str(config)?;
        Ok(Some(vec![
            tt_csv_input_path_pattern,
            sd_csv_input_path_pattern,
            seeds_to_use_as_space_sep_str,
            betas_to_use_as_space_sep_str,
        ]))
    }

    /// The param sweep overwrites for this module are:
    /// 1. Either `use_random_seeds` or `read_from_random_seed_indices` will be overwritten to
    ///     contain only the value "avg_over_all", since the plotting function will use all those
    ///     seeds itself. The seeds that are overwritten are those that are specified in the
    ///     module's `seeds_to_avg_over` field.
    /// 2. The `betas` parameter will be overwritten to contain only the value "use_all", since
    ///     the plotting function will use all those betas itself.
    /// Motivation: the plotting function uses all betas and seeds_to_avg_over itself, so the bash
    /// function should only be called once per cartesian product of the other parameters.
    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        let mut overwrites = self.seeds_to_avg_over.get_param_sweep_overwrites("avg_over_all");
        // each plot contains all betas, so replace the beta array by a single value "use_all"
        // (it will never be read, but the important part is that it is only one value)
        overwrites.insert("betas".to_string(), "betas=(use_all)".to_string());
        Some(overwrites)
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "plot_deviation_boxplots_to_diff_run_over_beta_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avg_deviations_to_diff_run_over_beta.sh",
            "python",
        )
    }

    /// The extra string arguments for this module are:
    /// 1. `tt_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 2. `sd_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 3. `secondary_input_tt_csv_path_pattern` (read from module field)
    /// 4. `secondary_input_sd_csv_path_pattern` (read from module field)
    /// 5. `seeds_to_use_as_space_sep_str` (read from expset-config param sweep,
    ///     based on the module's `seeds_to_avg_over` field, and converted to a space-separated string)
    /// 6. `betas_to_use_as_space_sep_str` (read from expset-config param sweep, and converted to a space-separated string)
    /// 7. `which_deviation` (read from module field)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;
        let secondary_input_tt_csv_path_pattern = global_config.replace_common_pattern(
            self.secondary_input_tt_csv_path_pattern
                .as_ref()
                .ok_or("secondary_input_tt_csv_path_pattern must be set")?,
        )?;
        let secondary_input_sd_csv_path_pattern = global_config.replace_common_pattern(
            self.secondary_input_sd_csv_path_pattern
                .as_ref()
                .ok_or("secondary_input_sd_csv_path_pattern must be set")?,
        )?;
        let seeds_to_use_as_space_sep_str = self.seeds_to_avg_over.get_seeds_to_use_as_str(config)?;
        let betas_to_use_as_space_sep_str = get_betas_to_use_as_space_sep_str(config)?;
        Ok(Some(vec![
            tt_csv_input_path_pattern,
            sd_csv_input_path_pattern,
            secondary_input_tt_csv_path_pattern,
            secondary_input_sd_csv_path_pattern,
            seeds_to_use_as_space_sep_str,
            betas_to_use_as_space_sep_str,
            self.which_deviation.clone(),
        ]))
    }

    /// The param sweep overwrites for this module are:
    /// 1. Either `use_random_seeds` or `read_from_random_seed_indices` will be overwritten to
    ///     contain only the value "use_all", since the plotting function will use all those
    ///     seeds itself. The seeds that are overwritten are those that are specified in the
    ///     module's `seeds_to_avg_over` field.
    /// 2. The `betas` parameter will be overwritten to contain only the value "use_all", since
    ///     the plotting function will use all those betas itself.
    /// Motivation: the plotting function uses all betas and seeds_to_avg_over itself, so the bash
    /// function should only be called once per cartesian product of the other parameters.
    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        let mut overwrites = self.seeds_to_avg_over.get_param_sweep_overwrites("use_all");
        // each plot contains all betas, so replace the beta array by a single value "use_all"
        // (it will never be read, but the important part is that it is only one value)
        overwrites.insert("betas".to_string(), "betas=(use_all)".to_string());
        Some(overwrites)
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "plot_deviation_scatterplots_to_diff_run_over_beta_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avg_deviations_to_diff_run_over_beta.sh",
            "python",
        )
    }
    /// The extra string arguments for this module are:
    /// 1. `tt_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 2. `sd_csv_input_path_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 3. `secondary_input_tt_csv_path_pattern` (read from module field)
    /// 4. `secondary_input_sd_csv_path_pattern` (read from module field)
    /// 5. `seeds_to_use_as_space_sep_str` (read from expset-config param sweep,
    ///     based on the module's `seeds_to_avg_over` field, and converted to a space-separated string)
    /// 6. `betas_to_use_as_space_sep_str` (read from expset-config param sweep, and converted to a space-separated string)
    /// 7. `which_deviation` (read from module field)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;
        let secondary_input_tt_csv_path_pattern = global_config.replace_common_pattern(
            self.secondary_input_tt_csv_path_pattern
                .as_ref()
                .ok_or("secondary_input_tt_csv_path_pattern must be set")?,
        )?;
        let secondary_input_sd_csv_path_pattern = global_config.replace_common_pattern(
            self.secondary_input_sd_csv_path_pattern
                .as_ref()
                .ok_or("secondary_input_sd_csv_path_pattern must be set")?,
        )?;
        let seeds_to_use_as_space_sep_str = self.seeds_to_avg_over.get_seeds_to_use_as_str(config)?;
        let betas_to_use_as_space_sep_str = get_betas_to_use_as_space_sep_str(config)?;
        Ok(Some(vec![
            tt_csv_input_path_pattern,
            sd_csv_input_path_pattern,
            secondary_input_tt_csv_path_pattern,
            secondary_input_sd_csv_path_pattern,
            seeds_to_use_as_space_sep_str,
            betas_to_use_as_space_sep_str,
            self.which_deviation.clone(),
        ]))
    }

    /// The param sweep overwrites for this module are:
    /// 1. Either `use_random_seeds` or `read_from_random_seed_indices` will be overwritten to
    ///     contain only the value "avg_over_all", since the plotting function will use all those
    ///     seeds itself. The seeds that are overwritten are those that are specified in the
    ///     module's `seeds_to_avg_over` field.
    /// 2. The `betas` parameter will be overwritten to contain only the value "use_all", since
    ///     the plotting function will use all those betas itself.
    /// Motivation: the plotting function uses all betas and seeds_to_avg_over itself, so the bash
    /// function should only be called once per cartesian product of the other parameters.
    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        let mut overwrites = self.seeds_to_avg_over.get_param_sweep_overwrites("avg_over_all");
        // each plot contains all betas, so replace the beta array by a single value "use_all"
        // (it will never be read, but the important part is that it is only one value)
        overwrites.insert("betas".to_string(), "betas=(use_all)".to_string());
        Some(overwrites)
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for ReformatOriginalExtractedMeasurements {
    fn get_bash_function(&self) -> BashFunction {
        BashFunction::new(
            "reformat_original_java_extracted_measurements_case",
            "./experiments/compare_braess_to_java/runnable_modules/reformat_original_measurements.sh",
            "rust",
        )
    }
    /// The extra string arguments for this module are:
    /// 1. `original_tt_tsv_file_pattern` (read from module field)
    /// 2. `original_sd_tsv_file_pattern` (read from module field)
    /// 3. `reformatted_tt_csv_file_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    /// 4. `reformatted_sd_csv_file_pattern` (read from expset-configs globals, or from the module's overwrites, if existing)
    fn get_extra_str_args(&self, config: &Config, global_config: &GlobalConfig) -> Result<Option<Vec<String>>, String> {
        let (reformatted_tt_csv_file_pattern, reformatted_sd_csv_file_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;
        // replace placeholders in file name
        let original_tt_tsv_file_pattern = global_config.replace_common_pattern(
            self.original_tt_tsv_file_pattern
                .as_ref()
                .ok_or("original_tt_tsv_file_pattern must be set")?,
        )?;
        let original_sd_tsv_file_pattern = global_config.replace_common_pattern(
            self.original_sd_tsv_file_pattern
                .as_ref()
                .ok_or("original_sd_tsv_file_pattern must be set")?,
        )?;
        Ok(Some(vec![
            original_tt_tsv_file_pattern,
            original_sd_tsv_file_pattern,
            reformatted_tt_csv_file_pattern,
            reformatted_sd_csv_file_pattern,
        ]))
    }

    /// The param sweep overwrites for this module are:
    /// 1. The `use_random_seeds` parameter will be overwritten to contain only the value "None",
    ///     since this module doesn't use any use_random seeds, and the bash function should only
    ///    be called once per cartesian product of the other parameters.
    fn get_param_sweep_overwrites(&self) -> Option<HashMap<String, String>> {
        let use_random_seeds_decl_overwrite = "use_random_seeds=(None)".to_string();
        let overwrites = HashMap::from_iter([(
            "use_random_seeds".to_string(),
            use_random_seeds_decl_overwrite.clone(),
        )]);
        Some(overwrites)
    }

    fn get_expset_config_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    // TODO think about if I need a test here, for e.g. making sure that nothing panics or smth.
    //  Also at some point consider to make things more solid, like verifying that the config file
    //  is valid, and that the modules are valid, etc.

    #[test]
    fn test_config_reading() {
        // Test the config reading functionality

        let parsed_config: Config = serde_yaml::from_reader(
            File::open("./../experiments/compare_braess_to_java/experiment_sets/test_config.yaml")
                .expect("Failed to open test config file"),
        )
            .expect("Failed to parse test config file");

        let expected_config = Config {
            globals: Some(HashMap::from([
                (
                    "experiment_set_name".to_string(),
                    Value::String("test_set".to_string()),
                ),
                (
                    "base_output_dir".to_string(),
                    Value::String("/tmp/output".to_string()),
                ),
                ("max_parallel_jobs".to_string(), Value::Number(8.into())),
            ])),
            param_sweep: HashMap::from([
                (
                    "replanning_variants".to_string(),
                    vec![Value::String("sel-exp10-switch-at80".to_string())],
                ),
                ("betas".to_string(), vec![Value::Number(1.into())]),
                (
                    "java_seed_indices".to_string(),
                    vec![
                        Value::Number(1.into()),
                        Value::Number(20.into()),
                        Value::Number(4.into()),
                    ],
                ),
                (
                    "rust_seeds".to_string(),
                    vec![Value::Number(42.into()), Value::Number(43.into())],
                ),
            ]),
            modules: vec![
                Box::new(RunRustBasedOnJavaOutput {
                    overwrite_mode: OverwriteFiles::DeleteDirectoryIfExists,
                    overwrites: None,
                }),
                Box::new(PlotTtAndSdAvgdOverSeeds {
                    overwrites: None,
                    seeds_to_avg_over: SeedsToAvgOver::UseRandomSeeds,
                }),
            ],
        };
        assert_eq!(
            parsed_config.globals, expected_config.globals,
            "Parsed config globals do not match expected config globals"
        );
        assert_eq!(
            parsed_config.param_sweep, expected_config.param_sweep,
            "Parsed config param_sweep does not match expected config param_sweep"
        );
        assert_eq!(
            parsed_config.modules.len(),
            expected_config.modules.len(),
            "Parsed config modules length does not match expected config modules length"
        );
    }
}
