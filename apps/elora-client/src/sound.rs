//! Sounds im Client (M5.7): Ereignisse der Szene → [`elora_audio`].
//!
//! Die Zustandsbits einer Figur (Sprung, Hook) gelten einen ganzen Tick, der
//! Client zeichnet aber öfter. Deshalb werden sie je Figur gemerkt und nur neu
//! gesetzte Bits bzw. Wechsel des Hook-Zustands lösen einen Sound aus.

use std::collections::HashMap;

use elora_audio::cues::{self, Cue, Listener, Sound};
use elora_audio::{Audio, AudioSettings, Bank};
use elora_client::scene::{Scene, SceneChar};
use elora_sim::{Event, HookState, Vec2};

use crate::figure::Landing;

/// Musik von der Platte laden; fehlt die Datei, bleibt das Menü still.
fn load_music() -> Option<Vec<f32>> {
    let data = std::fs::read(MENU_MUSIC).ok()?;
    match elora_audio::decode_wav(&data) {
        Ok(samples) => Some(samples),
        Err(e) => {
            tracing::warn!("{MENU_MUSIC}: {e}");
            None
        }
    }
}

#[derive(Debug)]
pub struct Sounds {
    audio: Audio,
    pub settings: AudioSettings,
    /// Zuletzt gesehene Bits und Hook-Zustand je Slot.
    last: HashMap<usize, (u8, HookState)>,
    /// Menümusik (E-121), falls `assets/music/menu.wav` vorhanden ist.
    music: Option<Vec<f32>>,
}

/// Datei der Menümusik (WAV, 16 Bit, 44,1 kHz; Mono oder Stereo).
pub const MENU_MUSIC: &str = "assets/music/menu.wav";

impl Sounds {
    /// # Panics
    /// Wenn `sounds.toml` oder eine Tondatei fehlerhaft ist (wird von Tests abgedeckt).
    pub fn new(settings: AudioSettings) -> Self {
        let bank = Bank::load().expect("assets/sounds: sounds.toml und Tondateien gültig");
        Self {
            audio: Audio::new(&bank),
            settings,
            last: HashMap::new(),
            music: load_music(),
        }
    }

    /// Menümusik an (im Menü) oder aus (im Spiel).
    pub fn menu_music(&mut self, on: bool) {
        self.audio.apply(self.settings);
        match (&self.music, on) {
            (Some(samples), true) => {
                let volume = self.settings.music_volume.clamp(0.0, 1.0);
                self.audio.play_music(samples, volume);
            }
            _ => self.audio.stop_music(),
        }
    }

    pub fn has_device(&self) -> bool {
        self.audio.has_device()
    }

    /// Sounds des Frames abspielen. `extra`: Sounds aus dem Client selbst
    /// (Chat, Emotes); Hörposition ist die Kameramitte.
    pub fn update(
        &mut self,
        scene: &Scene,
        events: &[Event],
        landings: &[Landing],
        extra: &[Cue],
        ear: Vec2,
    ) {
        self.audio.apply(self.settings);
        let listener = Listener {
            local: scene.local().map(|c| c.slot),
            team: scene.local().map(|c| c.team).unwrap_or_default(),
        };
        let pos_of = |slot: usize| {
            scene
                .chars
                .iter()
                .find(|c| c.slot == slot)
                .map(SceneChar::pos)
        };
        let mut cues: Vec<Cue> = events
            .iter()
            .flat_map(|e| cues::for_event(e, listener, pos_of))
            .collect();

        self.last
            .retain(|slot, _| scene.chars.iter().any(|c| c.slot == *slot));
        for c in &scene.chars {
            let bits = c.ch.core.triggered_events;
            let hook = c.ch.core.hook_state;
            let (last_bits, last_hook) = self.last.get(&c.slot).copied().unwrap_or((bits, hook));
            cues.extend(cues::for_character(
                c.pos(),
                bits & !last_bits,
                last_hook,
                hook,
            ));
            self.last.insert(c.slot, (bits, hook));
        }
        cues.extend(
            landings
                .iter()
                .filter(|l| l.strength > 0.3)
                .map(|l| Cue::at(Sound::Land, l.pos)),
        );
        cues.extend_from_slice(extra);

        for cue in &cues {
            self.audio.play(cue, ear);
        }
    }
}
