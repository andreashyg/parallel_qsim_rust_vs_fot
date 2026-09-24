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
    pub(crate) fn get_global_config_parameter<'a>(&self, key: &str) -> Result<&Value, String> {
        // if key exists in (per experiment set) globals, return that value, else check in
        // global_config
        // if let Some(globals) = &self.globals {
        //     if let Some(value) = globals.get(key) {
        //         return Ok(value);
        //     }
        // }
        if let Some(value) = self.global_parameters.get(key) {
            return Ok(value);
        }
        // if let Some(global_config) = &self.global_config {
        //     if let Some(value) = global_config.get(key) {
        //         return Ok(value);
        //     }
        // }
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
    // global_config: Option<HashMap<String, Value>>, // whis is now a completely different file, not read here.
}

impl Config {
    pub(crate) fn get_expset_config_globals_parameter(
        &self,
        key: &str,
        // global_config: &'a GlobalConfig,
    ) -> Result<&Value, String> {
        // if key exists in (per experiment set) globals, return that value, else check in
        // global_config
        if let Some(globals) = &self.globals {
            if let Some(value) = globals.get(key) {
                return Ok(value);
            }
        }
        // if let Some(value) = global_config.global_parameters.get(key) {
        //     return Ok(value);
        // }
        // if let Some(global_config) = &self.global_config {
        //     if let Some(value) = global_config.get(key) {
        //         return Ok(value);
        //     }
        // }
        Err(format!("Global parameter '{}' not found in config.", key))
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
        let values = self
            .param_sweep
            .get(key)
            .ok_or_else(|| format!("Missing param_sweep key: {key}"))?;

        if values.is_empty() {
            return Err(format!("param_sweep.{key} must not be empty"));
        }

        let atoms: Result<Vec<_>, _> = values
            .iter()
            .map(bash_utils::yaml_value_as_shell_atom)
            .collect();
        Ok(format!("{var_name}=({})", atoms?.join(" ")))
    }

    // pub(crate) fn replace_common_pattern(
    //     &self,
    //     pattern: &str,
    //     global_config: &GlobalConfig,
    // ) -> Result<String, String> {
    //     if pattern.contains("{common_output_from_runs_pattern}") {
    //         let actual_pattern = global_config
    //             .get_global_config_parameter("common_output_from_runs_pattern")?
    //             .as_str()
    //             .ok_or_else(|| "common_output_from_runs_pattern must be a string".to_string())?;
    //
    //         // Replace the pattern with the actual common output path
    //         Ok(pattern.replace("{common_output_from_runs_pattern}", actual_pattern))
    //     } else if pattern.contains("{common_extracted_data_pattern}") {
    //         let actual_pattern = global_config
    //             .get_global_config_parameter("common_extracted_data_pattern")?
    //             .as_str()
    //             .ok_or_else(|| "common_extracted_data_pattern must be a string".to_string())?;
    //
    //         // Replace the pattern with the actual common input path
    //         Ok(pattern.replace("{common_extracted_data_pattern}", actual_pattern))
    //     } else if pattern.contains("{common_plots_pattern}") {
    //         let actual_pattern = global_config
    //             .get_global_config_parameter("common_plots_pattern")?
    //             .as_str()
    //             .ok_or_else(|| "common_plots_pattern must be a string".to_string())?;
    //
    //         // Replace the pattern with the actual common input path
    //         Ok(pattern.replace("{common_plots_pattern}", actual_pattern))
    //     } else if pattern.contains("{original_dir_to_read_from}") {
    //         let actual_pattern = global_config
    //             .get_global_config_parameter("original_dir_to_read_from")?
    //             .as_str()
    //             .ok_or_else(|| "original_dir_to_read_from must be a string".to_string())?;
    //         Ok(pattern.replace("{original_dir_to_read_from}", actual_pattern))
    //     } else {
    //         // If no known pattern is found, return the original pattern
    //         Ok(pattern.to_string())
    //     }
    // }

    pub(crate) fn run_all_modules(
        &self,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        // Run all modules.
        for module in &self.modules {
            module.run(self, global_config)?;
        }

        Ok(())
    }
}

#[typetag::serde(tag = "type")]
pub trait Module: Debug {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String>;

    /// Some modules may have the possibility to get overwrites for global parameters. The modules
    /// need to implement where they get them from (typically, a field self.overwrites).
    fn get_overwrites(&self) -> Option<HashMap<String, Value>>;

    /// Get a global parameter, or an overwrite if it exists.
    /// Does *not* read the global config, but just the global parameters of the experiment set
    /// specific config.
    fn get_expset_config_global_parameter_or_overwrite(
        &self,
        config: &Config,
        param_name: &str,
    ) -> Result<Value, String> {
        if let Some(overwrites) = &self.get_overwrites() {
            if let Some(value) = overwrites.get(param_name) {
                return Ok(value.clone());
            }
        }
        config
            .get_expset_config_globals_parameter(param_name)
            .cloned()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunRustBasedOnJavaOutput {
    pub overwrite_mode: OverwriteFiles,
    pub overwrites: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunJavaSingleIterBasedOnJavaOutput {
    pub overwrite_mode: OverwriteFiles,
    // pub config_file_pattern: Option<String>,
    // pub base_dir_to_read_from: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractMeasurementsFromEvents {
    pub input_file_stem_pattern: Option<String>,
    pub input_file_format: Option<String>,
    // overwrite global parameters by putting them in this hashmap
    pub overwrites: Option<HashMap<String, Value>>,
    // pub tt_csv_output_path_pattern: Option<String>,
    // pub sd_csv_output_path_pattern: Option<String>,
    pub num_parts: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddDummyCoordinatesToEvents {
    pub input_file_pattern: Option<String>,
    pub output_file_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReplaceActivityTimesInPopulation {
    pub overwrites: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdPerSeed {
    // pub output_tt_plot_path_pattern: Option<String>,
    // pub output_sd_plot_path_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
    // pub input_tt_csv_path_pattern: Option<String>,
    // pub input_sd_csv_path_pattern: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDiffPerSeed {
    // pub output_tt_plot_path_pattern: Option<String>,
    // pub output_sd_plot_path_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
    // pub main_input_tt_csv_path_pattern: Option<String>,
    // pub main_input_sd_csv_path_pattern: Option<String>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub minus_what: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdAvgdOverSeeds {
    // pub output_tt_plot_path_pattern: Option<String>,
    // pub output_sd_plot_path_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
    // pub input_tt_csv_path_pattern: Option<String>,
    // pub input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdAvgdOverSeedsDiff {
    // pub output_tt_plot_path_pattern: Option<String>,
    // pub output_sd_plot_path_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
    // pub main_input_tt_csv_path_pattern: Option<String>,
    // pub main_input_sd_csv_path_pattern: Option<String>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
    pub minus_what: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToNashBoxplotsOverBeta {
    // pub output_tt_plot_path_pattern: Option<String>,
    // pub output_sd_plot_path_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
    // pub input_tt_csv_path_pattern: Option<String>,
    // pub input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToNashScatterplotsOverBeta {
    // pub output_tt_plot_path_pattern: Option<String>,
    // pub output_sd_plot_path_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
    // pub input_tt_csv_path_pattern: Option<String>,
    // pub input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta {
    // pub output_tt_plot_path_pattern: Option<String>,
    // pub output_sd_plot_path_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
    // pub main_input_tt_csv_path_pattern: Option<String>,
    // pub main_input_sd_csv_path_pattern: Option<String>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
    pub which_deviation: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta {
    // pub output_tt_plot_path_pattern: Option<String>,
    // pub output_sd_plot_path_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
    // pub main_input_tt_csv_path_pattern: Option<String>,
    // pub main_input_sd_csv_path_pattern: Option<String>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
    pub which_deviation: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReformatOriginalExtractedMeasurements {
    pub original_tt_tsv_file_pattern: Option<String>,
    pub original_sd_tsv_file_pattern: Option<String>,
    // pub reformatted_tt_csv_file_pattern: Option<String>,
    // pub reformatted_sd_csv_file_pattern: Option<String>,
    pub overwrites: Option<HashMap<String, Value>>,
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
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running RunRustBasedOnJavaOutput module.\n");

        let bash_function = BashFunction::new(
            "run_experiment_case",
            "./experiments/compare_braess_to_java/runnable_modules/run_rust_based_on_java_output.sh",
        );

        let delete_output_dir_if_existing = match self.overwrite_mode {
            OverwriteFiles::DeleteDirectoryIfExists => "true".to_string(),
            _ => "false".to_string(),
        };

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![delete_output_dir_if_existing]),
            None,
            // global_log_file,
        )
    }

    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for RunJavaSingleIterBasedOnJavaOutput {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running RunJavaSingleIterBasedOnJavaOutput module.\n");

        let bash_function = BashFunction::new(
            "run_java_single_iter_case",
            "./experiments/compare_braess_to_java/runnable_modules/run_java_single_iter_based_on_java_output.sh",
        );

        let delete_output_dir_if_existing = match self.overwrite_mode {
            OverwriteFiles::DeleteDirectoryIfExists => "true".to_string(),
            _ => "false".to_string(),
        };

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                delete_output_dir_if_existing,
                // self.config_file_pattern
                //     .clone()
                //     .expect("config_file_pattern must be set"),
                // this is now read from the global config directly in bash
                // self.base_dir_to_read_from
                //     .clone()
                //     .expect("base_dir_to_read_from must be set"),
            ]),
            None,
            // global_log_file,
        )
    }

    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for ExtractMeasurementsFromEvents {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running ExtractMeasurementsFromEvents module.\n");

        // Note: this will run for all combinations of java and rust seeds. While this would not
        // make sense in the case of reading original java data, the intended use is that in such
        // an experiment set, the only rust seed would be "None"

        let (tt_csv_output_path_pattern, sd_csv_output_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;

        let bash_function = BashFunction::new(
            "extract_travel_time_sum_dep_case",
            "./experiments/compare_braess_to_java/runnable_modules/extract_tt_and_sd_from_events.sh",
        );

        let input_file_stem_pattern = global_config.replace_common_pattern(
            self.input_file_stem_pattern
                .as_ref()
                .ok_or("input_file_stem_pattern must be set")?,
        )?;

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                input_file_stem_pattern,
                self.input_file_format
                    .clone()
                    .ok_or("input_file_format must be set")?,
                // self.tt_csv_output_path_pattern
                //     .clone()
                //     .ok_or("tt_csv_output_path_pattern must be set")?,
                tt_csv_output_path_pattern,
                // self.sd_csv_output_path_pattern
                //     .clone()
                //     .ok_or("sd_csv_output_path_pattern must be set")?,
                sd_csv_output_path_pattern,
                self.num_parts.ok_or("num_parts must be set")?.to_string(),
            ]),
            None,
            // global_log_file,
        )
    }
    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for AddDummyCoordinatesToEvents {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running AddDummyCoordinatesToEvents module.\n");

        let bash_function = BashFunction::new(
            "add_dummy_coordinates_case",
            "./experiments/compare_braess_to_java/runnable_modules/add_dummy_coordinates_to_events.sh",
        );

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

        // // this module doesn't use any rust seeds, so we overwrite the rust_seeds variable to
        // // contain only one parameter, so that the bash function is only called once per cartesian
        // // product of the other parameters
        // let rust_seed_decl_overwrite = "rust_seeds=(None)".to_string();
        // let overwrites =
        //     HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite.clone())]);

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![input_file_stem_pattern, output_file_stem_pattern]),
            None, // Some(overwrites),
                  // global_log_file,
        )
    }
    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for ReplaceActivityTimesInPopulation {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running ReplaceActivityTimesInPopulation module.\n");

        let bash_function = BashFunction::new(
            "replace_activity_times_case",
            "./experiments/compare_braess_to_java/runnable_modules/replace_activity_times.sh",
        );

        // this module doesn't use any rust seeds, so we overwrite the rust_seeds variable to
        // contain only one parameter, so that the bash function is only called once per cartesian
        // product of the other parameters
        let rust_seed_decl_overwrite = "rust_seeds=(None)".to_string();
        let overwrites =
            HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite.clone())]);

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            None,
            Some(overwrites),
            // global_log_file,
        )
    }
    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdPerSeed {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running PlotTtAndSdPerSeed module.\n");

        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;

        let bash_function = BashFunction::new(
            "plot_per_seed_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_per_seed.sh",
        );

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![tt_csv_input_path_pattern, sd_csv_input_path_pattern]),
            None,
            // global_log_file,
        )
    }
    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDiffPerSeed {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running PlotTtAndSdDiffPerSeed module.\n");

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

        let bash_function = BashFunction::new(
            "plot_diff_per_seed_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_diff_per_seed.sh",
        );

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                tt_csv_input_path_pattern,
                sd_csv_input_path_pattern,
                secondary_input_tt_csv_path_pattern,
                secondary_input_sd_csv_path_pattern,
                self.minus_what.clone(),
            ]),
            None,
            // global_log_file,
        )
    }
    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdAvgdOverSeeds {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running PlotTtAndSdAvgdOverSeeds module.\n");

        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;

        let bash_function = BashFunction::new(
            "plot_avgd_over_seeds_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avgd_over_seeds.sh",
        );

        let (seeds_to_use_str, overwrites) = match self.seeds_to_avg_over {
            SeedsToAvgOver::RustSeeds => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("rust_seeds")
                    .ok_or("rust_seeds must be set in param_sweep for PlotTtAndSdAvgdOverSeeds module when averaging over rust seeds")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();

                let rust_seed_decl_overwrite = format!(
                    "rust_seeds=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String(
                        "avg_over_all".to_string()
                    ))?
                );
                let overwrites =
                    HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite)]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
            SeedsToAvgOver::JavaSeedIndices => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("java_seed_indices")
                    .ok_or("java_seed_indices must be set in param_sweep for PlotTtAndSdAvgdOverSeeds module when averaging over java seed indices")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();
                let java_seed_decl_overwrite = format!(
                    "java_seed_indices=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String(
                        "avg_over_all".to_string()
                    ))?
                );
                let overwrites = HashMap::from_iter([(
                    "java_seed_indices".to_string(),
                    java_seed_decl_overwrite,
                )]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
        };

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                tt_csv_input_path_pattern,
                sd_csv_input_path_pattern,
                seeds_to_use_str,
            ]),
            Some(overwrites),
            // global_log_file,
        )
    }

    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdAvgdOverSeedsDiff {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running PlotTtAndSdAvgdOverSeedsDiff module.\n");

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

        let bash_function = BashFunction::new(
            "plot_avgd_over_seeds_diff_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avgd_over_seeds_diff.sh",
        );

        let (seeds_to_use_str, overwrites) = match self.seeds_to_avg_over {
            SeedsToAvgOver::RustSeeds => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("rust_seeds")
                    .ok_or("rust_seeds must be set in param_sweep for PlotTtAndSdAvgdOverSeeds module when averaging over rust seeds")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();

                let rust_seed_decl_overwrite = format!(
                    "rust_seeds=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String(
                        "avg_over_all".to_string()
                    ))?
                );
                let overwrites =
                    HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite)]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
            SeedsToAvgOver::JavaSeedIndices => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("java_seed_indices")
                    .ok_or("java_seed_indices must be set in param_sweep for PlotTtAndSdAvgdOverSeeds module when averaging over java seed indices")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();
                let java_seed_decl_overwrite = format!(
                    "java_seed_indices=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String(
                        "avg_over_all".to_string()
                    ))?
                );
                let overwrites = HashMap::from_iter([(
                    "java_seed_indices".to_string(),
                    java_seed_decl_overwrite,
                )]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
        };

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                tt_csv_input_path_pattern,
                sd_csv_input_path_pattern,
                secondary_input_tt_csv_path_pattern,
                secondary_input_sd_csv_path_pattern,
                seeds_to_use_str,
                self.minus_what.clone(),
            ]),
            Some(overwrites),
            // global_log_file,
        )
    }

    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToNashBoxplotsOverBeta {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running PlotTtAndSdDeviationBoxplotsOverBeta module.\n");

        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;

        let bash_function = BashFunction::new(
            "plot_deviation_boxplots_over_beta_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avg_deviations_to_nash_over_beta.sh",
        );
        // TODO this is a duplicate, should be refactored into a function to avoid code duplication
        let (seeds_to_use_str, mut overwrites) = match self.seeds_to_avg_over {
            SeedsToAvgOver::RustSeeds => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("rust_seeds")
                    .ok_or("rust_seeds must be set in param_sweep for PlotTtAndSdAvgdOverSeeds module when averaging over rust seeds")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();

                let rust_seed_decl_overwrite = format!(
                    "rust_seeds=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String(
                        "avg_over_all".to_string()
                    ))?
                );
                let overwrites =
                    HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite)]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
            SeedsToAvgOver::JavaSeedIndices => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("java_seed_indices")
                    .ok_or("java_seed_indices must be set in param_sweep for PlotTtAndSdAvgdOverSeeds module when averaging over java seed indices")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();
                let java_seed_decl_overwrite = format!(
                    "java_seed_indices=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String(
                        "avg_over_all".to_string()
                    ))?
                );
                let overwrites = HashMap::from_iter([(
                    "java_seed_indices".to_string(),
                    java_seed_decl_overwrite,
                )]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
        };

        // each plot contains all betas, so replace the beta array by a single value "use_all"
        overwrites.insert("betas".to_string(), "betas=(use_all)".to_string());
        // also pass the array of betas, so the plot function knows which betas to use
        // note: these are passed as a comma separated string, not as a bash array.
        // This is the easiest way and shouldn't cause any problems, since bash will simply pass on
        // this string to python, where the string can easily be separated into a list of ints
        let betas_to_use_str = config.param_sweep.get("betas")
            .ok_or("betas must be set in param_sweep for PlotTtAndSdDeviationScatterplotsOverBeta module")?
            .iter()
            .map(bash_utils::yaml_value_as_shell_atom)
            .collect::<Result<Vec<_>, _>>()?.join(" ");

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                tt_csv_input_path_pattern,
                sd_csv_input_path_pattern,
                seeds_to_use_str,
                betas_to_use_str,
            ]),
            Some(overwrites),
            // global_log_file,
        )
    }

    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToNashScatterplotsOverBeta {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running PlotTtAndSdDeviationScatterplotsOverBeta module.\n");

        let (tt_csv_input_path_pattern, sd_csv_input_path_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;

        let bash_function = BashFunction::new(
            "plot_deviation_scatterplots_over_beta_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avg_deviations_to_nash_over_beta.sh",
        );
        // TODO this is a duplicate, should be refactored into a function to avoid code duplication
        let (seeds_to_use_str, mut overwrites) = match self.seeds_to_avg_over {
            SeedsToAvgOver::RustSeeds => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("rust_seeds")
                    .ok_or("rust_seeds must be set in param_sweep for PlotTtAndSdAvgdOverSeeds module when averaging over rust seeds")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();

                let rust_seed_decl_overwrite = format!(
                    "rust_seeds=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String(
                        "avg_over_all".to_string()
                    ))?
                );
                let overwrites =
                    HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite)]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
            SeedsToAvgOver::JavaSeedIndices => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("java_seed_indices")
                    .ok_or("java_seed_indices must be set in param_sweep for PlotTtAndSdAvgdOverSeeds module when averaging over java seed indices")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();
                let java_seed_decl_overwrite = format!(
                    "java_seed_indices=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String(
                        "avg_over_all".to_string()
                    ))?
                );
                let overwrites = HashMap::from_iter([(
                    "java_seed_indices".to_string(),
                    java_seed_decl_overwrite,
                )]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
        };

        // each plot contains all betas, so replace the beta array by a single value "use_all"
        overwrites.insert("betas".to_string(), "betas=(use_all)".to_string());
        // also pass the array of betas, so the plot function knows which betas to use
        // note: these are passed as a comma separated string, not as a bash array.
        // This is the easiest way and shouldn't cause any problems, since bash will simply pass on
        // this string to python, where the string can easily be separated into a list of ints
        let betas_to_use_str = config.param_sweep.get("betas")
            .ok_or("betas must be set in param_sweep for PlotTtAndSdDeviationScatterplotsOverBeta module")?
            .iter()
            .map(bash_utils::yaml_value_as_shell_atom)
            .collect::<Result<Vec<_>, _>>()?.join(" ");

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                tt_csv_input_path_pattern,
                sd_csv_input_path_pattern,
                seeds_to_use_str,
                betas_to_use_str,
            ]),
            Some(overwrites),
            // global_log_file,
        )
    }

    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta module.\n");

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

        let bash_function = BashFunction::new(
            "plot_deviation_boxplots_to_diff_run_over_beta_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avg_deviations_to_diff_run_over_beta.sh",
        );

        // TODO this is a duplicate, should be refactored into a function to avoid code duplication
        //  EXCEPT here I have "use_all" instead of "avg_over_all", but maybe that should change elsewhere as well
        let (seeds_to_use_str, mut overwrites) = match self.seeds_to_avg_over {
            SeedsToAvgOver::RustSeeds => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("rust_seeds")
                    .ok_or("rust_seeds must be set in param_sweep for PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta module when averaging over rust seeds")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();

                let rust_seed_decl_overwrite = format!(
                    "rust_seeds=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String("use_all".to_string()))?
                );
                let overwrites =
                    HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite)]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
            SeedsToAvgOver::JavaSeedIndices => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("java_seed_indices")
                    .ok_or("java_seed_indices must be set in param_sweep for PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta module when averaging over java seed indices")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();
                let java_seed_decl_overwrite = format!(
                    "java_seed_indices=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String("use_all".to_string()))?
                );
                let overwrites = HashMap::from_iter([(
                    "java_seed_indices".to_string(),
                    java_seed_decl_overwrite,
                )]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
        };

        // each plot contains all betas, so replace the beta array by a single value "use_all"
        overwrites.insert("betas".to_string(), "betas=(use_all)".to_string());
        // also pass the array of betas, so the plot function knows which betas to use
        // note: these are passed as a comma separated string, not as a bash array.
        // This is the easiest way and shouldn't cause any problems, since bash will simply pass on
        // this string to python, where the string can easily be separated into a list of ints
        let betas_to_use_str = config.param_sweep.get("betas")
            .ok_or("betas must be set in param_sweep for PlotTtAndSdDeviationToDiffRUnBoxplotsOverBeta module")?
            .iter()
            .map(bash_utils::yaml_value_as_shell_atom)
            .collect::<Result<Vec<_>, _>>()?.join(" ");

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                tt_csv_input_path_pattern,
                sd_csv_input_path_pattern,
                secondary_input_tt_csv_path_pattern,
                secondary_input_sd_csv_path_pattern,
                seeds_to_use_str,
                betas_to_use_str,
                self.which_deviation.clone(),
            ]),
            Some(overwrites),
            // global_log_file,
        )
    }

    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta module.\n");

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

        let bash_function = BashFunction::new(
            "plot_deviation_scatterplots_to_diff_run_over_beta_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_avg_deviations_to_diff_run_over_beta.sh",
        );

        // TODO this is a duplicate, should be refactored into a function to avoid code duplication
        //  EXCEPT here I have "use_all" instead of "avg_over_all", but maybe that should change elsewhere as well
        let (seeds_to_use_str, mut overwrites) = match self.seeds_to_avg_over {
            SeedsToAvgOver::RustSeeds => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("rust_seeds")
                    .ok_or("rust_seeds must be set in param_sweep for PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta module when averaging over rust seeds")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();

                let rust_seed_decl_overwrite = format!(
                    "rust_seeds=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String("use_all".to_string()))?
                );
                let overwrites =
                    HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite)]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
            SeedsToAvgOver::JavaSeedIndices => {
                let seeds_to_use_atoms: Result<Vec<_>, _> = config
                    .param_sweep
                    .get("java_seed_indices")
                    .ok_or("java_seed_indices must be set in param_sweep for PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta module when averaging over java seed indices")?
                    .iter()
                    .map(bash_utils::yaml_value_as_shell_atom)
                    .collect();
                let java_seed_decl_overwrite = format!(
                    "java_seed_indices=({})",
                    bash_utils::yaml_value_as_shell_atom(&Value::String("use_all".to_string()))?
                );
                let overwrites = HashMap::from_iter([(
                    "java_seed_indices".to_string(),
                    java_seed_decl_overwrite,
                )]);

                (seeds_to_use_atoms?.join(" "), overwrites)
            }
        };

        // each plot contains all betas, so replace the beta array by a single value "use_all"
        overwrites.insert("betas".to_string(), "betas=(use_all)".to_string());
        // also pass the array of betas, so the plot function knows which betas to use
        // note: these are passed as a comma separated string, not as a bash array.
        // This is the easiest way and shouldn't cause any problems, since bash will simply pass on
        // this string to python, where the string can easily be separated into a list of ints
        let betas_to_use_str = config.param_sweep.get("betas")
            .ok_or("betas must be set in param_sweep for PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta module")?
            .iter()
            .map(bash_utils::yaml_value_as_shell_atom)
            .collect::<Result<Vec<_>, _>>()?.join(" ");

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                tt_csv_input_path_pattern,
                sd_csv_input_path_pattern,
                secondary_input_tt_csv_path_pattern,
                secondary_input_sd_csv_path_pattern,
                seeds_to_use_str,
                betas_to_use_str,
                self.which_deviation.clone(),
            ]),
            Some(overwrites),
            // global_log_file,
        )
    }

    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[typetag::serde]
impl Module for ReformatOriginalExtractedMeasurements {
    fn run(
        &self,
        config: &Config,
        global_config: &GlobalConfig,
        // global_log_file: Arc<Mutex<File>>,
    ) -> Result<(), String> {
        println!("Running ReformatOriginalExtractedMeasurements module.\n");

        let (reformatted_tt_csv_file_pattern, reformatted_sd_csv_file_pattern) =
            get_tt_and_sd_csv_path_patterns(config, global_config, self)?;

        let bash_function = BashFunction::new(
            "reformat_original_java_extracted_measurements_case",
            "./experiments/compare_braess_to_java/runnable_modules/reformat_original_measurements.sh",
        );

        // replace placholders in file name
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

        // This is now the default csv file pattern, or overwrites if they are given.

        // let reformatted_tt_csv_file_pattern = config.replace_common_pattern(
        //     self.reformatted_tt_csv_file_pattern
        //         .as_ref()
        //         .ok_or("reformatted_tt_csv_file_pattern must be set")?,
        //     global_config,
        // )?;
        // let reformatted_sd_csv_file_pattern = config.replace_common_pattern(
        //     self.reformatted_sd_csv_file_pattern
        //         .as_ref()
        //         .ok_or("reformatted_sd_csv_file_pattern must be set")?,
        //     global_config,
        // )?;

        // this module doesn't use any rust seeds, so we overwrite the rust_seeds variable to
        // contain only one parameter, so that the bash function is only called once per cartesian
        // product of the other parameters
        let rust_seed_decl_overwrite = "rust_seeds=(None)".to_string();
        let overwrites =
            HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite.clone())]);

        bash_function.run_for_cartprod_in_parallel(
            config,
            global_config,
            self,
            Some(vec![
                original_tt_tsv_file_pattern,
                original_sd_tsv_file_pattern,
                reformatted_tt_csv_file_pattern,
                reformatted_sd_csv_file_pattern,
            ]),
            Some(overwrites),
            // global_log_file,
        )
    }

    fn get_overwrites(&self) -> Option<HashMap<String, Value>> {
        self.overwrites.clone()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SeedsToAvgOver {
    RustSeeds,
    JavaSeedIndices,
}

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
                    // output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/avg_over_rust_seeds/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_from_random}.pdf".to_string()),
                    // output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/avg_over_rust_seeds/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{read_from_random}.pdf".to_string()),
                    overwrites: None,
                    // input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    // input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::RustSeeds,
                }),
            ],
            // global_config: None,
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

        println!("{:?}", parsed_config.modules);
        println!("{:?}", expected_config.modules);

        // assert_eq!(
        //     parsed_config.global_config, expected_config.global_config,
        //     "Parsed global config does not match expected config global config"
        // );
    }
}
