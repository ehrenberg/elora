//! Lokal hosten (E-060): Server-Konfiguration im Client bearbeiten, `elora-server`
//! als eigenen Prozess starten, stoppen oder weiterlaufen lassen.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::Context as _;
use elora_server::ServerConfig;

/// Konfigurationsdatei, die der Client für den Server schreibt.
pub const SERVER_CONFIG_FILE: &str = "server.toml";

#[derive(Debug)]
pub struct Hosting {
    pub config: ServerConfig,
    /// Server beim Beenden des Clients weiterlaufen lassen.
    pub keep_running: bool,
    process: Option<Child>,
    pub status: String,
}

impl Default for Hosting {
    fn default() -> Self {
        let config = ServerConfig::load(Path::new(SERVER_CONFIG_FILE)).unwrap_or_default();
        Self {
            config,
            keep_running: false,
            process: None,
            status: String::new(),
        }
    }
}

/// Pfad zu `elora-server` neben dem laufenden Client.
fn server_binary() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent().context("kein Programmverzeichnis")?;
    let path = dir.join(format!("elora-server{}", std::env::consts::EXE_SUFFIX));
    anyhow::ensure!(
        path.exists(),
        "{} nicht gefunden (einmal `cargo build` ausführen)",
        path.display()
    );
    Ok(path)
}

/// Kartenordner: mitgelieferte (`maps/`) und eigene aus dem Editor (E-152).
pub fn map_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("maps")];
    dirs.extend(crate::settings::user_maps_dir());
    dirs
}

/// Karten aus allen Kartenordnern; bei gleichem Namen gilt die eigene nicht doppelt.
pub fn available_maps() -> Vec<PathBuf> {
    let mut maps: Vec<PathBuf> = Vec::new();
    for dir in map_dirs() {
        let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == elora_map::EXTENSION))
            .filter(|p| !maps.iter().any(|m| m.file_name() == p.file_name()))
            .collect();
        found.sort();
        maps.extend(found);
    }
    maps
}

impl Hosting {
    /// Läuft der gestartete Server noch?
    pub fn is_running(&mut self) -> bool {
        match &mut self.process {
            Some(child) => match child.try_wait() {
                Ok(None) => true,
                Ok(Some(status)) => {
                    self.status = format!("Server beendet ({status})");
                    self.process = None;
                    false
                }
                Err(_) => false,
            },
            None => false,
        }
    }

    /// Speichert die Konfiguration und startet den Server. Liefert die Adresse zum Verbinden.
    ///
    /// # Errors
    /// Bei ungültiger Konfiguration oder fehlendem Server-Programm.
    pub fn start(&mut self) -> anyhow::Result<String> {
        self.config.validate()?;
        if self.is_running() {
            self.stop();
        }
        self.config.save(Path::new(SERVER_CONFIG_FILE))?;
        let child = Command::new(server_binary()?)
            .arg("--config")
            .arg(SERVER_CONFIG_FILE)
            .stdin(Stdio::null())
            .spawn()
            .context("Server konnte nicht gestartet werden")?;
        self.status = format!("Server läuft (PID {})", child.id());
        self.process = Some(child);
        Ok(format!("127.0.0.1:{}", self.config.port))
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
            self.status = "Server gestoppt".into();
        }
    }
}

impl Drop for Hosting {
    fn drop(&mut self) {
        if !self.keep_running {
            self.stop();
        }
    }
}
