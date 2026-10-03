//! Sounds von Elora (M5.7, E-032, E-081, E-105).
//!
//! - [`synth`]: prozeduraler Generator (sfxr-Prinzip), Parameter in
//!   `assets/sounds/sounds.toml`,
//! - Tondateien `assets/sounds/files/<name>.wav` (CC0, E-107, Quellen in
//!   `assets/SOURCES.md`) ersetzen den prozeduralen Klang eines Sounds,
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

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/files.rs"));
}
/// Eingebettete Tondateien (Sound-Name, WAV), siehe `build.rs`.
pub use embedded::FILES;

/// Räumliche Sounds sind bis zu dieser Entfernung (Welteinheiten) hörbar.
const RANGE: f32 = 1400.0;
/// Ab dieser seitlichen Entfernung ganz links bzw. rechts (begrenzt auf ±0.8).
const PAN_RANGE: f32 = 900.0;

#[derive(Debug, thiserror::Error)]
pub enum BankError {
    #[error("sounds.toml ist ungültig: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("unbekannter Sound `{0}`")]
    Unknown(String),
    #[error("Tondatei `{0}`: {1}")]
    Wav(String, &'static str),
}

/// Woher ein Sound kommt.
#[derive(Debug, Clone)]
pub enum Source {
    /// Prozedural aus `sounds.toml`.
    Synth(SoundDef),
    /// Tondatei (Mono, [`SAMPLE_RATE`]) mit Lautstärke aus `[gain]` in `sounds.toml`.
    File { samples: Vec<f32>, gain: f32 },
}

impl Source {
    pub fn samples(&self) -> Vec<f32> {
        match self {
            Self::Synth(def) => def.render(),
            Self::File { samples, gain } => samples.iter().map(|s| s * gain).collect(),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn duration(&self) -> f32 {
        match self {
            Self::Synth(def) => def.duration(),
            Self::File { samples, .. } => samples.len() as f32 / SAMPLE_RATE as f32,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }
}

/// Alle Sounds nach Name.
#[derive(Debug, Clone, Default)]
pub struct Bank {
    pub sounds: BTreeMap<Sound, Source>,
    /// Lautstärke je Tondatei (Abschnitt `[gain]` in `sounds.toml`, Standard 1).
    gains: BTreeMap<Sound, f32>,
}

impl Bank {
    /// Die eingebaute Bank: `sounds.toml` plus eingebettete Tondateien.
    ///
    /// # Errors
    /// Bei ungültigem TOML, unbekannten Namen oder fehlerhaften Tondateien.
    pub fn load() -> Result<Self, BankError> {
        let mut bank = Self::parse(SOUNDS_TOML)?;
        for (name, data) in FILES {
            bank.add_file(name, data)?;
        }
        Ok(bank)
    }

    /// Nur die prozeduralen Sounds und Lautstärken aus `sounds.toml`.
    ///
    /// # Errors
    /// Bei ungültigem TOML oder unbekannten Sound-Namen.
    pub fn parse(src: &str) -> Result<Self, BankError> {
        let raw: BTreeMap<String, toml::Value> = toml::from_str(src)?;
        let mut bank = Self::default();
        for (name, value) in raw {
            if name == "gain" {
                let gains: BTreeMap<String, f32> = value.try_into()?;
                for (n, g) in gains {
                    let sound = Sound::from_name(&n).ok_or(BankError::Unknown(n))?;
                    bank.gains.insert(sound, g);
                }
                continue;
            }
            let sound = Sound::from_name(&name).ok_or_else(|| BankError::Unknown(name.clone()))?;
            bank.sounds.insert(sound, Source::Synth(value.try_into()?));
        }
        Ok(bank)
    }

    /// Tondatei für `name` einsetzen (ersetzt einen prozeduralen Sound).
    ///
    /// # Errors
    /// Bei unbekanntem Namen oder nicht lesbarer WAV-Datei.
    pub fn add_file(&mut self, name: &str, data: &[u8]) -> Result<(), BankError> {
        let sound = Sound::from_name(name).ok_or_else(|| BankError::Unknown(name.to_owned()))?;
        let samples = decode_wav(data).map_err(|e| BankError::Wav(name.to_owned(), e))?;
        let gain = self.gains.get(&sound).copied().unwrap_or(1.0);
        self.sounds.insert(sound, Source::File { samples, gain });
        Ok(())
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

/// Liest 16-Bit-PCM-WAV (Mono oder Stereo, [`SAMPLE_RATE`]) als Mono-Samples.
///
/// # Errors
/// Bei anderem Format; `cargo xtask sound-import` erzeugt passende Dateien.
pub fn decode_wav(data: &[u8]) -> Result<Vec<f32>, &'static str> {
    let u16_at = |i: usize| data.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let u32_at = |i: usize| {
        data.get(i..i + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    if data.get(0..4) != Some(b"RIFF") || data.get(8..12) != Some(b"WAVE") {
        return Err("keine WAV-Datei");
    }
    let mut pos = 12;
    let mut channels = None;
    while let (Some(id), Some(len)) = (data.get(pos..pos + 4), u32_at(pos + 4)) {
        let body = pos + 8;
        let len = len as usize;
        match id {
            b"fmt " => {
                if u16_at(body) != Some(1) || u16_at(body + 14) != Some(16) {
                    return Err("nur 16-Bit-PCM");
                }
                if u32_at(body + 4) != Some(SAMPLE_RATE) {
                    return Err("Abtastrate muss 44100 Hz sein");
                }
                channels = u16_at(body + 2).filter(|c| (1..=2).contains(c));
            }
            b"data" => {
                let ch = usize::from(channels.ok_or("fmt fehlt oder mehr als 2 Kanäle")?);
                let bytes = data.get(body..body + len).ok_or("data zu kurz")?;
                return Ok(bytes
                    .chunks_exact(2 * ch)
                    .map(|frame| {
                        let sum: f32 = frame
                            .as_chunks::<2>()
                            .0
                            .iter()
                            .map(|b| f32::from(i16::from_le_bytes(*b)))
                            .sum();
                        #[allow(clippy::cast_precision_loss)]
                        let n = ch as f32;
                        sum / n / f32::from(i16::MAX)
                    })
                    .collect());
            }
            _ => {}
        }
        pos = body + len + (len & 1);
    }
    Err("data fehlt")
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
    /// Lautstärke der Menümusik 0..1 (E-121), zusätzlich zur Gesamtlautstärke.
    pub music_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume: 0.7,
            muted: false,
            music_volume: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_music_is_ogg_vorbis() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/music");
        for name in ["menu", "tauwinkel", "bluetenwiesen", "boss", "fest"] {
            let data = std::fs::read(format!("{dir}/{name}.ogg")).unwrap();
            assert_eq!(&data[..4], b"OggS", "{name}");
        }
    }

    #[test]
    fn embedded_bank_is_complete_and_renders() {
        let bank = Bank::load().expect("sounds.toml und Tondateien gültig");
        assert_eq!(bank.missing(), Vec::<Sound>::new(), "alle Sounds definiert");
        for (s, src) in &bank.sounds {
            let samples = src.samples();
            let peak = samples.iter().fold(0.0_f32, |m, v| m.max(v.abs()));
            assert!(!samples.is_empty(), "{}: leer", s.name());
            assert!(peak > 0.05, "{}: zu leise ({peak})", s.name());
            assert!(src.duration() <= 2.5, "{}: zu lang", s.name());
        }
    }

    #[test]
    fn wav_roundtrip_and_file_replaces_synth() {
        let samples = vec![0.0, 0.5, -0.5, 0.25];
        let decoded = decode_wav(&wav(&samples)).unwrap();
        for (orig, got) in samples.iter().zip(&decoded) {
            assert!((orig - got).abs() < 1e-3);
        }
        assert!(decode_wav(b"RIFF....WAVX").is_err());
        let mut bank = Bank::parse("[gain]\njump = 0.5\n[jump]\n[[jump.layer]]\n").unwrap();
        assert!(!bank.sounds[&Sound::Jump].is_file());
        bank.add_file("jump", &wav(&samples)).unwrap();
        let src = &bank.sounds[&Sound::Jump];
        assert!(src.is_file());
        assert!(
            (src.samples()[1] - 0.25).abs() < 1e-3,
            "Lautstärke aus [gain]"
        );
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
