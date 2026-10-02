//! Sounds im Client (M5.7): Ereignisse der Szene → [`elora_audio`].
//!
//! Die Zustandsbits einer Figur (Sprung, Hook) gelten einen ganzen Tick, der
//! Client zeichnet aber öfter. Deshalb werden sie je Figur gemerkt und nur neu
//! gesetzte Bits bzw. Wechsel des Hook-Zustands lösen einen Sound aus.

use std::collections::HashMap;
use std::sync::Arc;

use elora_audio::cues::{self, Cue, Listener, Sound};
use elora_audio::{Audio, AudioSettings, Bank};
use elora_client::scene::{Scene, SceneChar};
use elora_sim::{Event, HookState, Vec2};

use crate::figure::Landing;

/// Musikstück `assets/music/<name>.ogg` (oder `.wav`) lesen; fehlt es, bleibt es still.
fn load_music(name: &str) -> Option<Arc<[u8]>> {
    ["ogg", "wav"].iter().find_map(|ext| {
        let rel = format!("{MUSIC_DIR}/{name}.{ext}");
        std::fs::read(elora_server::paths::resolve(std::path::Path::new(&rel)))
            .ok()
            .map(Arc::from)
    })
}

#[derive(Debug)]
pub struct Sounds {
    audio: Audio,
    pub settings: AudioSettings,
    /// Zuletzt gesehene Bits und Hook-Zustand je Slot.
    last: HashMap<usize, (u16, HookState)>,
    /// Gelesene Musikstücke, gepackt (E-121, E-285); `None`: fehlt oder unlesbar.
    tracks: HashMap<String, Option<Arc<[u8]>>>,
    /// Gerade laufendes Stück.
    playing: Option<String>,
}

/// Ordner der Musik: `<name>.ogg` (Ogg Vorbis, 44,1 kHz) oder `<name>.wav`; `menu` im Hauptmenü.
pub const MUSIC_DIR: &str = "assets/music";

impl Sounds {
    /// # Panics
    /// Wenn `sounds.toml` oder eine Tondatei fehlerhaft ist (wird von Tests abgedeckt).
    pub fn new(settings: AudioSettings) -> Self {
        let bank = Bank::load().expect("assets/sounds: sounds.toml und Tondateien gültig");
        Self {
            audio: Audio::new(&bank),
            settings,
            last: HashMap::new(),
            tracks: HashMap::new(),
            playing: None,
        }
    }

    /// Menümusik an (im Menü) oder aus (im Spiel).
    pub fn menu_music(&mut self, on: bool) {
        self.music(on.then_some("menu"));
    }

    /// Musikstück `name` spielen (jeden Frame aufrufen); ein anderes wird ausgeblendet,
    /// `None` blendet aus. Fehlt das Stück, bleibt es still.
    pub fn music(&mut self, name: Option<&str>) {
        self.audio.apply(self.settings);
        if self.playing.as_deref() != name {
            self.audio.stop_music();
            self.playing = None;
        }
        let Some(name) = name else { return };
        let track = self
            .tracks
            .entry(name.to_owned())
            .or_insert_with(|| load_music(name));
        let Some(data) = track.clone() else { return };
        let volume = self.settings.music_volume.clamp(0.0, 1.0);
        match self.audio.play_music(&data, volume) {
            Ok(()) => self.playing = Some(name.to_owned()),
            Err(e) => {
                tracing::warn!("{MUSIC_DIR}/{name}: {e}");
                *track = None;
            }
        }
    }

    /// Nicht räumliche Klänge (Oberfläche) sofort abspielen.
    pub fn play_global(&mut self, cues: &[Cue]) {
        self.audio.apply(self.settings);
        for cue in cues {
            self.audio.play(cue, Vec2::ZERO);
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
