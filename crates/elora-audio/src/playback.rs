//! Wiedergabe mit kira (Feature `playback`).

use std::collections::BTreeMap;
use std::sync::Arc;

use elora_sim::Vec2;
use kira::sound::FromFileError;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Frame, Panning, Tween};

use crate::{AudioSettings, Bank, Cue, SAMPLE_RATE, Sound, spatial};

/// Wiedergabe mit kira.
pub struct Audio {
    manager: Option<AudioManager<DefaultBackend>>,
    sounds: BTreeMap<Sound, StaticSoundData>,
    applied: Option<AudioSettings>,
    /// Laufende Musik (Schleife) und ihre Lautstärke.
    music: Option<(StreamingSoundHandle<FromFileError>, f32)>,
}

impl std::fmt::Debug for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Audio")
            .field("device", &self.manager.is_some())
            .field("sounds", &self.sounds.len())
            .finish_non_exhaustive()
    }
}

fn to_data(samples: &[f32]) -> StaticSoundData {
    let frames: Arc<[Frame]> = samples.iter().map(|&s| Frame::from_mono(s)).collect();
    StaticSoundData {
        sample_rate: SAMPLE_RATE,
        frames,
        settings: StaticSoundSettings::default(),
        slice: None,
    }
}

/// Amplitude 0..1 → Dezibel für kira.
fn decibels(amplitude: f32) -> Decibels {
    if amplitude <= 0.001 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * amplitude.log10())
    }
}

impl Audio {
    /// Erzeugt alle Sounds der Bank und öffnet das Audiogerät. Fehlt ein Gerät,
    /// bleibt das Spiel stumm (Warnung im Log).
    pub fn new(bank: &Bank) -> Self {
        let manager = match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
            Ok(m) => Some(m),
            Err(e) => {
                tracing::warn!("Kein Audiogerät, Spiel bleibt stumm: {e}");
                None
            }
        };
        let sounds = bank
            .sounds
            .iter()
            .map(|(s, src)| (*s, to_data(&src.samples())))
            .collect();
        for s in bank.missing() {
            tracing::warn!(
                "Sound `{}` fehlt (weder sounds.toml noch Tondatei)",
                s.name()
            );
        }
        Self {
            manager,
            sounds,
            applied: None,
            music: None,
        }
    }

    pub fn has_device(&self) -> bool {
        self.manager.is_some()
    }

    /// Gesamtlautstärke übernehmen (nur bei Änderung).
    pub fn apply(&mut self, settings: AudioSettings) {
        if self.applied == Some(settings) {
            return;
        }
        self.applied = Some(settings);
        if let Some(m) = &mut self.manager {
            let amp = if settings.muted {
                0.0
            } else {
                settings.volume.clamp(0.0, 1.0)
            };
            m.main_track().set_volume(decibels(amp), Tween::default());
        }
    }

    /// Musik (Ogg Vorbis oder WAV, beim Abspielen entpackt) in Schleife starten, falls noch
    /// keine läuft; `volume` 0..1 wird bei Änderung sanft nachgeführt.
    ///
    /// # Errors
    /// Die Datei ist keine lesbare Musik (ohne Audiogerät: nie).
    pub fn play_music(&mut self, data: &Arc<[u8]>, volume: f32) -> Result<(), String> {
        let tween = Tween {
            duration: std::time::Duration::from_millis(300),
            ..Tween::default()
        };
        if let Some((handle, v)) = &mut self.music {
            if (*v - volume).abs() > 0.001 {
                handle.set_volume(decibels(volume), tween);
                *v = volume;
            }
            return Ok(());
        }
        let Some(manager) = &mut self.manager else {
            return Ok(());
        };
        let sound = StreamingSoundData::from_cursor(std::io::Cursor::new(data.clone()))
            .map_err(|e| e.to_string())?
            .loop_region(..)
            .volume(Decibels::SILENCE);
        match manager.play(sound) {
            // sanft einblenden
            Ok(mut handle) => {
                handle.set_volume(
                    decibels(volume),
                    Tween {
                        duration: std::time::Duration::from_millis(1200),
                        ..Tween::default()
                    },
                );
                self.music = Some((handle, volume));
            }
            Err(e) => tracing::debug!("Musik nicht abgespielt: {e}"),
        }
        Ok(())
    }

    /// Musik ausblenden und beenden.
    pub fn stop_music(&mut self) {
        if let Some((mut handle, _)) = self.music.take() {
            handle.stop(Tween {
                duration: std::time::Duration::from_millis(600),
                ..Tween::default()
            });
        }
    }

    /// Spielt `cue`; räumliche Sounds relativ zu `ear` (Kameramitte).
    pub fn play(&mut self, cue: &Cue, ear: Vec2) {
        let Some(manager) = &mut self.manager else {
            return;
        };
        let Some(data) = self.sounds.get(&cue.sound) else {
            return;
        };
        let (gain, pan) = match cue.pos {
            Some(p) => match spatial(p, ear) {
                Some(v) => v,
                None => return,
            },
            None => (1.0, 0.0),
        };
        let sound = data
            .volume(decibels(gain))
            .panning(Panning(pan))
            .playback_rate(kira::PlaybackRate(f64::from(cue.pitch)));
        if let Err(e) = manager.play(sound) {
            tracing::debug!("Sound `{}` nicht abgespielt: {e}", cue.sound.name());
        }
    }
}

#[cfg(test)]
mod music_tests {
    use super::*;

    #[test]
    fn shipped_music_can_be_streamed() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/music");
        for name in [
            "menu",
            "tauwinkel",
            "bluetenwiesen",
            "boss",
            "fest",
            "murmelwald",
            "boss-wald",
            "wueste",
            "boss-wueste",
        ] {
            let data: Arc<[u8]> = std::fs::read(format!("{dir}/{name}.ogg")).unwrap().into();
            assert!(
                StreamingSoundData::from_cursor(std::io::Cursor::new(data)).is_ok(),
                "{name}"
            );
        }
        let junk: Arc<[u8]> = Arc::from(&b"keine Musik"[..]);
        assert!(StreamingSoundData::from_cursor(std::io::Cursor::new(junk)).is_err());
    }
}
