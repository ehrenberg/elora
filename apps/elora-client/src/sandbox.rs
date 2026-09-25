//! Physik-Sandbox (M1): eine Figur auf einer Textkarte, fester Tick, Interpolation.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Context as _;
use elora_map::{EntityKind, Map};
use elora_sim::replay::Recording;
use elora_sim::{CharacterCore, TICKS_PER_SECOND, Tuning, Vec2, World};

use crate::controls::Controls;

pub const TICK: Duration = Duration::from_micros(1_000_000 / TICKS_PER_SECOND as u64);
/// Schutz gegen Aufholspiralen nach Hängern.
const MAX_TICKS_PER_FRAME: u32 = 10;

#[derive(Debug)]
pub struct Sandbox {
    pub map_path: PathBuf,
    pub map: Map,
    pub world: World,
    pub player: usize,
    /// Zustand vor dem letzten Tick (für Interpolation).
    pub prev: CharacterCore,
    accumulator: Duration,
    watcher: Option<MapWatcher>,
    /// Ergebnis des letzten Hot-Reloads (Fehlertext bei ungültiger Karte).
    pub reload_error: Option<String>,
    /// Laufende Eingabe-Aufzeichnung (M1.6).
    pub recording: Option<Recording>,
}

/// Ablage der Aufzeichnungen: jede wird dort zum Golden-Test von `elora-sim`.
pub const RECORDINGS_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../crates/elora-sim/tests/recordings"
);

impl Sandbox {
    pub fn load(map_path: &Path, tuning: Tuning) -> anyhow::Result<Self> {
        let map = load_map(map_path)?;
        let mut world = World::new(tuning, map.collision());
        let player = world.spawn(spawn_point(&map));
        let prev = world.characters[player].clone().expect("gerade gespawnt");
        let watcher = MapWatcher::new(map_path)
            .inspect_err(|e| tracing::warn!("Hot-Reload nicht verfügbar: {e:#}"))
            .ok();
        Ok(Self {
            map_path: map_path.to_path_buf(),
            map,
            world,
            player,
            prev,
            accumulator: Duration::ZERO,
            watcher,
            reload_error: None,
            recording: None,
        })
    }

    pub fn character(&self) -> &CharacterCore {
        self.world.characters[self.player]
            .as_ref()
            .expect("Spieler existiert")
    }

    /// Lädt die Karte neu, wenn sich die Datei geändert hat. Elora behält ihre Position.
    pub fn poll_reload(&mut self) {
        if !self.watcher.as_ref().is_some_and(MapWatcher::changed) {
            return;
        }
        match load_map(&self.map_path) {
            Ok(map) => {
                self.stop_recording("Karte geändert");
                self.world.collision = map.collision();
                self.map = map;
                self.reload_error = None;
                tracing::info!("Karte neu geladen");
            }
            Err(e) => {
                tracing::warn!("Karte ungültig: {e:#}");
                self.reload_error = Some(format!("{e:#}"));
            }
        }
    }

    /// Startet eine Aufzeichnung ab dem Spawnpunkt (Figur wird zurückgesetzt).
    pub fn start_recording(&mut self) {
        self.respawn();
        let spawn = self.character().pos;
        self.recording = Some(Recording::new(
            &self.world.collision,
            spawn,
            self.world.tuning.clone(),
        ));
    }

    /// Beendet die Aufzeichnung und speichert sie. Liefert eine Statusmeldung.
    pub fn stop_recording(&mut self, reason: &str) -> Option<String> {
        let rec = self.recording.take()?;
        if rec.is_empty() {
            return Some("Aufzeichnung leer – verworfen".into());
        }
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let path = Path::new(RECORDINGS_DIR).join(format!("rec-{secs}.erec.toml"));
        let result = rec
            .to_toml()
            .map_err(anyhow::Error::from)
            .and_then(|text| std::fs::write(&path, text).map_err(anyhow::Error::from));
        Some(match result {
            Ok(()) => format!(
                "Aufzeichnung ({reason}): {} Ticks → {} (Golden mit ELORA_BLESS=1 erzeugen)",
                rec.len(),
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            Err(e) => format!("Aufzeichnung nicht gespeichert: {e:#}"),
        })
    }

    /// Setzt die Figur auf den Spawnpunkt zurück.
    pub fn respawn(&mut self) {
        let core = CharacterCore::new(spawn_point(&self.map));
        self.prev = core.clone();
        self.world.characters[self.player] = Some(core);
    }

    /// Lässt die Simulation um die vergangene Echtzeit laufen.
    pub fn advance(&mut self, elapsed: Duration, controls: &Controls) {
        self.accumulator += elapsed;
        let mut ticks = 0;
        while self.accumulator >= TICK {
            self.accumulator -= TICK;
            ticks += 1;
            if ticks > MAX_TICKS_PER_FRAME {
                self.accumulator = Duration::ZERO;
                break;
            }
            self.prev = self.character().clone();
            let input = controls.player_input();
            if let Some(rec) = &mut self.recording {
                rec.push(&input);
            }
            self.world.step(&[input]);
            if self.character().death {
                self.respawn();
            }
        }
    }

    /// Anteil des aktuellen Ticks (0..1) für die Interpolation.
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / TICK.as_secs_f32()
    }

    /// Interpolierte Position der Figur.
    pub fn render_pos(&self) -> Vec2 {
        self.prev.pos.lerp(self.character().pos, self.alpha())
    }
}

pub fn load_map(path: &Path) -> anyhow::Result<Map> {
    let src = std::fs::read_to_string(path)
        .with_context(|| format!("Karte {} nicht lesbar", path.display()))?;
    elora_map::parse_text_map(&src).with_context(|| format!("Karte {}", path.display()))
}

fn spawn_point(map: &Map) -> Vec2 {
    [
        EntityKind::Spawn,
        EntityKind::SpawnRed,
        EntityKind::SpawnBlue,
    ]
    .into_iter()
    .find_map(|k| map.entities_of(k).next())
    .map(elora_map::Entity::pos)
    .expect("validierte Karte hat einen Spawn")
}

/// Beobachtet die Kartendatei. Beobachtet wird das Verzeichnis, weil viele Editoren
/// beim Speichern die Datei ersetzen statt sie zu beschreiben.
struct MapWatcher {
    _watcher: notify::RecommendedWatcher,
    events: mpsc::Receiver<()>,
}

impl std::fmt::Debug for MapWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapWatcher").finish_non_exhaustive()
    }
}

impl MapWatcher {
    fn new(path: &Path) -> anyhow::Result<Self> {
        use notify::Watcher as _;
        let file = std::fs::canonicalize(path)?;
        let dir = file
            .parent()
            .context("Karte hat kein Verzeichnis")?
            .to_path_buf();
        let (tx, events) = mpsc::channel();
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res
                    && (event.kind.is_modify() || event.kind.is_create())
                    && event.paths.iter().any(|p| p == &file)
                {
                    let _ = tx.send(());
                }
            })?;
        watcher.watch(&dir, notify::RecursiveMode::NonRecursive)?;
        Ok(Self {
            _watcher: watcher,
            events,
        })
    }

    /// Wurde die Datei seit dem letzten Aufruf geändert?
    fn changed(&self) -> bool {
        self.events.try_iter().count() > 0
    }
}
