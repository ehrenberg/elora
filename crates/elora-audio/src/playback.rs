//! Playback with kira (feature `playback`).

use std::collections::BTreeMap;
use std::sync::Arc;

use elora_sim::Vec2;
use kira::sound::FromFileError;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Frame, Panning, Tween};

use crate::{AudioSettings, Bank, Cue, SAMPLE_RATE, Sound, spatial};

/// Playback with kira.
pub struct Audio {
    manager: Option<AudioManager<DefaultBackend>>,
    sounds: BTreeMap<Sound, StaticSoundData>,
    applied: Option<AudioSettings>,
    /// Running music (loop) and its volume.
    music: Option<(StreamingSoundHandle<FromFileError>, f32)>,
    /// Ambience track (weather, R2-W1): running loops per name with volume.
    ambience: BTreeMap<String, (StreamingSoundHandle<FromFileError>, f32)>,
    /// Spoken narration (intro, E-355): once, can be cut off.
    voice: Option<StreamingSoundHandle<FromFileError>>,
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

/// Amplitude 0..1 → decibels for kira.
fn decibels(amplitude: f32) -> Decibels {
    if amplitude <= 0.001 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * amplitude.log10())
    }
}

impl Audio {
    /// Creates all sounds of the bank and opens the audio device. Without a device
    /// the game stays muted (warning in the log).
    pub fn new(bank: &Bank) -> Self {
        let manager = match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
            Ok(m) => Some(m),
            Err(e) => {
                tracing::warn!("No audio device, game stays silent: {e}");
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
                "Sound `{}` missing (neither sounds.toml nor sound file)",
                s.name()
            );
        }
        Self {
            manager,
            sounds,
            applied: None,
            music: None,
            ambience: BTreeMap::new(),
            voice: None,
        }
    }

    pub fn has_device(&self) -> bool {
        self.manager.is_some()
    }

    /// Applies the master volume (only on change).
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

    /// Starts music (Ogg Vorbis or WAV, decoded while playing) in a loop if none is
    /// running yet; `volume` 0..1 is smoothly adjusted on change.
    ///
    /// # Errors
    /// The file is not readable music (without an audio device: never).
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
            // fade in smoothly
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
            Err(e) => tracing::debug!("Music not played: {e}"),
        }
        Ok(())
    }

    /// Fades out and stops the music.
    pub fn stop_music(&mut self) {
        if let Some((mut handle, _)) = self.music.take() {
            handle.stop(Tween {
                duration: std::time::Duration::from_millis(600),
                ..Tween::default()
            });
        }
    }

    /// Adjusts the ambience loop `name` (second track next to the music, R2-W1) to
    /// `volume` 0..1 with a soft crossfade; at 0 it is faded out and stopped. Call every frame.
    ///
    /// # Errors
    /// The file is not a readable sound file (without an audio device: never).
    pub fn ambience(&mut self, name: &str, data: &Arc<[u8]>, volume: f32) -> Result<(), String> {
        let fade = |ms| Tween {
            duration: std::time::Duration::from_millis(ms),
            ..Tween::default()
        };
        if let Some((handle, v)) = self.ambience.get_mut(name) {
            if volume <= 0.001 {
                handle.stop(fade(1500));
                self.ambience.remove(name);
            } else if (*v - volume).abs() > 0.02 {
                handle.set_volume(decibels(volume), fade(800));
                *v = volume;
            }
            return Ok(());
        }
        if volume <= 0.001 {
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
            Ok(mut handle) => {
                handle.set_volume(decibels(volume), fade(2000));
                self.ambience.insert(name.to_owned(), (handle, volume));
            }
            Err(e) => tracing::debug!("Ambience `{name}` not played: {e}"),
        }
        Ok(())
    }

    /// Fades out all ambience loops (menu, editor).
    pub fn stop_ambience(&mut self) {
        let names: Vec<String> = self.ambience.keys().cloned().collect();
        for n in names {
            if let Some((mut h, _)) = self.ambience.remove(&n) {
                h.stop(Tween {
                    duration: std::time::Duration::from_millis(800),
                    ..Tween::default()
                });
            }
        }
    }

    /// Plays spoken narration once (streamed); a running one is replaced.
    ///
    /// # Errors
    /// The file is not a readable sound file (without an audio device: never).
    pub fn play_voice(&mut self, data: &Arc<[u8]>, volume: f32) -> Result<(), String> {
        self.stop_voice();
        let Some(manager) = &mut self.manager else {
            return Ok(());
        };
        let sound = StreamingSoundData::from_cursor(std::io::Cursor::new(data.clone()))
            .map_err(|e| e.to_string())?
            .volume(decibels(volume));
        match manager.play(sound) {
            Ok(handle) => self.voice = Some(handle),
            Err(e) => tracing::debug!("Voice not played: {e}"),
        }
        Ok(())
    }

    /// Fades the narration out quickly (intro skipped).
    pub fn stop_voice(&mut self) {
        if let Some(mut handle) = self.voice.take() {
            handle.stop(Tween {
                duration: std::time::Duration::from_millis(250),
                ..Tween::default()
            });
        }
    }

    /// Plays a longer sound (thunder) once, streamed, with volume and panning.
    ///
    /// # Errors
    /// The file is not a readable sound file (without an audio device: never).
    pub fn play_once(&mut self, data: &Arc<[u8]>, volume: f32, pan: f32) -> Result<(), String> {
        let Some(manager) = &mut self.manager else {
            return Ok(());
        };
        let sound = StreamingSoundData::from_cursor(std::io::Cursor::new(data.clone()))
            .map_err(|e| e.to_string())?
            .volume(decibels(volume))
            .panning(Panning(pan));
        if let Err(e) = manager.play(sound) {
            tracing::debug!("Sound not played: {e}");
        }
        Ok(())
    }

    /// Plays `cue`; spatial sounds relative to `ear` (camera center).
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
            tracing::debug!("Sound `{}` not played: {e}", cue.sound.name());
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
            "party",
            "murmelwald",
            "boss-forest",
            "desert",
            "boss-desert",
            "frost",
            "boss-frost",
            "../ambience/rain",
            "../ambience/wind",
            "../ambience/sand",
            "../ambience/thunder",
            "../ambience/fire",
        ] {
            let data: Arc<[u8]> = std::fs::read(format!("{dir}/{name}.ogg")).unwrap().into();
            assert!(
                StreamingSoundData::from_cursor(std::io::Cursor::new(data)).is_ok(),
                "{name}"
            );
        }
        let junk: Arc<[u8]> = Arc::from(&b"not music"[..]);
        assert!(StreamingSoundData::from_cursor(std::io::Cursor::new(junk)).is_err());
    }
}
