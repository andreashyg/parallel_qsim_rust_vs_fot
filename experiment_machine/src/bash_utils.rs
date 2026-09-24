use crate::config::{Config, GlobalConfig, Module};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Write},
    process::{ExitStatus, Stdio},
    sync::{Arc, Mutex},
    thread,
};
use tracing::{error, info};

/// Init: start tracing to a file (keep the guard alive for the program lifetime)
pub fn init_tracing(
    log_file_path: impl AsRef<Path>,
) -> tracing_appender::non_blocking::WorkerGuard {
    // use this if you want to write to a rolling log file, e.g. one per day
    // let file_appender = tracing_appender::rolling::never("logs", "run.log");

    // we will instead write to a single log file
    if let Some(parent) = log_file_path.as_ref().parent() {
        create_dir_all(parent).expect("create log parent dir");
    }
    let log_file = File::create(log_file_path).expect("create log file");
    let (non_blocking, guard) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(tracing::Level::INFO)
        .init();
    guard
}

/// Run one external step with live tee'ing to console + tracing
///
/// # Arguments
/// - name: The name of the step, will be used when printing to console and tracing
/// - program: The program to execute
/// - args: The arguments to pass to the program
pub fn run_step(
    name: &str,
    program: &str,
    args: &[&str],
    // global_log: Arc<Mutex<File>>,
    // global_err_log: Arc<Mutex<File>>,
    // per_step_path: Option<PathBuf>,
) -> Result<ExitStatus, Box<dyn Error>> {
    // // optional per-step writer wrapped for shared access
    // if let Some(ref p) = per_step_path {
    //     if let Some(parent) = p.parent() {
    //         create_dir_all(parent).expect("create per-step log parent dir");
    //     }
    // };
    //
    // let per_step_writer =
    //     per_step_path.map(|p| Arc::new(Mutex::new(File::create(p).expect("create per-step log"))));

    info!(step = %name, "starting to run step");

    // spawn the child process (this is where the command is actually executed)
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // take pipes (this is where we can read the output of the command)
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    // shared clones for threads
    // let g1 = Arc::clone(&global_log);
    // let p1 = per_step_writer.as_ref().map(Arc::clone);
    let name_out = name.to_string();

    // spawn threads to read stdout and stderr
    let out_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            // live console
            println!("[{}][OUT] {}", name_out, line);
            // tracing
            info!(step = %name_out, stream = "stdout", %line);

            // // append to global log (best-effort)
            // if let Ok(mut g) = g1.lock() {
            //     let _ = writeln!(g, "[{}][OUT] {}", name_out, line);
            //     let _ = g.flush();
            // }
            // append to per-step log (best-effort)
            // if let Some(pw) = &p1 {
            //     if let Ok(mut p) = pw.lock() {
            //         let _ = writeln!(p, "[{}][OUT] {}", name_out, line);
            //         let _ = p.flush();
            //     }
            // }
        }
    });

    // same thing for stderr
    // let g2 = Arc::clone(&global_log);
    // let p2 = per_step_writer.as_ref().map(Arc::clone);
    let name_err = name.to_string();
    let err_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            eprintln!("[{}][ERR] {}", name_err, line);
            // this is a hack to ignore the progress bar from GNU parallel, which starts with
            // \r (carriage return) and contains # and % characters. We don't want to log these
            // lines, as they are not useful and clutter the logs.
            if line.starts_with("\r") && line.contains("#") && line.contains("%") {
                continue;
            }
            error!(step = %name_err, stream = "stderr", %line);
            // if let Ok(mut g) = g2.lock() {
            //     let _ = writeln!(g, "[{}][ERR] {}", name_err, line);
            //     let _ = g.flush();
            // }
            // if let Some(pw) = &p2 {
            //     if let Ok(mut p) = pw.lock() {
            //         let _ = writeln!(p, "[{}][ERR] {}", name_err, line);
            //         let _ = p.flush();
            //     }
            // }
        }
    });

    // wait for process
    let status = child.wait()?;

    // ensure readers finished
    let _ = out_handle.join();
    let _ = err_handle.join();

    // log exit
    if status.success() {
        info!(step = %name, "step finished successfully");
    } else {
        error!(step = %name, code = ?status.code(), "step failed");
    }

    Ok(status)
}
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
        global_config: &GlobalConfig,
        module: &dyn Module,
        extra_str_args: Option<Vec<String>>,
        param_sweep_overwrites: Option<HashMap<String, String>>,
        // global_log_file: Arc<Mutex<File>>,
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
            experiment_set_name = shell_escape_single_quoted(&experiment_set_name),
            base_output_dir = shell_escape_single_quoted(&base_output_dir),
            max_parallel_jobs = max_parallel_jobs,
            extra_args_escaped = extra_args_escaped,
            function_name = function_name
        );

        let status = run_step(
            function_name,
            "bash",
            &["-lc", &bash_script],
            // global_log_file,
            // Some(PathBuf::from(format!(
            //     "{base_output_dir}/test_logs/{}_step.log",
            //     function_name
            // ))),
        )
        .map_err(|e| {
            format!(
                "Failed to run bash function {} in parallel: {e}",
                function_name
            )
        })?;

        // let output = Command::new("bash")
        //     // .current_dir("./../")
        //     // set the working directory to the root of the project, so that the relative paths in the bash script work
        //     .arg("-lc")
        //     .arg(bash_script)
        //     .output()
        //     .map_err(|e| format!("Failed to start bash: {e}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "Bash function {} failed with status {}.",
                function_name, status
            ))
        }

        // if output.status.success() {
        //     // Print the output of the bash command
        //     println!(
        //         "Bash command output: {}",
        //         String::from_utf8_lossy(&output.stdout)
        //     );
        //     println!(
        //         "Bash command stderr: {}",
        //         String::from_utf8_lossy(&output.stderr)
        //     );
        //     Ok(())
        // } else {
        //     Err(format!(
        //         "Bash pipeline failed with status {}. stderr: {}",
        //         output.status,
        //         String::from_utf8_lossy(&output.stderr)
        //     ))
        // }
    }
}
