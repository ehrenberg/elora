//! Mitgelieferte Daten finden (M8.2), egal aus welchem Ordner das Programm startet.
//!
//! Ein relativer Pfad (z. B. `maps/dm-wiese.emap`) gilt zuerst im Arbeitsverzeichnis (Entwicklung
//! mit `cargo run`), sonst im Datenordner: `ELORA_DATA`, neben dem Programm, `../share/elora`
//! (Linux, `AppImage`) oder `../Resources` (macOS-Bundle) – der erste mit einem Ordner `maps`.

use std::path::{Path, PathBuf};

/// Umgebungsvariable für einen eigenen Datenordner.
pub const DATA_ENV: &str = "ELORA_DATA";

fn candidates() -> Vec<PathBuf> {
    let mut list = Vec::new();
    if let Some(d) = std::env::var_os(DATA_ENV).filter(|v| !v.is_empty()) {
        list.push(PathBuf::from(d));
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        list.push(dir.to_path_buf());
        list.push(dir.join("../share/elora"));
        list.push(dir.join("../Resources"));
    }
    list
}

/// Datenordner der Installation (mit `maps/`), sonst das Arbeitsverzeichnis.
pub fn data_dir() -> PathBuf {
    candidates()
        .into_iter()
        .find(|d| d.join("maps").is_dir())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Relativen Pfad auflösen: erst im Arbeitsverzeichnis, dann im Datenordner.
pub fn resolve(path: &Path) -> PathBuf {
    if path.is_absolute() || path.exists() {
        return path.to_path_buf();
    }
    data_dir().join(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_and_existing_paths_stay() {
        let abs = std::env::temp_dir();
        assert_eq!(resolve(&abs), abs);
        // Tests laufen im Crate-Ordner: Cargo.toml liegt dort
        assert_eq!(
            resolve(Path::new("Cargo.toml")),
            PathBuf::from("Cargo.toml")
        );
        // Unbekanntes fällt auf den Datenordner zurück
        assert!(resolve(Path::new("gibt/es/nicht")).ends_with("gibt/es/nicht"));
    }
}
