use crate::config::{Config, GlobalConfig, Module};
use crate::logging::run_step_while_tracing;
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn shell_escape_single_quoted(s: &str) -> String {
    // 'abc' -> 'abc',  a'b -> 'a'"'"'b'
    format!("'{}'", s.replace('\'', r#"'"'"'"#))
}

// Helpers
pub fn yaml_value_as_shell_atom(v: &Value) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(shell_escape_single_quoted(s)),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(if *b { "true".into() } else { "false".into() }),
        _ => Err(format!("Unsupported YAML value for shell arg: {v:?}")),
    }
}

pub struct BashFunction {
    function_name: String,
    path_to_function_def: PathBuf,
    script_type: String,
}

impl BashFunction {
    pub fn new(
        function_name: &str,
        path_to_function_def: impl AsRef<Path>,
        script_type: impl Into<String>,
    ) -> Self {
        Self {
            function_name: function_name.to_string(),
            path_to_function_def: PathBuf::from(path_to_function_def.as_ref()),
            script_type: script_type.into(),
        }
    }

    /// run the bash function for all combinations of the parameters in config.param_sweep, in
    /// parallel, using the run_callback_for_parameter_arrays_parallel function defined in
    /// common_code_for_modules.sh (which in turn uses GNU parallel)
    /// # Arguments:
    /// - config: the Config object containing the parameter sweep definitions
    /// - global_config: the GlobalConfig object containing the global parameters
    /// - module: the Module object containing the module-specific parameters and overwrites
    /// - resume: if true, only run experiments that have not yet been run. Can be overwritten by
    ///     the module's overwrites
    /// - resume_failed: if true, only run experiments that have failed or have not been run. Can be
    ///     overwritten by the module's overwrites
    /// - extra_str_args: optional extra string arguments to pass to the bash function
    /// - param_sweep_overwrites: optional overwrites for the parameter sweep arrays, in the form
    ///     of a HashMap from parameter name to string representation of the array
    pub fn run_for_cartprod_in_parallel<M: Module + ?Sized>(
        &self,
        config: &Config,
        _global_config: &GlobalConfig,  // keeping this unused parameter for future use
        module: &M,
        resume: bool,
        resume_failed: bool,
        extra_str_args: Option<Vec<String>>,
        param_sweep_overwrites: Option<HashMap<String, String>>,
    ) -> Result<(), String> {
        // if the module has an overwrite for resume, use that instead of the passed in value
        let resume = if let Some(resume_overwrite) = module.get_expset_config_overwrites()
            .as_ref()
            .and_then(|m| m.get("resume"))
        {
            resume_overwrite.as_bool().expect(&format!(
                "Failed to parse resume overwrite value {:?} as bool",
                resume_overwrite
            ))
        } else {
            resume
        };

        // if the module has an overwrite for resume_failed, use that instead of the passed in value
        let resume_failed = if let Some(resume_failed_overwrite) = module.get_expset_config_overwrites()
            .as_ref()
            .and_then(|m| m.get("resume_failed"))
        {
            resume_failed_overwrite.as_bool().expect(&format!(
                "Failed to parse resume_failed overwrite value {:?} as bool",
                resume_failed_overwrite
            ))
        } else {
            resume_failed
        };

        // param_sweep -> first four parameters of the bash function
        // we need to transform the arrays into bash array declarations
        let replanning_decl = {
            if let Some(overwrite) = param_sweep_overwrites
                .as_ref()
                .and_then(|m| m.get("replanning_variants"))
            {
                overwrite.clone()
            } else {
                config.get_sweep_array_decl("replanning_variants", "replanning_variants")?
            }
        };

        let betas_decl = {
            if let Some(overwrite) = param_sweep_overwrites.as_ref().and_then(|m| m.get("betas")) {
                overwrite.clone()
            } else {
                config.get_sweep_array_decl("betas", "betas")?
            }
        };
        let read_from_random_seed_decl = {
            if let Some(overwrite) = param_sweep_overwrites
                .as_ref()
                .and_then(|m| m.get("read_from_random_seed_indices"))
            {
                overwrite.clone()
            } else {
                config.get_sweep_array_decl(
                    "read_from_random_seed_indices",
                    "read_from_random_seed_indices",
                )?
            }
        };
        let use_random_seed_decl = {
            if let Some(overwrite) = param_sweep_overwrites
                .as_ref()
                .and_then(|m| m.get("use_random_seeds"))
            {
                overwrite.clone()
            } else {
                config.get_sweep_array_decl("use_random_seeds", "use_random_seeds")?
            }
        };

        // globals/global_config -> Parameters 5-9

        let experiment_set_name = module
            .get_expset_config_global_parameter_or_overwrite(config, "experiment_set_name")?
            .as_str()
            .ok_or("experiment_set_name must be string")?
            .to_string();
        let base_output_dir = module
            .get_expset_config_global_parameter_or_overwrite(config, "base_output_dir")?
            .as_str()
            .ok_or("base_output_dir must be string")?
            .to_string();
        let max_parallel_jobs = module
            .get_expset_config_global_parameter_or_overwrite(config, "max_parallel_jobs")?
            .as_i64()
            .ok_or("max_parallel_jobs must be integer")?;

        // transform all extra args into one string, with each arg shell-escaped and separated by space
        let extra_args_escaped = if let Some(extra_str_args) = extra_str_args {
            extra_str_args
                .iter()
                .map(|s| shell_escape_single_quoted(s))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        };

        let function_name = &self.function_name;
        let script_type = &self.script_type;

        let common =
            "./experiments/compare_braess_to_java/runnable_modules/common_code_for_modules.sh";
        let module = self
            .path_to_function_def
            .to_str()
            .ok_or("Failed to convert module path to string")?;

        let bash_script = format!(
            r#"
set -uo pipefail
{replanning_decl}
{betas_decl}
{read_from_random_seed_decl}
{use_random_seed_decl}

source {common}
source {module}

run_callback_for_parameter_arrays_parallel \
  replanning_variants \
  betas \
  read_from_random_seed_indices \
  use_random_seeds \
  {experiment_set_name} \
  {base_output_dir} \
  {max_parallel_jobs} \
  {resume} \
  {resume_failed} \
  {script_type} \
  {function_name} \
  {extra_args_escaped}

printf "\n\nAll experiments completed.\n"
print_failure_summary
"#,
            replanning_decl = replanning_decl,
            betas_decl = betas_decl,
            read_from_random_seed_decl = read_from_random_seed_decl,
            use_random_seed_decl = use_random_seed_decl,
            common = shell_escape_single_quoted(common),
            module = shell_escape_single_quoted(module),
            experiment_set_name = shell_escape_single_quoted(&experiment_set_name),
            base_output_dir = shell_escape_single_quoted(&base_output_dir),
            max_parallel_jobs = max_parallel_jobs,
            resume = resume,
            resume_failed = resume_failed,
            script_type = script_type,
            extra_args_escaped = extra_args_escaped,
            function_name = function_name
        );

        let status = run_step_while_tracing(
            function_name,
            "bash",
            &["-lc", &bash_script],
        )
            .map_err(|e| {
                format!(
                    "Failed to run bash function {} in parallel: {e}",
                    function_name
                )
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "Bash function {} failed with status {}.",
                function_name, status
            ))
        }
    }
}
