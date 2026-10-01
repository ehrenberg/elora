//! `settings.toml` im Benutzerverzeichnis (M7.2, E-116): alles, was ein Spieler
//! einstellt – Sprache, Name und Skin, Grafik, Ton, Effekte, Maus, Favoriten.
//! Das Entwickler-Tuning der Sandbox bleibt in `tuning.toml` ([`crate::tuning_file`]).
//!
//! Ort: Linux `$XDG_CONFIG_HOME/elora` bzw. `~/.config/elora`, Windows `%APPDATA%\Elora`,
//! macOS `~/Library/Application Support/Elora`. Fehlende Einträge behalten ihren
//! Standardwert, unbekannte werden ignoriert.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use elora_protocol::Skin;
use serde::{Deserialize, Serialize};

use crate::effects::EffectSettings;
use crate::lang::Language;

pub const SETTINGS_FILE: &str = "settings.toml";

/// Verzeichnis für Einstellungen des Benutzers; `None`, wenn keins bestimmbar ist.
pub fn config_dir() -> Option<PathBuf> {
    let env = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if cfg!(target_os = "windows") {
        env("APPDATA").map(|p| p.join("Elora"))
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support/Elora"))
    } else {
        env("XDG_CONFIG_HOME")
            .or_else(|| env("HOME").map(|h| h.join(".config")))
            .map(|p| p.join("elora"))
    }
}

/// Benutzerverzeichnis für Daten wie heruntergeladene Karten (M6.5); unter Linux nach XDG
/// `~/.local/share/elora`, sonst wie [`config_dir`].
pub fn data_dir() -> Option<PathBuf> {
    if cfg!(any(target_os = "windows", target_os = "macos")) {
        return config_dir();
    }
    std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .map(|p| p.join("elora"))
}

/// Eigene Karten aus dem Editor (E-152).
pub fn user_maps_dir() -> Option<PathBuf> {
    data_dir().map(|d| d.join("maps"))
}

/// Pfad der Einstellungsdatei (Rückfall: Arbeitsverzeichnis).
pub fn settings_path() -> PathBuf {
    config_dir().map_or_else(|| PathBuf::from(SETTINGS_FILE), |d| d.join(SETTINGS_FILE))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: Language,
    pub player: PlayerSettings,
    pub graphics: GraphicsSettings,
    pub audio: elora_audio::AudioSettings,
    pub effects: EffectSettings,
    pub input: InputSettings,
    /// Tastenbelegung (E-117).
    pub bindings: crate::bindings::Bindings,
    /// Gespeicherte Server (Adresse:Port).
    pub favorites: Vec<String>,
    /// Zuletzt verbundener Server („Schnell spielen“).
    pub last_server: Option<String>,
    /// Master-Server für die Internet-Liste (HTTPS, E-112/E-127); leer = keiner (O-47).
    pub master_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::from_env(),
            player: PlayerSettings::default(),
            graphics: GraphicsSettings::default(),
            audio: elora_audio::AudioSettings::default(),
            effects: EffectSettings::default(),
            input: InputSettings::default(),
            bindings: crate::bindings::Bindings::default(),
            favorites: Vec::new(),
            last_server: None,
            master_url: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerSettings {
    pub name: String,
    /// Palettennummern (E-095/E-096).
    pub body: u8,
    pub feet: u8,
    pub eyes: u8,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            name: "Elora".into(),
            body: 0,
            feet: 0,
            eyes: 0,
        }
    }
}

impl PlayerSettings {
    /// Skin; ungültige Nummern aus der Datei fallen auf den Standard zurück.
    pub fn skin(&self) -> Skin {
        let skin = Skin {
            body: self.body,
            feet: self.feet,
            eyes: self.eyes,
        };
        if skin.is_valid() {
            skin
        } else {
            Skin::default()
        }
    }

    pub fn set_skin(&mut self, skin: Skin) {
        self.body = skin.body;
        self.feet = skin.feet;
        self.eyes = skin.eyes;
    }
}

/// Grafik (E-120).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphicsSettings {
    pub fullscreen: bool,
    pub vsync: bool,
    /// Kantenglättung (4× MSAA, falls die Grafikkarte es kann).
    pub msaa: bool,
    /// Faktor auf die automatische Größe von Menü und HUD (1 = nach Fensterhöhe).
    pub ui_scale: f32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true,
            msaa: true,
            ui_scale: 1.0,
        }
    }
}

impl GraphicsSettings {
    /// UI-Skalierung, begrenzt auf einen sinnvollen Bereich.
    pub fn ui_scale(self) -> f32 {
        self.ui_scale.clamp(0.5, 2.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputSettings {
    /// Maus-Empfindlichkeit in Prozent.
    pub mouse_sensitivity: f32,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 100.0,
        }
    }
}

impl Settings {
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

    /// Speichert über eine Zwischendatei (kein halb geschriebenes `settings.toml`).
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("{} nicht anlegbar", dir.display()))?;
        }
        let body = toml::to_string_pretty(self)?;
        let text = format!("# Elora – Einstellungen (vom Spiel geschrieben)\n\n{body}");
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, text)
            .with_context(|| format!("{} nicht schreibbar", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| format!("{} nicht schreibbar", path.display()))
    }

    /// Favorit hinzufügen (ohne Doppelte) bzw. entfernen.
    #[allow(dead_code)] // Server-Browser (M7.7)
    pub fn toggle_favorite(&mut self, address: &str) {
        if let Some(i) = self.favorites.iter().position(|a| a == address) {
            self.favorites.remove(i);
        } else {
            self.favorites.push(address.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_through_file() {
        let dir = std::env::temp_dir().join(format!("elora-settings-{}", std::process::id()));
        let path = dir.join("sub").join(SETTINGS_FILE);
        let mut s = Settings {
            language: Language::En,
            ..Settings::default()
        };
        s.player.name = "Nimbus".into();
        s.player.set_skin(Skin {
            body: 7,
            feet: 6,
            eyes: 1,
        });
        s.graphics.msaa = false;
        s.toggle_favorite("127.0.0.1:8303");
        s.toggle_favorite("10.0.0.2:8303");
        s.toggle_favorite("127.0.0.1:8303");
        s.save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();
        assert_eq!(loaded, s);
        assert_eq!(loaded.favorites, ["10.0.0.2:8303"]);
        assert!(!path.with_extension("toml.tmp").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_file_and_bad_values() {
        let s = Settings::load(Path::new("/gibt/es/nicht/settings.toml")).unwrap();
        assert!(s.graphics.vsync);
        let s: Settings =
            toml::from_str("[player]\nbody = 99\n[graphics]\nui_scale = 9.0\n").unwrap();
        assert_eq!(
            s.player.skin(),
            Skin::default(),
            "ungültiger Skin fällt zurück"
        );
        assert!((s.graphics.ui_scale() - 2.0).abs() < f32::EPSILON);
        assert_eq!(s.player.name, "Elora");
    }

    #[test]
    fn config_dir_follows_xdg() {
        if cfg!(target_os = "linux") {
            // nur prüfen, dass ein Pfad mit „elora“ entsteht (Umgebung nicht verändern)
            let dir = config_dir().expect("HOME gesetzt");
            assert!(dir.ends_with("elora"));
        }
    }
}
