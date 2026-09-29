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

#[derive(Debug)]
pub struct Sounds {
    audio: Audio,
    pub settings: AudioSettings,
    /// Zuletzt gesehene Bits und Hook-Zustand je Slot.
    last: HashMap<usize, (u8, HookState)>,
}

impl Sounds {
    /// # Panics
    /// Wenn `sounds.toml` oder eine Tondatei fehlerhaft ist (wird von Tests abgedeckt).
    pub fn new(settings: AudioSettings) -> Self {
        let bank = Bank::load().expect("assets/sounds: sounds.toml und Tondateien gültig");
        Self {
            audio: Audio::new(&bank),
            settings,
            last: HashMap::new(),
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
