//! Sandbox (M1/M2): Elora, Dummies und Pickups auf einer Textkarte, fester Tick,
//! Interpolation, Hot-Reload und Aufzeichnung.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Context as _;
use elora_client::online::ChatLine;
use elora_client::scene::{Scene, SceneChar};
use elora_game::{GameEvent, Rules, RulesConfig, Winner};
use elora_map::Map;
use elora_protocol::GameView;
use elora_protocol::{Message, WinnerName};
use elora_sim::replay::Recording;
use elora_sim::{
    CharacterCore, Controller, Event, PlayerInput, TICKS_PER_SECOND, Tuning, Vec2, World,
};

use crate::controls::Controls;

pub const TICK: Duration = Duration::from_micros(1_000_000 / TICKS_PER_SECOND as u64);
/// Schutz gegen Aufholspiralen nach Hängern.
const MAX_TICKS_PER_FRAME: u32 = 10;

/// Ablage der Aufzeichnungen: jede wird dort zum Golden-Test von `elora-sim`.
pub const RECORDINGS_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../crates/elora-sim/tests/recordings"
);

#[derive(Debug)]
pub struct Sandbox {
    pub map_path: PathBuf,
    pub map: Map,
    pub world: World,
    /// Slot des menschlichen Spielers.
    pub player: usize,
    /// Figuren vor dem letzten Tick (für Interpolation), Index = Slot.
    pub prev: Vec<Option<CharacterCore>>,
    /// Letzte bekannte Position von Elora (Kamera bleibt dort, solange sie tot ist).
    last_pos: Vec2,
    accumulator: Duration,
    watcher: Option<MapWatcher>,
    /// Ergebnis des letzten Hot-Reloads (Fehlertext bei ungültiger Karte).
    pub reload_error: Option<String>,
    /// Laufende Eingabe-Aufzeichnung (M1.6).
    pub recording: Option<Recording>,
    /// Ereignisse seit dem letzten Abholen (für Effekte).
    pending_events: Vec<Event>,
    /// Positionen von Gegnern und Beute vor dem letzten Tick (Interpolation), Id → Position.
    prev_creatures: std::collections::HashMap<u32, Vec2>,
    /// Spielmodus in der Sandbox (E-075); `None` = freies Spiel ohne Regeln.
    pub rules: Option<Rules>,
    /// Hinweise der Regeln (Rundenende usw.).
    pub notices: std::collections::VecDeque<ChatLine>,
}

impl Sandbox {
    pub fn load(map_path: &Path, tuning: Tuning) -> anyhow::Result<Self> {
        let map = load_map(map_path)?;
        let (world, player) = fresh_world(&map, tuning);
        let watcher = MapWatcher::new(map_path)
            .inspect_err(|e| tracing::warn!("Hot-Reload nicht verfügbar: {e:#}"))
            .ok();
        let mut s = Self {
            map_path: map_path.to_path_buf(),
            map,
            world,
            player,
            prev: Vec::new(),
            last_pos: Vec2::ZERO,
            accumulator: Duration::ZERO,
            watcher,
            reload_error: None,
            recording: None,
            pending_events: Vec::new(),
            prev_creatures: std::collections::HashMap::new(),
            rules: None,
            notices: std::collections::VecDeque::new(),
        };
        s.sync_prev();
        Ok(s)
    }

    /// Elora, falls sie lebt.
    pub fn character(&self) -> Option<&elora_sim::Character> {
        self.world.character(self.player)
    }

    fn sync_prev(&mut self) {
        self.prev = self
            .world
            .players
            .iter()
            .map(|p| {
                p.as_ref()
                    .and_then(|p| p.character.as_ref())
                    .map(|c| c.core.clone())
            })
            .collect();
        self.prev_creatures = self
            .world
            .creatures
            .iter()
            .map(|c| (c.id, c.pos))
            .chain(self.world.loot.iter().map(|l| (l.id, l.pos)))
            .collect();
        if let Some(c) = self.character() {
            self.last_pos = c.core.pos;
        }
    }

    /// Lädt die Karte neu, wenn sich die Datei geändert hat. Elora behält ihren
    /// Zustand; Pickups und Dummies kommen aus der neuen Karte.
    pub fn poll_reload(&mut self) {
        if !self.watcher.as_ref().is_some_and(MapWatcher::changed) {
            return;
        }
        match load_map(&self.map_path) {
            Ok(map) => {
                self.stop_recording("Karte geändert");
                let elora = self.character().cloned();
                let abilities = self.abilities();
                let (mut world, player) = fresh_world(&map, self.world.tuning.clone());
                world.set_abilities(player, abilities);
                if let Some(p) = world.players[player].as_mut() {
                    p.character = elora;
                }
                self.world = world;
                self.player = player;
                self.map = map;
                self.reload_error = None;
                if let Some(cfg) = self.rules.as_ref().map(|r| r.cfg.clone()) {
                    self.rules = Some(Rules::new(cfg, &mut self.world, false));
                }
                self.sync_prev();
                tracing::info!("Karte neu geladen");
            }
            Err(e) => {
                tracing::warn!("Karte ungültig: {e:#}");
                self.reload_error = Some(format!("{e:#}"));
            }
        }
    }

    /// Karte aus dem Speicher spielen (Testspielen aus dem Editor, M6.9): frische Welt im
    /// freien Spiel. Liefert die bisherige Karte zum Wiederherstellen.
    pub fn play_map(&mut self, map: Map) -> Map {
        self.stop_recording("Testspiel");
        let abilities = self.abilities();
        let (mut world, player) = fresh_world(&map, self.world.tuning.clone());
        world.set_abilities(player, abilities);
        self.world = world;
        self.player = player;
        self.rules = None;
        self.reload_error = None;
        self.notices.clear();
        self.accumulator = Duration::ZERO;
        self.sync_prev();
        std::mem::replace(&mut self.map, map)
    }

    /// Spielmodus setzen oder abschalten (E-075). Startet ein neues Match mit Countdown.
    pub fn set_mode(&mut self, cfg: Option<RulesConfig>) {
        self.stop_recording("Modus");
        let tuning = self
            .rules
            .as_ref()
            .map_or_else(|| self.world.tuning.clone(), |_| self.base_tuning());
        let abilities = self.abilities();
        let (mut world, player) = fresh_world(&self.map, tuning);
        world.set_abilities(player, abilities);
        self.rules = cfg.map(|c| Rules::new(c, &mut world, false));
        self.world = world;
        self.player = player;
        self.sync_prev();
    }

    /// Fähigkeiten von Elora (Panel, A1.1) – bleiben beim Neuladen der Karte erhalten.
    fn abilities(&self) -> elora_sim::Abilities {
        self.world
            .player(self.player)
            .map_or(elora_sim::Abilities::NONE, |p| p.abilities)
    }

    /// Tuning ohne Instagib-Anpassungen (Regler im Panel).
    fn base_tuning(&self) -> Tuning {
        let mut t = self.world.tuning.clone();
        if self.rules.as_ref().is_some_and(|r| r.cfg.instagib) {
            t.laser_damage = Tuning::default().laser_damage;
        }
        t
    }

    /// Namen der Slots (Sandbox: Elora und Dummies).
    pub fn names(&self) -> std::collections::BTreeMap<usize, String> {
        self.world
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.as_ref()?;
                Some((
                    i,
                    if i == self.player {
                        "Elora".to_owned()
                    } else {
                        format!("Dummy {i}")
                    },
                ))
            })
            .collect()
    }

    /// Spielzustand für die Anzeige.
    pub fn game(&self) -> Option<GameView> {
        let r = self.rules.as_ref()?;
        elora_protocol::Snapshot::from_world(&self.world)
            .with_rules(&self.world, r)
            .game_view()
    }

    /// `kill` (Taste K, E-078).
    pub fn kill(&mut self) {
        match &self.rules {
            Some(r) => r.kill(&mut self.world, self.player),
            None => self.world.kill(self.player),
        }
    }

    /// Team wechseln.
    pub fn set_team(&mut self, team: elora_sim::Team) {
        if let Some(r) = &mut self.rules {
            r.set_team(&mut self.world, self.player, team);
        }
    }

    /// Hinweis im Chat-Verlauf (übersetzbar wie die Meldungen eines Servers).
    pub fn notice(&mut self, message: Message) {
        self.notices
            .push_back(ChatLine::notice(message, std::time::Instant::now()));
        while self.notices.len() > 50 {
            self.notices.pop_front();
        }
    }

    fn rule_events(&mut self) {
        let Some(r) = &mut self.rules else { return };
        let events = r.take_events();
        let mode = r.cfg.title();
        let names = self.names();
        let name_of = |i: usize| names.get(&i).cloned().unwrap_or_default();
        let winner = |w: Winner| match w {
            Winner::Player(i) => WinnerName::Player(name_of(i)),
            Winner::Team(t @ (elora_sim::Team::Red | elora_sim::Team::Blue)) => WinnerName::Team(t),
            _ => WinnerName::Nobody,
        };
        for e in events {
            let message = match e {
                GameEvent::MatchStarted => Message::MatchStarted { mode: mode.clone() },
                GameEvent::RoundOver(Winner::Draw) => Message::RoundDraw,
                GameEvent::RoundOver(w) => Message::RoundWon(winner(w)),
                GameEvent::MatchOver(w) => Message::MatchWon(winner(w)),
                GameEvent::SuddenDeath => Message::SuddenDeath,
                GameEvent::TeamChanged { player, team } => Message::TeamBalanced {
                    name: name_of(player),
                    team,
                },
                GameEvent::RoundStarted | GameEvent::NextMap => continue,
            };
            self.notice(message);
        }
    }

    /// Startet eine Aufzeichnung mit frischer Welt aus der Karte (Dummies und
    /// Pickups im Ausgangszustand).
    pub fn start_recording(&mut self) {
        let (world, player) = fresh_world(&self.map, self.world.tuning.clone());
        self.world = world;
        self.player = player;
        self.sync_prev();
        self.recording = Some(Recording::new(&self.world));
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

    /// Setzt Elora sofort an den besten Spawnpunkt (Taste R).
    pub fn spawn_now(&mut self) {
        let pos = self.world.best_spawn().unwrap_or(self.last_pos);
        self.world.spawn_character(self.player, pos);
        self.sync_prev();
    }

    /// Lässt die Simulation um die vergangene Echtzeit laufen.
    pub fn advance(&mut self, elapsed: Duration, controls: &mut Controls) {
        self.accumulator += elapsed;
        let mut ticks = 0;
        while self.accumulator >= TICK {
            self.accumulator -= TICK;
            ticks += 1;
            if ticks > MAX_TICKS_PER_FRAME {
                self.accumulator = Duration::ZERO;
                break;
            }
            self.sync_prev();
            let input = controls.player_input();
            if let Some(rec) = &mut self.recording {
                rec.push(&input);
            }
            let mut inputs = vec![PlayerInput::default(); self.world.players.len()];
            inputs[self.player] = input;
            self.world.step(&inputs);
            if let Some(r) = &mut self.rules {
                r.update(&mut self.world);
            }
            self.rule_events();
            // zerbrochene Tiles auch in der Karte (Grafik), E-230
            for e in &self.world.events {
                if let Event::TileBroken { tx, ty } = *e
                    && let (Ok(x), Ok(y)) = (usize::try_from(tx), usize::try_from(ty))
                    && x < self.map.width
                    && let Some(t) = self.map.tiles.get_mut(y * self.map.width + x)
                {
                    *t = elora_sim::Tile::Air;
                }
            }
            self.pending_events
                .extend(self.world.events.iter().cloned());
            if let Some(c) = self.character() {
                self.last_pos = c.core.pos;
            }
        }
    }

    /// Ereignisse seit dem letzten Aufruf.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.pending_events)
    }

    /// Anteil des aktuellen Ticks (0..1) für die Interpolation.
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / TICK.as_secs_f32()
    }

    /// Kern der Figur in Slot `i`: (aktuell, vor dem letzten Tick).
    pub fn cores(&self, i: usize) -> Option<(&CharacterCore, &CharacterCore)> {
        let cur = &self.world.character(i)?.core;
        let prev = self.prev.get(i).and_then(Option::as_ref).unwrap_or(cur);
        Some((cur, prev))
    }

    /// Was jetzt zu zeichnen ist.
    pub fn scene(&self) -> Scene {
        let alpha = self.alpha();
        let mut scene = Scene {
            camera: self.render_pos(),
            ..Scene::default()
        };
        for (i, p) in self.world.players.iter().enumerate() {
            let Some(p) = p else { continue };
            let Some(ch) = &p.character else { continue };
            let prev = self
                .prev
                .get(i)
                .and_then(Option::as_ref)
                .unwrap_or(&ch.core)
                .clone();
            scene.chars.push(SceneChar {
                slot: i,
                ch: ch.clone(),
                prev,
                alpha,
                dummy: matches!(p.controller, Controller::Dummy { .. }),
                local: i == self.player,
                team: p.team,
            });
        }
        scene.add_shots(&self.world, alpha, |_| true);
        scene.add_creatures(&self.world, &self.prev_creatures, alpha);
        scene.add_pickups(&self.world);
        scene.add_flags(&self.world);
        scene
    }

    /// Kamera-Position: interpolierte Position von Elora, sonst die letzte bekannte.
    pub fn render_pos(&self) -> Vec2 {
        self.cores(self.player)
            .map_or(self.last_pos, |(cur, prev)| {
                prev.pos.lerp(cur.pos, self.alpha())
            })
    }
}

/// Welt aus der Karte plus menschlicher Spieler am besten Spawnpunkt.
/// Gegnerarten des Abenteuers (A1.2).
const CREATURES_TOML: &str = include_str!("../../../assets/adventure/creatures.toml");

pub fn creature_kinds() -> Vec<elora_sim::CreatureKind> {
    elora_sim::creature::kinds_from_toml(CREATURES_TOML)
        .unwrap_or_else(|e| panic!("assets/adventure/creatures.toml: {e}"))
}

fn fresh_world(map: &Map, tuning: Tuning) -> (World, usize) {
    let mut world = map.world(tuning);
    world.creature_kinds = creature_kinds();
    let player = world.join();
    if let Some(pos) = world.best_spawn() {
        world.spawn_character(player, pos);
    }
    world.events.clear();
    (world, player)
}

pub fn load_map(path: &Path) -> anyhow::Result<Map> {
    let data =
        std::fs::read(path).with_context(|| format!("Karte {} nicht lesbar", path.display()))?;
    elora_map::decode(&data).with_context(|| format!("Karte {}", path.display()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_map_swaps_and_restores() {
        let path = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../maps/sandbox.emap"
        ));
        let mut sb = Sandbox::load(path, Tuning::default()).unwrap();
        let own = Map::from_rows("Test", &["#####", "#S..#", "#####"]).unwrap();
        let previous = sb.play_map(own.clone());
        assert_eq!(sb.map, own);
        assert!(
            sb.character().is_some(),
            "Elora steht am Spawn der Testkarte"
        );
        assert_eq!(sb.world.collision.width(), 5);
        sb.play_map(previous);
        assert_eq!(sb.map.name, "Sandbox");
        assert_eq!(sb.world.collision.width(), 48);
    }
}
