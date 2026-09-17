use crate::config::Config;
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

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
}

impl BashFunction {
    pub fn new(function_name: &str, path_to_function_def: impl AsRef<Path>) -> Self {
        Self {
            function_name: function_name.to_string(),
            path_to_function_def: PathBuf::from(path_to_function_def.as_ref()),
        }
    }

    /// run the bash function for all combinations of the parameters in config.param_sweep, in
    /// parallel, using the run_callback_for_parameter_arrays_parallel function defined in
    /// common_code_for_modules.sh (which in turn uses GNU parallel)
    pub fn run_for_cartprod_in_parallel(
        &self,
        config: &Config,
        extra_str_args: Option<Vec<String>>,
        param_sweep_overwrites: Option<HashMap<String, String>>,
    ) -> Result<(), String> {
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
        let java_seed_decl = {
            if let Some(overwrite) = param_sweep_overwrites
                .as_ref()
                .and_then(|m| m.get("java_seed_indices"))
            {
                overwrite.clone()
            } else {
                config.get_sweep_array_decl("java_seed_indices", "java_seed_indices")?
            }
        };
        let rust_seed_decl = {
            if let Some(overwrite) = param_sweep_overwrites
                .as_ref()
                .and_then(|m| m.get("rust_seeds"))
            {
                overwrite.clone()
            } else {
                config.get_sweep_array_decl("rust_seeds", "rust_seeds")?
            }
        };

        // globals/global_config -> Parameters 5-9
        let experiment_set_name = config
            .get_global_parameter("experiment_set_name")?
            .as_str()
            .ok_or("experiment_set_name must be string")?;
        let base_output_dir = config
            .get_global_parameter("base_output_dir")?
            .as_str()
            .ok_or("base_output_dir must be string")?;
        let experiment_output_dir_pattern = config
            .get_global_parameter("experiment_output_dir_pattern")?
            .as_str()
            .ok_or("experiment_output_dir_pattern must be string")?;
        let delete_output_dir_if_existing = config
            .get_global_parameter("delete_output_dir_if_existing")?
            .as_bool()
            .ok_or("delete_output_dir_if_existing must be bool")?;
        let max_parallel_jobs = config
            .get_global_parameter("max_parallel_jobs")?
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
{java_seed_decl}
{rust_seed_decl}

source {common}
source {module}

run_callback_for_parameter_arrays_parallel \
  replanning_variants \
  betas \
  java_seed_indices \
  rust_seeds \
  {experiment_set_name} \
  {base_output_dir} \
  {experiment_output_dir_pattern} \
  {delete_output_dir_if_existing} \
  {max_parallel_jobs} \
  {function_name} \
  {extra_args_escaped}

printf "\n\nAll experiments completed.\n"
print_failure_summary
"#,
            replanning_decl = replanning_decl,
            betas_decl = betas_decl,
            java_seed_decl = java_seed_decl,
            rust_seed_decl = rust_seed_decl,
            common = shell_escape_single_quoted(common),
            module = shell_escape_single_quoted(module),
            experiment_set_name = shell_escape_single_quoted(experiment_set_name),
            base_output_dir = shell_escape_single_quoted(base_output_dir),
            experiment_output_dir_pattern =
                shell_escape_single_quoted(experiment_output_dir_pattern),
            delete_output_dir_if_existing = if delete_output_dir_if_existing {
                "true"
            } else {
                "false"
            },
            max_parallel_jobs = max_parallel_jobs,
            extra_args_escaped = extra_args_escaped,
            function_name = function_name
        );

        let status = Command::new("bash")
            .current_dir("./../")
            // set the working directory to the root of the project, so that the relative paths in the bash script work
            .arg("-lc")
            .arg(bash_script)
            .status()
            .map_err(|e| format!("Failed to start bash: {e}"))?;

        if status.success() {
            Ok(())
        } else {
            println!("Bash pipeline failed with status {status}");
            // Ok(())
            Err(format!("Bash pipeline failed with status {status}"))
        }
    }
}
