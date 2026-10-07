//! Startup diagnostics: a log file, a crash report and – on Windows, where release builds have
//! no console – the report opened in Notepad, so that Elora never quits silently (playtest:
//! “nothing happened” on a Windows PC). No `unsafe` and no extra dependency.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use tracing_subscriber::fmt::writer::MakeWriterExt;

/// Log of the current run, next to the settings (`%APPDATA%\Elora` on Windows).
const LOG_FILE: &str = "elora.log";
/// Written when Elora panics or cannot start.
const CRASH_FILE: &str = "crash.txt";

fn file_in_config(name: &str) -> PathBuf {
    let path = crate::settings::config_file(name);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    path
}

/// Logs to stderr and to [`LOG_FILE`] (overwritten on every start).
pub fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,wgpu_core=warn,wgpu_hal=warn".into());
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false);
    match std::fs::File::create(file_in_config(LOG_FILE)) {
        Ok(file) => builder
            .with_writer(std::io::stderr.and(Mutex::new(file)))
            .init(),
        Err(_) => builder.with_writer(std::io::stderr).init(),
    }
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        "Elora starting"
    );
}

/// On a panic: keep the default output, write a crash report and show it.
pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default(info);
        let backtrace = std::backtrace::Backtrace::force_capture();
        report_fatal(&format!("{info}\n\n{backtrace}"));
    }));
}

/// Error that ends the program: log it, write [`CRASH_FILE`] and tell the player.
pub fn report_fatal(message: &str) {
    tracing::error!("{message}");
    let crash = file_in_config(CRASH_FILE);
    if let Ok(mut f) = std::fs::File::create(&crash) {
        let _ = writeln!(
            f,
            "Elora wurde wegen eines Fehlers beendet. / Elora stopped because of an error.\n\
             Elora {} ({})\n\n{message}",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS
        );
    }
    show_report(&crash, message);
}

/// Windows release builds have no console: open the report in Notepad.
#[cfg(windows)]
fn show_report(crash: &std::path::Path, _message: &str) {
    let _ = std::process::Command::new("notepad").arg(crash).spawn();
}

#[cfg(not(windows))]
fn show_report(crash: &std::path::Path, message: &str) {
    let first_line = message.lines().next().unwrap_or(message);
    eprintln!(
        "Elora wurde wegen eines Fehlers beendet. / Elora stopped because of an error.\n\n{first_line}\n\n{}\n{}",
        crash.display(),
        file_in_config(LOG_FILE).display()
    );
}
