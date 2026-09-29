//! Sounds von Elora (M5.7, E-032, E-081, E-105).
//!
//! - [`synth`]: prozeduraler Generator (sfxr-Prinzip), Parameter in
//!   `assets/sounds/sounds.toml`,
//! - [`cues`]: welche Ereignisse welche Sounds auslösen,
//! - [`Audio`]: Wiedergabe mit kira – Lautstärke nach Entfernung zur Kamera,
//!   Stereo-Panorama, Gesamtlautstärke. Ohne Audiogerät läuft das Spiel stumm weiter.

pub mod cues;
pub mod synth;

#[cfg(feature = "playback")]
mod playback;

use std::collections::BTreeMap;

use elora_sim::Vec2;

#[cfg(feature = "playback")]
pub use playback::Audio;

pub use cues::{Cue, Listener, Sound};
pub use synth::{Layer, SAMPLE_RATE, SoundDef, Wave};

/// Die Sound-Parameter des Spiels (eingebettet).
pub const SOUNDS_TOML: &str = include_str!("../../../assets/sounds/sounds.toml");

/// Räumliche Sounds sind bis zu dieser Entfernung (Welteinheiten) hörbar.
const RANGE: f32 = 1400.0;
/// Ab dieser seitlichen Entfernung ganz links bzw. rechts (begrenzt auf ±0.8).
const PAN_RANGE: f32 = 900.0;

#[derive(Debug, thiserror::Error)]
pub enum BankError {
    #[error("sounds.toml ist ungültig: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("unbekannter Sound `{0}` in sounds.toml")]
    Unknown(String),
}

/// Sound-Definitionen nach Name.
#[derive(Debug, Clone, Default)]
pub struct Bank {
    pub sounds: BTreeMap<Sound, SoundDef>,
}

impl Bank {
    /// # Errors
    /// Bei ungültigem TOML oder unbekannten Sound-Namen.
    pub fn parse(src: &str) -> Result<Self, BankError> {
        let raw: BTreeMap<String, SoundDef> = toml::from_str(src)?;
        let mut sounds = BTreeMap::new();
        for (name, def) in raw {
            let sound = Sound::from_name(&name).ok_or_else(|| BankError::Unknown(name.clone()))?;
            sounds.insert(sound, def);
        }
        Ok(Self { sounds })
    }

    /// Sounds, für die es keine Definition gibt.
    pub fn missing(&self) -> Vec<Sound> {
        Sound::ALL
            .into_iter()
            .filter(|s| !self.sounds.contains_key(s))
            .collect()
    }
}

/// Mono-Samples als 16-Bit-WAV (für Hörproben).
#[allow(clippy::cast_possible_truncation)]
pub fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = u32::try_from(samples.len() * 2).unwrap_or(u32::MAX);
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // Größe des fmt-Blocks
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // Mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // Bytes pro Sekunde
    out.extend_from_slice(&2u16.to_le_bytes()); // Bytes pro Frame
    out.extend_from_slice(&16u16.to_le_bytes()); // Bits pro Sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// Lautstärke (Amplitude 0..1) und Panorama (−1..1) eines Sounds bei `pos`,
/// gehört von `ear`. `None` = zu weit weg.
pub fn spatial(pos: Vec2, ear: Vec2) -> Option<(f32, f32)> {
    let d = pos - ear;
    let gain = 1.0 - d.length() / RANGE;
    (gain > 0.0).then(|| (gain, (d.x / PAN_RANGE).clamp(-0.8, 0.8)))
}

/// Einstellungen der Wiedergabe, gespeichert in `tuning.toml` unter `[audio]`.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    /// Gesamtlautstärke 0..1.
    pub volume: f32,
    pub muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume: 0.7,
            muted: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_bank_is_complete_and_renders() {
        let bank = Bank::parse(SOUNDS_TOML).expect("sounds.toml gültig");
        assert_eq!(bank.missing(), Vec::<Sound>::new(), "alle Sounds definiert");
        for (s, def) in &bank.sounds {
            let samples = def.render();
            let peak = samples.iter().fold(0.0_f32, |m, v| m.max(v.abs()));
            assert!(!samples.is_empty(), "{}: leer", s.name());
            assert!(peak > 0.05, "{}: zu leise ({peak})", s.name());
            assert!(def.duration() <= 2.5, "{}: zu lang", s.name());
        }
    }

    #[test]
    fn unknown_names_are_rejected() {
        assert!(matches!(
            Bank::parse("[gibt_es_nicht]\nvolume = 0.5\n"),
            Err(BankError::Unknown(_))
        ));
    }

    #[test]
    fn spatial_falloff_and_pan() {
        let ear = Vec2::default();
        let (g, p) = spatial(Vec2::default(), ear).unwrap();
        assert!((g - 1.0).abs() < 1e-6 && p.abs() < 1e-6);
        let (g, p) = spatial(Vec2::new(700.0, 0.0), ear).unwrap();
        assert!((g - 0.5).abs() < 1e-3 && p > 0.7);
        let (_, p) = spatial(Vec2::new(-300.0, 0.0), ear).unwrap();
        assert!(p < 0.0);
        assert!(spatial(Vec2::new(0.0, 2000.0), ear).is_none());
    }

    #[test]
    fn wav_header() {
        let w = wav(&[0.0, 1.0, -1.0]);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(&w[8..12], b"WAVE");
        assert_eq!(w.len(), 44 + 6);
        assert_eq!(i16::from_le_bytes([w[46], w[47]]), i16::MAX);
    }
}
