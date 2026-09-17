use crate::bash_utils;
use crate::bash_utils::BashFunction;
use rust_qsim::simulation::config::OverwriteFiles;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    globals: Option<HashMap<String, Value>>,
    param_sweep: HashMap<String, Vec<Value>>,
    modules: Vec<Box<dyn Module>>,
    global_config: Option<HashMap<String, Value>>, // from separate global config file TODO this will not work automatically like this
}

impl Config {
    pub(crate) fn get_global_parameter(&self, key: &str) -> Result<&Value, String> {
        // if key exists in (per experiment set) globals, return that value, else check in
        // global_config
        if let Some(globals) = &self.globals {
            if let Some(value) = globals.get(key) {
                return Ok(value);
            }
        }
        if let Some(global_config) = &self.global_config {
            if let Some(value) = global_config.get(key) {
                return Ok(value);
            }
        }
        Err(format!(
            "Global parameter '{}' not found neither in per-experiment-set globals or in global config",
            key
        ))
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

    pub(crate) fn run_all_modules(&self) -> Result<(), String> {
        for module in &self.modules {
            module.run(self)?;
        }
        Ok(())
    }
}

#[typetag::serde(tag = "type")]
pub trait Module: Debug {
    fn run(&self, config: &Config) -> Result<(), String>;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunRustBasedOnJavaOutput {
    pub overwrite_mode: OverwriteFiles,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunJavaSingleIterBasedOnJavaOutput {
    pub overwrite_mode: OverwriteFiles,
    pub config_file_pattern: Option<String>,
    pub base_dir_to_read_from: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractMeasurementsFromEvents {
    pub input_file_stem_pattern: Option<String>,
    pub input_file_format: Option<String>,
    pub tt_csv_output_path_pattern: Option<String>,
    pub sd_csv_output_path_pattern: Option<String>,
    pub num_parts: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddDummyCoordinatesToEvents {
    pub input_file_stem_pattern: Option<String>,
    pub output_file_stem_pattern: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdPerSeed {
    pub output_tt_plot_path_pattern: Option<String>,
    pub output_sd_plot_path_pattern: Option<String>,
    pub input_tt_csv_path_pattern: Option<String>,
    pub input_sd_csv_path_pattern: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDiffPerSeed {
    pub output_tt_plot_path_pattern: Option<String>,
    pub output_sd_plot_path_pattern: Option<String>,
    pub main_input_tt_csv_path_pattern: Option<String>,
    pub main_input_sd_csv_path_pattern: Option<String>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdAvgdOverSeeds {
    pub output_tt_plot_path_pattern: Option<String>,
    pub output_sd_plot_path_pattern: Option<String>,
    pub input_tt_csv_path_pattern: Option<String>,
    pub input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdAvgdOverSeedsDiff {
    pub output_tt_plot_path_pattern: Option<String>,
    pub output_sd_plot_path_pattern: Option<String>,
    pub main_input_tt_csv_path_pattern: Option<String>,
    pub main_input_sd_csv_path_pattern: Option<String>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToNashBoxplotsOverBeta {
    pub output_tt_plot_path_pattern: Option<String>,
    pub output_sd_plot_path_pattern: Option<String>,
    pub input_tt_csv_path_pattern: Option<String>,
    pub input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToNashScatterplotsOverBeta {
    pub output_tt_plot_path_pattern: Option<String>,
    pub output_sd_plot_path_pattern: Option<String>,
    pub input_tt_csv_path_pattern: Option<String>,
    pub input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta {
    pub output_tt_plot_path_pattern: Option<String>,
    pub output_sd_plot_path_pattern: Option<String>,
    pub main_input_tt_csv_path_pattern: Option<String>,
    pub main_input_sd_csv_path_pattern: Option<String>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta {
    pub output_tt_plot_path_pattern: Option<String>,
    pub output_sd_plot_path_pattern: Option<String>,
    pub main_input_tt_csv_path_pattern: Option<String>,
    pub main_input_sd_csv_path_pattern: Option<String>,
    pub secondary_input_tt_csv_path_pattern: Option<String>,
    pub secondary_input_sd_csv_path_pattern: Option<String>,
    pub seeds_to_avg_over: SeedsToAvgOver,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReformatOriginalExtractedMeasurements {
    pub original_tt_tsv_file_pattern: Option<String>,
    pub original_sd_tsv_file_pattern: Option<String>,
    pub reformatted_tt_csv_file_pattern: Option<String>,
    pub reformatted_sd_csv_file_pattern: Option<String>,
}

#[typetag::serde]
impl Module for RunRustBasedOnJavaOutput {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running RunRustBasedOnJavaOutput module.\n");

        let bash_function = BashFunction::new(
            "run_experiment_case",
            "./experiments/compare_braess_to_java/runnable_modules/run_rust_based_on_java_output.sh",
        );

        bash_function.run_for_cartprod_in_parallel(config, None, None)
    }
}

#[typetag::serde]
impl Module for RunJavaSingleIterBasedOnJavaOutput {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running RunJavaSingleIterBasedOnJavaOutput module.\n");

        let bash_function = BashFunction::new(
            "run_java_single_iter_case",
            "./experiments/compare_braess_to_java/runnable_modules/run_java_single_iter_based_on_java_output.sh",
        );

        bash_function.run_for_cartprod_in_parallel(
            config,
            Some(vec![
                self.config_file_pattern
                    .clone()
                    .expect("config_file_pattern must be set"),
                self.base_dir_to_read_from
                    .clone()
                    .expect("base_dir_to_read_from must be set"),
            ]),
            None,
        )
    }
}

#[typetag::serde]
impl Module for ExtractMeasurementsFromEvents {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running ExtractMeasurementsFromEvents module.\n");

        let bash_function = BashFunction::new(
            "extract_travel_time_sum_dep_case",
            "./experiments/compare_braess_to_java/runnable_modules/extract_tt_and_sd_from_events.sh",
        );

        // FIXME this should use a function to get these patterns from the config (hierarchy module/globals/global_config)
        bash_function.run_for_cartprod_in_parallel(
            config,
            Some(vec![
                self.input_file_stem_pattern
                    .clone()
                    .ok_or("input_file_stem_pattern must be set")?,
                self.input_file_format
                    .clone()
                    .ok_or("input_file_format must be set")?,
                self.tt_csv_output_path_pattern
                    .clone()
                    .ok_or("tt_csv_output_path_pattern must be set")?,
                self.sd_csv_output_path_pattern
                    .clone()
                    .ok_or("sd_csv_output_path_pattern must be set")?,
                self.num_parts.ok_or("num_parts must be set")?.to_string(),
            ]),
            None,
        )
    }
}

#[typetag::serde]
impl Module for AddDummyCoordinatesToEvents {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running AddDummyCoordinatesToEvents module.\n");

        let bash_function = BashFunction::new(
            "add_dummy_coordinates_case",
            "./experiments/compare_braess_to_java/runnable_modules/add_dummy_coordinates_to_events.sh",
        );

        // this module doesn't use any rust seeds, so we overwrite the rust_seeds variable to
        // contain only one parameter, so that the bash function is only called once per cartesian
        // product of the other parameters
        let rust_seed_decl_overwrite = "rust_seeds=(None)".to_string();
        let overwrites =
            HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite.clone())]);

        // FIXME also here, the pattern should be read correctly from the config (e.g. using hierarchy)
        bash_function.run_for_cartprod_in_parallel(
            config,
            Some(vec![
                self.input_file_stem_pattern.clone().unwrap(),
                self.output_file_stem_pattern.clone().unwrap(),
            ]),
            Some(overwrites),
        )
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdPerSeed {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running PlotTtAndSdPerSeed module.\n");

        let bash_function = BashFunction::new(
            "plot_per_seed_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_per_seed.sh",
        );

        bash_function.run_for_cartprod_in_parallel(
            config,
            Some(vec![
                self.output_tt_plot_path_pattern
                    .clone()
                    .ok_or("output_tt_plot_path_pattern must be set")?,
                self.output_sd_plot_path_pattern
                    .clone()
                    .ok_or("output_sd_plot_path_pattern must be set")?,
                self.input_tt_csv_path_pattern
                    .clone()
                    .ok_or("input_tt_csv_path_pattern must be set")?,
                self.input_sd_csv_path_pattern
                    .clone()
                    .ok_or("input_sd_csv_path_pattern must be set")?,
            ]),
            None,
        )
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDiffPerSeed {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running PlotTtAndSdDiffPerSeed module.\n");

        let bash_function = BashFunction::new(
            "plot_diff_per_seed_case",
            "./experiments/compare_braess_to_java/runnable_modules/plot_tt_and_sd_diff_per_seed.sh",
        );

        bash_function.run_for_cartprod_in_parallel(
            config,
            Some(vec![
                self.output_tt_plot_path_pattern
                    .clone()
                    .ok_or("output_tt_plot_path_pattern must be set")?,
                self.output_sd_plot_path_pattern
                    .clone()
                    .ok_or("output_sd_plot_path_pattern must be set")?,
                self.main_input_tt_csv_path_pattern
                    .clone()
                    .ok_or("main_input_tt_csv_path_pattern must be set")?,
                self.main_input_sd_csv_path_pattern
                    .clone()
                    .ok_or("main_input_sd_csv_path_pattern must be set")?,
                self.secondary_input_tt_csv_path_pattern
                    .clone()
                    .ok_or("secondary_input_tt_csv_path_pattern must be set")?,
                self.secondary_input_sd_csv_path_pattern
                    .clone()
                    .ok_or("secondary_input_sd_csv_path_pattern must be set")?,
            ]),
            None,
        )
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdAvgdOverSeeds {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running PlotTtAndSdAvgdOverSeeds module.\n");

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
            Some(vec![
                self.output_tt_plot_path_pattern
                    .clone()
                    .ok_or("output_tt_plot_path_pattern must be set")?,
                self.output_sd_plot_path_pattern
                    .clone()
                    .ok_or("output_sd_plot_path_pattern must be set")?,
                self.input_tt_csv_path_pattern
                    .clone()
                    .ok_or("input_tt_csv_path_pattern must be set")?,
                self.input_sd_csv_path_pattern
                    .clone()
                    .ok_or("input_sd_csv_path_pattern must be set")?,
                seeds_to_use_str,
            ]),
            Some(overwrites),
        )
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdAvgdOverSeedsDiff {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running PlotTtAndSdAvgdOverSeedsDiff module.\n");

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
            Some(vec![
                self.output_tt_plot_path_pattern
                    .clone()
                    .ok_or("output_tt_plot_path_pattern must be set")?,
                self.output_sd_plot_path_pattern
                    .clone()
                    .ok_or("output_sd_plot_path_pattern must be set")?,
                self.main_input_tt_csv_path_pattern
                    .clone()
                    .ok_or("main_input_tt_csv_path_pattern must be set")?,
                self.main_input_sd_csv_path_pattern
                    .clone()
                    .ok_or("main_input_sd_csv_path_pattern must be set")?,
                self.secondary_input_tt_csv_path_pattern
                    .clone()
                    .ok_or("secondary_input_tt_csv_path_pattern must be set")?,
                self.secondary_input_sd_csv_path_pattern
                    .clone()
                    .ok_or("secondary_input_sd_csv_path_pattern must be set")?,
                seeds_to_use_str,
            ]),
            Some(overwrites),
        )
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToNashBoxplotsOverBeta {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running PlotTtAndSdDeviationBoxplotsOverBeta module.\n");

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
            Some(vec![
                self.output_tt_plot_path_pattern
                    .clone()
                    .ok_or("output_tt_plot_path_pattern must be set")?,
                self.output_sd_plot_path_pattern
                    .clone()
                    .ok_or("output_sd_plot_path_pattern must be set")?,
                self.input_tt_csv_path_pattern
                    .clone()
                    .ok_or("input_tt_csv_path_pattern must be set")?,
                self.input_sd_csv_path_pattern
                    .clone()
                    .ok_or("input_sd_csv_path_pattern must be set")?,
                seeds_to_use_str,
                betas_to_use_str,
            ]),
            Some(overwrites),
        )
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToNashScatterplotsOverBeta {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running PlotTtAndSdDeviationScatterplotsOverBeta module.\n");

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
            Some(vec![
                self.output_tt_plot_path_pattern
                    .clone()
                    .ok_or("output_tt_plot_path_pattern must be set")?,
                self.output_sd_plot_path_pattern
                    .clone()
                    .ok_or("output_sd_plot_path_pattern must be set")?,
                self.input_tt_csv_path_pattern
                    .clone()
                    .ok_or("input_tt_csv_path_pattern must be set")?,
                self.input_sd_csv_path_pattern
                    .clone()
                    .ok_or("input_sd_csv_path_pattern must be set")?,
                seeds_to_use_str,
                betas_to_use_str,
            ]),
            Some(overwrites),
        )
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta module.\n");

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
            Some(vec![
                self.output_tt_plot_path_pattern
                    .clone()
                    .ok_or("output_tt_plot_path_pattern must be set")?,
                self.output_sd_plot_path_pattern
                    .clone()
                    .ok_or("output_sd_plot_path_pattern must be set")?,
                self.main_input_tt_csv_path_pattern
                    .clone()
                    .ok_or("main_input_tt_csv_path_pattern must be set")?,
                self.main_input_sd_csv_path_pattern
                    .clone()
                    .ok_or("main_input_sd_csv_path_pattern must be set")?,
                self.secondary_input_tt_csv_path_pattern
                    .clone()
                    .ok_or("secondary_input_tt_csv_path_pattern must be set")?,
                self.secondary_input_sd_csv_path_pattern
                    .clone()
                    .ok_or("secondary_input_sd_csv_path_pattern must be set")?,
                seeds_to_use_str,
                betas_to_use_str,
            ]),
            Some(overwrites),
        )
    }
}

#[typetag::serde]
impl Module for PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta module.\n");

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
            Some(vec![
                self.output_tt_plot_path_pattern
                    .clone()
                    .ok_or("output_tt_plot_path_pattern must be set")?,
                self.output_sd_plot_path_pattern
                    .clone()
                    .ok_or("output_sd_plot_path_pattern must be set")?,
                self.main_input_tt_csv_path_pattern
                    .clone()
                    .ok_or("main_input_tt_csv_path_pattern must be set")?,
                self.main_input_sd_csv_path_pattern
                    .clone()
                    .ok_or("main_input_sd_csv_path_pattern must be set")?,
                self.secondary_input_tt_csv_path_pattern
                    .clone()
                    .ok_or("secondary_input_tt_csv_path_pattern must be set")?,
                self.secondary_input_sd_csv_path_pattern
                    .clone()
                    .ok_or("secondary_input_sd_csv_path_pattern must be set")?,
                seeds_to_use_str,
                betas_to_use_str,
            ]),
            Some(overwrites),
        )
    }
}

#[typetag::serde]
impl Module for ReformatOriginalExtractedMeasurements {
    fn run(&self, config: &Config) -> Result<(), String> {
        println!("Running ReformatOriginalExtractedMeasurements module.\n");

        let bash_function = BashFunction::new(
            "reformat_original_java_extracted_measurements_case",
            "./experiments/compare_braess_to_java/runnable_modules/reformat_original_measurements.sh",
        );

        // this module doesn't use any rust seeds, so we overwrite the rust_seeds variable to
        // contain only one parameter, so that the bash function is only called once per cartesian
        // product of the other parameters
        let rust_seed_decl_overwrite = "rust_seeds=(None)".to_string();
        let overwrites =
            HashMap::from_iter([("rust_seeds".to_string(), rust_seed_decl_overwrite.clone())]);

        bash_function.run_for_cartprod_in_parallel(
            config,
            Some(vec![
                self.original_tt_tsv_file_pattern
                    .clone()
                    .ok_or("original_tt_tsv_file_pattern must be set")?,
                self.original_sd_tsv_file_pattern
                    .clone()
                    .ok_or("original_sd_tsv_file_pattern must be set")?,
                self.reformatted_tt_csv_file_pattern
                    .clone()
                    .ok_or("reformatted_tt_csv_file_pattern must be set")?,
                self.reformatted_sd_csv_file_pattern
                    .clone()
                    .ok_or("reformatted_sd_csv_file_pattern must be set")?,
            ]),
            Some(overwrites),
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SeedsToAvgOver {
    RustSeeds,
    JavaSeedIndices,
}

mod tests {
    use super::*;
    use serde_yaml::Value;

    #[test]
    fn test_shell_execution() {
        let config = Config {
            globals: Some(HashMap::from([
                (
                    "experiment_set_name".to_string(),
                    Value::String("test_set".to_string()),
                ),
                (
                    "base_output_dir".to_string(),
                    Value::String("/tmp/output".to_string()),
                ),
                (
                    "experiment_output_dir_pattern".to_string(),
                    Value::String(
                        "{base_output_dir}/{replanning_variant}/{experiment_set_name}/beta{beta}/read_from_random_{read_from_random}_use_random_seed_{use_random_seed}".to_string(),
                    ),
                ),
                (
                    "delete_output_dir_if_existing".to_string(),
                    Value::Bool(true),
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
                    vec![Value::Number(1.into()), Value::Number(20.into()), Value::Number(4.into())],
                ),
                ("rust_seeds".to_string(), vec![Value::Number(42.into()), Value::Number(43.into())]),
            ]),
            modules: vec![
                Box::new(RunRustBasedOnJavaOutput {
                    overwrite_mode: OverwriteFiles::DeleteDirectoryIfExists,
                }),
                Box::new(
                    ExtractMeasurementsFromEvents {
                        input_file_stem_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/beta{beta}/read_from_random_{read_from_random}_use_random_seed_{use_random_seed}/events/events".to_string()),
                        input_file_format: Some("binpb".to_string()),
                        tt_csv_output_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                        sd_csv_output_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                        num_parts: Some(1)
                    }),
                Box::new(PlotTtAndSdPerSeed {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/per_seed/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/per_seed/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.pdf".to_string()),
                    input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                }),
                Box::new(AddDummyCoordinatesToEvents {
                    input_file_stem_pattern: Some("/home/andreas/RustroverProjects/runs-svn/braess/refinement/no_spillback_scenario/2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it/beta{beta}/random{read_from_random}/beta{beta}random{read_from_random}.output_events.xml.gz".to_string()),
                    output_file_stem_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/event_files_with_dummy_coordinates/beta{beta}/random{read_from_random}/beta{beta}random{read_from_random}.output_events.xml.gz".to_string()),
                }),
                // extract measurements from the (original java) event files with dummy coordinates
                Box::new(
                    ExtractMeasurementsFromEvents {
                        input_file_stem_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/event_files_with_dummy_coordinates/beta{beta}/random{read_from_random}/beta{beta}random{read_from_random}.output_events".to_string()),
                        input_file_format: Some("xml.gz".to_string()),
                        tt_csv_output_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_original_java_data_new_extraction.csv".to_string()),
                        sd_csv_output_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_original_java_data_new_extraction.csv".to_string()),
                        num_parts: Some(0)
                    }
                ),
                // reformat the original Java extracted measurements
                Box::new(ReformatOriginalExtractedMeasurements {
                    original_tt_tsv_file_pattern: Some("/home/andreas/RustroverProjects/runs-svn/braess/refinement/no_spillback_scenario/2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it/analysis/average_traveltimes/avgRouteTTsPerDeparture_{beta}_{read_from_random}_500.txt".to_string()),
                    original_sd_tsv_file_pattern: Some("/home/andreas/RustroverProjects/runs-svn/braess/refinement/no_spillback_scenario/2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it/analysis/summed_departures/summedDeparturesPerRoute_{beta}_{read_from_random}_500.txt".to_string()),
                    reformatted_tt_csv_file_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    reformatted_sd_csv_file_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                }),
                Box::new(PlotTtAndSdPerSeed {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/plots/per_seed/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/plots/per_seed/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.pdf".to_string()),
                    input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                }),
                Box::new(PlotTtAndSdAvgdOverSeeds {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/avg_over_rust_seeds/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_from_random}.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/avg_over_rust_seeds/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{read_from_random}.pdf".to_string()),
                    input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::RustSeeds,
                }),
                Box::new(PlotTtAndSdDeviationToNashBoxplotsOverBeta {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/deviations/tt_avg_deviation_boxplots/tt_avg_deviation_boxplot.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/deviations/sd_avg_deviation_boxplots/sd_avg_deviation_boxplot.pdf".to_string()),
                    input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::RustSeeds,
                }),
                Box::new(PlotTtAndSdDeviationToNashScatterplotsOverBeta {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/deviations/tt_avg_first_deviation_scatterplots/tt_avg_first_deviation_scatterplot.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/deviations/sd_avg_first_deviation_scatterplots/sd_avg_first_deviation_scatterplot.pdf".to_string()),
                    input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::RustSeeds,
                }),
                // plot deviation of original java data
                Box::new(PlotTtAndSdDeviationToNashBoxplotsOverBeta {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/plots/deviations/tt_avg_deviation_boxplots/tt_avg_deviation_boxplot.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/plots/deviations/sd_avg_deviation_boxplots/sd_avg_deviation_boxplot.pdf".to_string()),
                    input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::JavaSeedIndices,
                }),
                Box::new(PlotTtAndSdDeviationToNashScatterplotsOverBeta {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/plots/deviations/tt_avg_first_deviation_scatterplots/tt_avg_first_deviation_scatterplot.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/plots/deviations/sd_avg_first_deviation_scatterplots/sd_avg_first_deviation_scatterplot.pdf".to_string()),
                    input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::JavaSeedIndices,
                }),
                // Plot difference of rust data to the original data - per seed
                Box::new(PlotTtAndSdDiffPerSeed {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/per_seed/tt_per_path_over_deptime_minus_original_java/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/per_seed/sd_per_path_over_time_minus_original_java/sd_per_path_over_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.pdf".to_string()),
                    main_input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    main_input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    secondary_input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    secondary_input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                }),
                // Plot difference of rust data to the original data - averaged over seeds
                Box::new(PlotTtAndSdAvgdOverSeedsDiff {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/avg_over_rust_seeds/tt_per_path_over_deptime_minus_original_java/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_from_random}.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/avg_over_rust_seeds/sd_per_path_over_time_minus_original_java/sd_per_path_over_time_beta{beta}_read_from_random_{read_from_random}.pdf".to_string()),
                    main_input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    main_input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    secondary_input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    secondary_input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::RustSeeds,
                }),
                // Plot difference of rust data to the original data as a boxplot
                Box::new(PlotTtAndSdDeviationToDiffRunBoxplotsOverBeta {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/deviations/tt_avg_deviation_boxplots_rust_minus_java/tt_avg_deviation_boxplot.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/deviations/sd_avg_deviation_boxplots_rust_minus_java/sd_avg_deviation_boxplot.pdf".to_string()),
                    main_input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    main_input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    secondary_input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    secondary_input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::RustSeeds,
                }),
                // Plot difference of rust data (avgd first) to the original data as a scatterplot
                Box::new(PlotTtAndSdDeviationToDiffRunScatterplotsOverBeta {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/deviations/tt_avg_first_deviation_scatterplots_rust_minus_java/tt_avg_first_deviation_scatterplot.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/deviations/sd_avg_first_deviation_scatterplots_rust_minus_java/sd_avg_first_deviation_scatterplot.pdf".to_string()),
                    main_input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    main_input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    secondary_input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    secondary_input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/temp_recreating_java/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_reformatted_original_java_data.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::RustSeeds,
                }),
                // Run a last iteration in java based on the original output plans
                Box::new(RunJavaSingleIterBasedOnJavaOutput {
                    overwrite_mode: OverwriteFiles::DeleteDirectoryIfExists,
                    config_file_pattern: Some("{base_dir_to_read_from}/{replanning_variant}/beta{beta}/random{read_from_random}/beta{beta}random{read_from_random}.output_config.xml".to_string()),
                    base_dir_to_read_from: Some("./../runs-svn/braess/refinement/no_spillback_scenario".to_string()),
                }
                )
            ],
            global_config: None,
        };

        // TODO continue here: I need run single java iteration module.
        //  and then finally: make the config readable

        // TODO the run_all should handle print_failure_summary etc.
        // TODO also there should be failure kinds for every module. Can this be automatic?
        config.run_all_modules().expect("Failed to run all modules");

        // TODO is it reasonable that I cannot change the experiment output dir pattern for the java run?
        //  No, it makes sense that there is a default, but there might be situations where you
        //  want to do several runs (rust/rust, rust/java, java/java) in one experiment set (e.g.
        //  when comparing them), and this would require different output dir patterns.
        //  So I should make this configurable, but with a default.
        //  QUESTION: How do I handle defaults? somehow in the global file but still.

        // TODO also related: move delete-output-dir-if-existing to run modules, I think.
        //  generally think about how I can group things by what parameters they need, maybe.
        let config_java = Config {
            globals: Some(HashMap::from([
                (
                    "experiment_set_name".to_string(),
                    Value::String("test_set_run_java".to_string()),
                ),
                (
                    "base_output_dir".to_string(),
                    Value::String("/tmp/output".to_string()),
                ),
                (
                    "experiment_output_dir_pattern".to_string(),
                    Value::String(
                        "{base_output_dir}/{replanning_variant}/{experiment_set_name}/beta{beta}/read_from_random_{read_from_random}_use_random_seed_{use_random_seed}".to_string(),
                    ),
                ),
                (
                    "delete_output_dir_if_existing".to_string(),
                    Value::Bool(true),
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
                    vec![Value::Number(1.into()), Value::Number(20.into()), Value::Number(4.into())],
                ),
                ("rust_seeds".to_string(), vec![Value::Number(42.into()), Value::Number(43.into())]),
            ]),
            modules: vec![
                // Run a last iteration in java based on the original output plans
                Box::new(RunJavaSingleIterBasedOnJavaOutput {
                    overwrite_mode: OverwriteFiles::DeleteDirectoryIfExists,
                    config_file_pattern: Some("{base_dir_to_read_from}/{replanning_variant}/beta{beta}/random{read_from_random}/beta{beta}random{read_from_random}.output_config.xml".to_string()),
                    base_dir_to_read_from: Some("./../runs-svn/braess/refinement/no_spillback_scenario".to_string()),
                }
                )
            ],
            global_config: None,
        };

        config_java
            .run_all_modules()
            .expect("Failed to run all modules");
    }

    #[test]
    fn test_config_reading() {
        // Test the config reading functionality

        let parsed_config: Config = serde_yaml::from_reader(
            std::fs::File::open(
                "./../experiments/compare_braess_to_java/experiment_sets/test_config.yaml",
            )
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
                (
                    "experiment_output_dir_pattern".to_string(),
                    Value::String(
                        "{base_output_dir}/{replanning_variant}/{experiment_set_name}/beta{beta}/read_from_random_{read_from_random}_use_random_seed_{use_random_seed}".to_string(),
                    ),
                ),
                (
                    "delete_output_dir_if_existing".to_string(),
                    Value::Bool(true),
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
                    vec![Value::Number(1.into()), Value::Number(20.into()), Value::Number(4.into())],
                ),
                ("rust_seeds".to_string(), vec![Value::Number(42.into()), Value::Number(43.into())]),
            ]),
            modules: vec![
                Box::new(RunRustBasedOnJavaOutput {
                    overwrite_mode: OverwriteFiles::DeleteDirectoryIfExists,
                }),
                Box::new(PlotTtAndSdAvgdOverSeeds {
                    output_tt_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/avg_over_rust_seeds/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_from_random}.pdf".to_string()),
                    output_sd_plot_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/avg_over_rust_seeds/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{read_from_random}.pdf".to_string()),
                    input_tt_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    input_sd_csv_path_pattern: Some("{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/extracted_data/summed_deps_per_time_beta{beta}_read_from_random_{read_from_random}_use_random_seed_{use_random_seed}.csv".to_string()),
                    seeds_to_avg_over: SeedsToAvgOver::RustSeeds,
                }),
            ],
            global_config: None,
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

        assert_eq!(
            parsed_config.global_config, expected_config.global_config,
            "Parsed global config does not match expected config global config"
        );
    }
}
