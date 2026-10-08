//! `settings.toml` in the user directory (M7.2, E-116): everything a player
//! sets – language, name and skin, graphics, sound, effects, mouse, favourites.
//! The developer tuning of the sandbox stays in `tuning.toml` ([`crate::tuning_file`]).
//!
//! Location: Linux `$XDG_CONFIG_HOME/elora` or `~/.config/elora`, Windows `%APPDATA%\Elora`,
//! macOS `~/Library/Application Support/Elora`. Missing entries keep their
//! default value, unknown ones are ignored.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use elora_protocol::Skin;
use serde::{Deserialize, Serialize};

use crate::effects::EffectSettings;
use crate::lang::Language;

pub const SETTINGS_FILE: &str = "settings.toml";

/// Directory for the user's settings; `None` if none can be determined.
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

/// User directory for data such as downloaded maps (M6.5); on Linux per XDG
/// `~/.local/share/elora`, otherwise like [`config_dir`].
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

/// File in the settings folder (fallback: working directory), e.g. `known_servers.toml`.
pub fn config_file(name: &str) -> PathBuf {
    config_dir().map_or_else(|| PathBuf::from(name), |d| d.join(name))
}

/// Own maps from the editor (E-152).
pub fn user_maps_dir() -> Option<PathBuf> {
    data_dir().map(|d| d.join("maps"))
}

/// Moves self-made adventure maps from the 0.9.x folder `maps/abenteuer/` to
/// `maps/adventure/` with English names (RF-13); the maps themselves are converted when they
/// are loaded. Does nothing once the old folder is gone.
pub fn migrate_user_maps() {
    let Some(maps) = user_maps_dir() else { return };
    move_old_maps(&maps.join("abenteuer"), &maps.join("adventure"));
}

fn move_old_maps(old: &Path, new: &Path) {
    let Ok(entries) = std::fs::read_dir(old) else {
        return;
    };
    if let Err(e) = std::fs::create_dir_all(new) {
        tracing::warn!("{}: {e}", new.display());
        return;
    }
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let target = new.join(format!("{}.{ext}", elora_map::rename::translate_id(stem)));
        if target.exists() {
            continue;
        }
        match std::fs::rename(&path, &target) {
            Ok(()) => tracing::info!("moved {} to {}", path.display(), target.display()),
            Err(e) => tracing::warn!("{}: {e}", path.display()),
        }
    }
    let _ = std::fs::remove_dir(old);
}

/// Path of the settings file (fallback: working directory).
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
    /// Key bindings (E-117).
    pub bindings: crate::bindings::Bindings,
    /// Saved servers (address:port).
    pub favorites: Vec<String>,
    /// Last connected server (“Quick play”).
    pub last_server: Option<String>,
    /// Master server for the internet list (HTTPS, E-112/E-127); empty = none (O-47).
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
            master_url: elora_server::config::DEFAULT_MASTER.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerSettings {
    pub name: String,
    /// Palette numbers (E-095/E-096).
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
    /// Skin; invalid numbers from the file fall back to the default.
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

/// Graphics (E-120).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphicsSettings {
    pub fullscreen: bool,
    pub vsync: bool,
    /// Anti-aliasing (4× MSAA, if the graphics card supports it).
    pub msaa: bool,
    /// Factor on the automatic size of menu and HUD (1 = by window height).
    pub ui_scale: f32,
    /// Weather: full, gentle or off (E-335).
    pub weather: WeatherQuality,
}

/// How much weather is visible (E-335); the effect in the adventure stays the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeatherQuality {
    #[default]
    Full,
    /// Fewer particles, no flashes, thinner veil.
    Gentle,
    Off,
}

impl WeatherQuality {
    pub const ALL: [Self; 3] = [Self::Full, Self::Gentle, Self::Off];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Full => "settings.weather_full",
            Self::Gentle => "settings.weather_gentle",
            Self::Off => "settings.weather_off",
        }
    }
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true,
            msaa: true,
            ui_scale: 1.0,
            weather: WeatherQuality::Full,
        }
    }
}

impl GraphicsSettings {
    /// UI scaling, clamped to a sensible range.
    pub fn ui_scale(self) -> f32 {
        self.ui_scale.clamp(0.5, 2.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputSettings {
    /// Mouse sensitivity in percent.
    pub mouse_sensitivity: f32,
    /// Switch to the picked-up weapon (E-287).
    pub auto_switch: AutoSwitch,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 100.0,
            auto_switch: AutoSwitch::New,
        }
    }
}

/// Switching to the picked-up weapon (E-287).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoSwitch {
    Off,
    /// Only if one did not have the weapon yet (as in the original).
    #[default]
    New,
    /// Also on ammo for a weapon one already has.
    Always,
}

impl AutoSwitch {
    pub const ALL: [Self; 3] = [Self::Off, Self::New, Self::Always];

    /// Switch if `w` was picked up and `had` applied before?
    pub fn wants(self, had: bool) -> bool {
        match self {
            Self::Off => false,
            Self::New => !had,
            Self::Always => true,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Off => "settings.auto_switch_off",
            Self::New => "settings.auto_switch_new",
            Self::Always => "settings.auto_switch_always",
        }
    }
}

impl Settings {
    /// Loads the file; if it does not exist, the default values apply.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(src) => {
                toml::from_str(&src).with_context(|| format!("{} is invalid", path.display()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("{} not readable", path.display())),
        }
    }

    /// Saves via a temporary file (no half-written `settings.toml`).
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("{} cannot be created", dir.display()))?;
        }
        let body = toml::to_string_pretty(self)?;
        let text = format!("# Elora – settings (written by the game)\n\n{body}");
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, text).with_context(|| format!("{} not writable", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| format!("{} not writable", path.display()))
    }

    /// Add a favourite (without duplicates) or remove it.
    #[allow(dead_code)] // Server browser (M7.7)
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
    fn auto_switch_modes() {
        assert!(!AutoSwitch::Off.wants(false));
        assert!(AutoSwitch::New.wants(false) && !AutoSwitch::New.wants(true));
        assert!(AutoSwitch::Always.wants(true));
        assert_eq!(Settings::default().input.auto_switch, AutoSwitch::New);
        let s: Settings = toml::from_str("[input]\nauto_switch = \"always\"\n").unwrap();
        assert_eq!(s.input.auto_switch, AutoSwitch::Always);
    }

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
        assert_eq!(s.player.skin(), Skin::default(), "invalid skin falls back");
        assert!((s.graphics.ui_scale() - 2.0).abs() < f32::EPSILON);
        assert_eq!(s.player.name, "Elora");
    }

    #[test]
    fn config_dir_follows_xdg() {
        if cfg!(target_os = "linux") {
            // only check that a path with “elora” results (don't change the environment)
            let dir = config_dir().expect("HOME set");
            assert!(dir.ends_with("elora"));
        }
    }

    #[test]
    fn old_adventure_maps_move_to_english_names() {
        let dir = std::env::temp_dir().join(format!("elora-maps-{}", std::process::id()));
        let (old, new) = (dir.join("abenteuer"), dir.join("adventure"));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("wiese-1.emap"), b"x").unwrap();
        move_old_maps(&old, &new);
        assert!(new.join("meadow-1.emap").is_file());
        assert!(!old.exists(), "old folder removed");
        move_old_maps(&old, &new); // nothing left to do
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
