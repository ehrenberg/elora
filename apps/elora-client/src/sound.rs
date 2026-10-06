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

/// Musikstück `<dir>/<name>.ogg` (oder `.wav`) lesen; fehlt es, bleibt es still.
fn load_music(dir: &str, name: &str) -> Option<Arc<[u8]>> {
    ["ogg", "wav"].iter().find_map(|ext| {
        let rel = format!("{dir}/{name}.{ext}");
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
    /// Je Figur: Ereignis-Bits, Hook-Zustand und ob sie im Treibsand steckt (letzter Frame).
    last: HashMap<usize, (u16, HookState, bool)>,
    /// Art jedes Gegners der Szene (für Klänge, wenn er schon verschwunden ist).
    kinds: HashMap<u32, String>,
    /// Figuren, die im letzten Frame erstarrt waren (Frostgeist).
    frozen: Vec<usize>,
    /// Gelesene Musikstücke, gepackt (E-121, E-285); `None`: fehlt oder unlesbar.
    tracks: HashMap<String, Option<Arc<[u8]>>>,
    /// Gerade laufendes Stück.
    playing: Option<String>,
    /// Gelesene Wetterklänge (`assets/ambience`, R2-W1).
    ambience: HashMap<String, Option<Arc<[u8]>>>,
    /// Donner, der noch kommt: Ort und verbleibende Verzögerung (s).
    thunder: Vec<(Vec2, f32)>,
}

/// Ordner der Wetterklänge: Schleifen `regen`, `wind`, `sand` und `donner` (R2-W1, E-338).
pub const AMBIENCE_DIR: &str = "assets/ambience";

/// Die Schleifen der Umgebungsspur.
pub const AMBIENCE: [&str; 3] = ["regen", "wind", "sand"];

/// Wie [`ambience_levels`], unter Dach oder in Höhlen (`shelter` 0..1) gedämpft (W1.6).
pub fn sheltered_levels(w: elora_map::Weather, shelter: f32) -> [f32; 3] {
    let s = shelter.clamp(0.0, 1.0);
    let [rain, wind, sand] = ambience_levels(w);
    [
        rain * (1.0 - 0.65 * s),
        wind * (1.0 - 0.4 * s),
        sand * (1.0 - 0.65 * s),
    ]
}

/// Lautstärken (0..1) der Schleifen [`AMBIENCE`] für ein Wetter.
pub fn ambience_levels(w: elora_map::Weather) -> [f32; 3] {
    use elora_map::WeatherKind as K;
    let i = w.intensity.clamp(0.0, 1.0);
    let breeze = w.wind.abs().min(1.0);
    match w.kind {
        K::Clear => [0.0, 0.0, 0.0],
        K::Rain => [0.35 + 0.45 * i, 0.12 * breeze, 0.0],
        K::Storm => [0.5 + 0.5 * i, 0.25 + 0.35 * i, 0.0],
        K::Fog => [0.0, 0.1 + 0.1 * i, 0.0],
        K::Leaves | K::Petals => [0.0, 0.12 + 0.25 * breeze * i.max(0.4), 0.0],
        K::Sandstorm => [0.0, 0.15 * i, 0.4 + 0.5 * i],
        K::Snow => [0.0, 0.1 + 0.15 * i, 0.0],
        K::Blizzard => [0.0, 0.45 + 0.5 * i, 0.0],
    }
}

/// Ordner der Musik: `<name>.ogg` (Ogg Vorbis, 44,1 kHz) oder `<name>.wav`; `menu` im Hauptmenü.
pub const MUSIC_DIR: &str = "assets/music";

/// Grundlautstärke der Umgebungsspur unter der Gesamtlautstärke.
const AMBIENCE_GAIN: f32 = 0.6;

impl Sounds {
    /// # Panics
    /// Wenn `sounds.toml` oder eine Tondatei fehlerhaft ist (wird von Tests abgedeckt).
    pub fn new(settings: AudioSettings) -> Self {
        let bank = Bank::load().expect("assets/sounds: sounds.toml und Tondateien gültig");
        Self {
            audio: Audio::new(&bank),
            settings,
            last: HashMap::new(),
            kinds: HashMap::new(),
            frozen: Vec::new(),
            tracks: HashMap::new(),
            playing: None,
            ambience: HashMap::new(),
            thunder: Vec::new(),
        }
    }

    /// Menümusik an (im Menü) oder aus (im Spiel); das Wetter verstummt.
    pub fn menu_music(&mut self, on: bool) {
        self.audio.stop_ambience();
        self.thunder.clear();
        self.music(on.then_some("menu"));
    }

    /// Umgebungsspur des Wetters (jeden Frame): Schleifen weich nachführen, neuen Donner
    /// (Ort, Verzögerung) einreihen und fälligen spielen – laut in der Nähe, leiser fern.
    pub fn weather(
        &mut self,
        dt: f32,
        weather: elora_map::Weather,
        shelter: f32,
        new_thunder: Vec<(Vec2, f32)>,
        ear: Vec2,
    ) {
        self.audio.apply(self.settings);
        for (name, level) in AMBIENCE.into_iter().zip(sheltered_levels(weather, shelter)) {
            let Some(data) = self.ambience_file(name) else {
                continue;
            };
            if let Err(e) = self.audio.ambience(name, &data, level * AMBIENCE_GAIN) {
                tracing::warn!("{AMBIENCE_DIR}/{name}: {e}");
                self.ambience.insert(name.to_owned(), None);
            }
        }
        self.thunder.extend(new_thunder);
        let mut due = Vec::new();
        self.thunder.retain_mut(|t| {
            t.1 -= dt;
            if t.1 <= 0.0 {
                due.push(t.0);
            }
            t.1 > 0.0
        });
        if due.is_empty() {
            return;
        }
        let Some(data) = self.ambience_file("donner") else {
            return;
        };
        for pos in due {
            let d = pos - ear;
            let volume = (1.0 - d.length() / 3000.0).clamp(0.35, 1.0);
            let pan = (d.x / 1500.0).clamp(-0.6, 0.6);
            if let Err(e) = self.audio.play_once(&data, volume, pan) {
                tracing::warn!("{AMBIENCE_DIR}/donner: {e}");
                self.ambience.insert("donner".to_owned(), None);
                return;
            }
        }
    }

    /// Knistern der nächsten Feuerstelle (R2-M2.4): `level` 0..1 nach Abstand, weich nachgeführt.
    pub fn fire(&mut self, level: f32) {
        let Some(data) = self.ambience_file("feuer") else {
            return;
        };
        if let Err(e) = self
            .audio
            .ambience("feuer", &data, level.clamp(0.0, 1.0) * AMBIENCE_GAIN)
        {
            tracing::warn!("{AMBIENCE_DIR}/feuer: {e}");
            self.ambience.insert("feuer".to_owned(), None);
        }
    }

    fn ambience_file(&mut self, name: &str) -> Option<Arc<[u8]>> {
        self.ambience
            .entry(name.to_owned())
            .or_insert_with(|| load_music(AMBIENCE_DIR, name))
            .clone()
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
            .or_insert_with(|| load_music(MUSIC_DIR, name));
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
        // Gegner haben eigene Klänge (R2-M2.3, R2-M2.4): Art über die Id des Gegners; besiegte
        // sind schon aus der Szene verschwunden, ihre Art ist gemerkt
        let kinds = &self.kinds;
        let kind_of = |e: &Event| {
            let (Event::CreatureAct { id, .. }
            | Event::CreatureFire { id, .. }
            | Event::CreatureHit { id, .. }
            | Event::CreatureDeath { id, .. }) = *e
            else {
                return None;
            };
            scene
                .creatures
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.kind.as_str())
                .or_else(|| kinds.get(&id).map(String::as_str))
        };
        let mut cues: Vec<Cue> = events
            .iter()
            .flat_map(|e| {
                kind_of(e)
                    .and_then(|k| cues::for_creature(e, k))
                    .unwrap_or_else(|| cues::for_event(e, listener, pos_of))
            })
            .collect();

        self.kinds = scene
            .creatures
            .iter()
            .map(|c| (c.id, c.kind.clone()))
            .collect();
        self.last
            .retain(|slot, _| scene.chars.iter().any(|c| c.slot == *slot));
        for c in &scene.chars {
            let bits = c.ch.core.triggered_events;
            let hook = c.ch.core.hook_state;
            let sand = c.ch.core.sand_ticks > 0;
            let (last_bits, last_hook, last_sand) = self
                .last
                .get(&c.slot)
                .copied()
                .unwrap_or((bits, hook, sand));
            cues.extend(cues::for_character(
                c.pos(),
                bits & !last_bits,
                last_hook,
                hook,
            ));
            // hinein in den Treibsand (E-318)
            if sand && !last_sand {
                cues.push(Cue::at(Sound::Quicksand, c.pos()));
            }
            // erstarrt (Frostgeist, R2-M2.4)
            if c.ch.core.frozen > 0 && !self.frozen.contains(&c.slot) {
                cues.push(Cue::at(Sound::Freeze, c.pos()));
            }
            self.last.insert(c.slot, (bits, hook, sand));
        }
        self.frozen = scene
            .chars
            .iter()
            .filter(|c| c.ch.core.frozen > 0)
            .map(|c| c.slot)
            .collect();
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

#[cfg(test)]
mod tests {
    use super::*;
    use elora_map::{Weather, WeatherKind};

    fn w(kind: WeatherKind, intensity: f32) -> Weather {
        Weather {
            kind,
            intensity,
            wind: 0.0,
        }
    }

    #[test]
    fn weather_sounds_follow_the_weather() {
        assert!(
            ambience_levels(Weather::CLEAR).iter().all(|l| *l <= 0.0),
            "schön: still"
        );
        let [rain, wind, sand] = ambience_levels(w(WeatherKind::Storm, 1.0));
        assert!(
            rain > 0.9 && wind > 0.5 && sand == 0.0,
            "Gewitter: Regen und Wind"
        );
        let [_, _, sand] = ambience_levels(w(WeatherKind::Sandstorm, 1.0));
        assert!(sand > 0.8, "Sandsturm rieselt");
        let soft = ambience_levels(w(WeatherKind::Rain, 0.2))[0];
        let hard = ambience_levels(w(WeatherKind::Rain, 1.0))[0];
        assert!(hard > soft, "stärkerer Regen ist lauter");
        let open = ambience_levels(w(WeatherKind::Rain, 1.0))[0];
        assert!(
            sheltered_levels(w(WeatherKind::Rain, 1.0), 1.0)[0] < open * 0.5,
            "unter Dach leiser"
        );
        for kind in WeatherKind::ALL {
            for l in ambience_levels(w(kind, 1.0)) {
                assert!((0.0..=1.0).contains(&l), "{kind:?}: {l}");
            }
        }
    }

    #[test]
    fn ambience_files_are_shipped() {
        for name in AMBIENCE.into_iter().chain(["donner", "feuer"]) {
            let path = format!(
                "{}/../../{AMBIENCE_DIR}/{name}.ogg",
                env!("CARGO_MANIFEST_DIR")
            );
            let data = std::fs::read(&path).expect(&path);
            assert_eq!(&data[..4], b"OggS", "{name}");
        }
    }
}
