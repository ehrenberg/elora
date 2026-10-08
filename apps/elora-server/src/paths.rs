//! Find the bundled data (M8.2), no matter from which folder the program starts.
//!
//! A relative path (e.g. `maps/dm-wiese.emap`) is looked up first in the working directory
//! (development with `cargo run`), otherwise in the data folder: `ELORA_DATA`, next to the
//! program, `../share/elora` (Linux, `AppImage`) or `../Resources` (macOS bundle) – the first
//! one with a `maps` folder.

use std::path::{Path, PathBuf};

/// Environment variable for a custom data folder.
pub const DATA_ENV: &str = "ELORA_DATA";

/// Folders searched for the game data, in this order.
pub fn candidates() -> Vec<PathBuf> {
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

/// Data folder of the installation (with `maps/`), otherwise the working directory.
pub fn data_dir() -> PathBuf {
    candidates()
        .into_iter()
        .find(|d| d.join("maps").is_dir())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Is the game data there at all (a `maps` folder in the working directory or in one of the
/// [`candidates`])? Without it Elora cannot start – usually the ZIP was not extracted.
pub fn data_found() -> bool {
    Path::new("maps").is_dir() || candidates().iter().any(|d| d.join("maps").is_dir())
}

/// Resolve a relative path: first in the working directory, then in the data folder.
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
        // tests run in the crate folder: Cargo.toml is there
        assert_eq!(
            resolve(Path::new("Cargo.toml")),
            PathBuf::from("Cargo.toml")
        );
        // unknown paths fall back to the data folder
        assert!(resolve(Path::new("gibt/es/nicht")).ends_with("gibt/es/nicht"));
    }
}
