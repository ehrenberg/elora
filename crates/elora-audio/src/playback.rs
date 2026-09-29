//! Wiedergabe mit kira (Feature `playback`).

use std::collections::BTreeMap;
use std::sync::Arc;

use elora_sim::Vec2;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Frame, Panning, Tween};

use crate::{AudioSettings, Bank, Cue, SAMPLE_RATE, Sound, spatial};

/// Wiedergabe mit kira.
pub struct Audio {
    manager: Option<AudioManager<DefaultBackend>>,
    sounds: BTreeMap<Sound, StaticSoundData>,
    applied: Option<AudioSettings>,
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
            .map(|(s, def)| (*s, to_data(&def.render())))
            .collect();
        for s in bank.missing() {
            tracing::warn!("Sound `{}` fehlt in sounds.toml", s.name());
        }
        Self {
            manager,
            sounds,
            applied: None,
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
        let sound = data.volume(decibels(gain)).panning(Panning(pan));
        if let Err(e) = manager.play(sound) {
            tracing::debug!("Sound `{}` nicht abgespielt: {e}", cue.sound.name());
        }
    }
}
