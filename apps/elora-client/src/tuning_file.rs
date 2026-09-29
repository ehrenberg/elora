//! `tuning.toml`: gespeicherte Tuning-Werte der Sandbox (E-046) – Entwicklerwerkzeug.
//! Spieler-Einstellungen stehen in `settings.toml` ([`crate::settings`], E-116).
//!
//! Die Standardwerte stehen im Code (`Tuning::default`, `ViewSettings::default`);
//! die Datei überschreibt sie nur. Fehlende Einträge behalten ihren Standardwert.

use std::path::Path;

use anyhow::Context as _;
use elora_render::ViewSettings;
use elora_sim::Tuning;
use serde::{Deserialize, Serialize};

pub const TUNING_FILE: &str = "tuning.toml";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TuningFile {
    pub physics: Tuning,
    pub view: ViewFile,
}

/// Sichtbereich (E-045).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewFile {
    pub area: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl Default for ViewFile {
    fn default() -> Self {
        ViewSettings::default().into()
    }
}

impl From<ViewSettings> for ViewFile {
    fn from(v: ViewSettings) -> Self {
        Self {
            area: v.area,
            max_width: v.max_width,
            max_height: v.max_height,
        }
    }
}

impl From<ViewFile> for ViewSettings {
    fn from(v: ViewFile) -> Self {
        Self {
            area: v.area,
            max_width: v.max_width,
            max_height: v.max_height,
        }
    }
}

impl TuningFile {
    /// Lädt die Datei; existiert sie nicht, gelten die Standardwerte.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(src) => {
                toml::from_str(&src).with_context(|| format!("{} ist ungültig", path.display()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("{} nicht lesbar", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let body = toml::to_string_pretty(self)?;
        let text = format!(
            "# Elora – Sandbox-Tuning (E-046). Von der Sandbox geschrieben.\n\
             # Fehlende Einträge nutzen die Standardwerte aus docs/04-tuning.md.\n\n{body}"
        );
        std::fs::write(path, text).with_context(|| format!("{} nicht schreibbar", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut f = TuningFile::default();
        f.physics.hook_length = 420.0;
        f.view.area = 1_300_000.0;
        let text = toml::to_string_pretty(&f).unwrap();
        assert_eq!(toml::from_str::<TuningFile>(&text).unwrap(), f);
    }

    #[test]
    fn missing_entries_use_defaults() {
        let f: TuningFile = toml::from_str("[physics]\ngravity = 0.6\n").unwrap();
        assert!((f.physics.gravity - 0.6).abs() < f32::EPSILON);
        assert!((f.physics.hook_length - Tuning::default().hook_length).abs() < f32::EPSILON);
        assert_eq!(f.view, ViewFile::default());
    }
}
