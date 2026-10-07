//! Startup diagnostics: a log file, a crash report and – on Windows, where release builds have
//! no console – an error dialog, so that Elora never quits silently (playtest: “nothing
//! happened” on a Windows PC).

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
            "Elora {} ({})\n\n{message}",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS
        );
    }
    let first_line = message.lines().next().unwrap_or(message);
    show_error(&format!(
        "Elora konnte nicht starten.\nElora could not start.\n\n{first_line}\n\n{}\n{}",
        crash.display(),
        file_in_config(LOG_FILE).display()
    ));
}

#[cfg(windows)]
fn show_error(text: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let (text, title) = (wide(text), wide("Elora"));
    // SAFETY: both strings are valid, zero-terminated UTF-16 buffers that outlive the call.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
fn show_error(text: &str) {
    eprintln!("{text}");
}
