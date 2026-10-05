use std::error::Error;
use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use tracing::{error, info};
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

/// Init: start tracing to a file (keep the guard alive for the program lifetime)
pub fn init_tracing(
    main_log_file_path: impl AsRef<Path>,
    error_log_file_path: impl AsRef<Path>,
    resume: bool,
    resume_failed: bool,
    overwrite_and_append: bool,
) -> (
    tracing_appender::non_blocking::WorkerGuard,
    tracing_appender::non_blocking::WorkerGuard,
) {
    // use this if you want to write to a rolling log file, e.g. one per day
    // let file_appender = tracing_appender::rolling::never("logs", "run.log");

    // we will instead write to a single log file
    if let Some(parent) = main_log_file_path.as_ref().parent() {
        create_dir_all(parent).expect("create log parent dir");
    }
    let main_log_file = if (resume || resume_failed || overwrite_and_append)
        && main_log_file_path.as_ref().exists()
    {
        File::options()
            .append(true)
            .open(main_log_file_path)
            .expect("opening main log file failed")
    } else {
        File::create(main_log_file_path).expect("create log file")
    };

    let (main_writer, main_guard) = tracing_appender::non_blocking(main_log_file);

    // separate error log file
    if let Some(parent) = error_log_file_path.as_ref().parent() {
        create_dir_all(parent).expect("create error log parent dir");
    }
    let error_log_file = if (resume || resume_failed || overwrite_and_append)
        && error_log_file_path.as_ref().exists()
    {
        File::options()
            .append(true)
            .open(error_log_file_path)
            .expect("opening error log file failed")
    } else {
        File::create(error_log_file_path).expect("create error log file")
    };
    let (error_writer, error_guard) = tracing_appender::non_blocking(error_log_file);

    let main_layer = layer()
        .with_writer(main_writer)
        // .with_ansi(false)  // disable ANSI colors in the log file
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    let error_layer = layer()
        .with_writer(error_writer)
        // .with_ansi(false)
        .with_filter(tracing_subscriber::filter::LevelFilter::ERROR);

    let subscriber = Registry::default().with(main_layer).with(error_layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    if resume {
        error!(
            "********** Resuming, running not yet run experiments, appending to existing log files **********"
        );
    }
    if resume_failed {
        error!(
            "********** Resuming, running failed and not yet run experiments, appending to existing log files **********"
        );
    }
    if overwrite_and_append {
        error!(
            "********** New run appended to log file! Output above this line may be overwritten below **********"
        );
    }
    (main_guard, error_guard)
}

/// Run one external step with live tee'ing to console + tracing
///
/// # Arguments
/// - name: The name of the step, will be used when printing to console and tracing
/// - program: The program to execute
/// - args: The arguments to pass to the program
pub fn run_step_while_tracing(name: &str, program: &str, args: &[&str]) -> Result<ExitStatus, Box<dyn Error>> {
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
        }
    });

    // same thing for stderr
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