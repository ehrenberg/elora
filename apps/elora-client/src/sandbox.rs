//! Sandbox (M1/M2): Elora, dummies and pickups on a text map, fixed tick,
//! interpolation, hot reload and recording.

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
/// Protection against catch-up spirals after hangs.
const MAX_TICKS_PER_FRAME: u32 = 10;

/// Storage of the recordings: each one becomes a golden test of `elora-sim` there.
pub const RECORDINGS_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../crates/elora-sim/tests/recordings"
);

#[derive(Debug)]
pub struct Sandbox {
    pub map_path: PathBuf,
    pub map: Map,
    pub world: World,
    /// Slot of the human player.
    pub player: usize,
    /// Characters before the last tick (for interpolation), index = slot.
    pub prev: Vec<Option<CharacterCore>>,
    /// Last known position of Elora (the camera stays there while she is dead).
    last_pos: Vec2,
    accumulator: Duration,
    watcher: Option<MapWatcher>,
    /// Result of the last hot reload (error text for an invalid map).
    pub reload_error: Option<String>,
    /// Running input recording (M1.6).
    pub recording: Option<Recording>,
    /// Events since the last fetch (for effects).
    pending_events: Vec<Event>,
    /// Positions of enemies and loot before the last tick (interpolation), id → position.
    prev_creatures: std::collections::HashMap<u32, Vec2>,
    /// Game mode in the sandbox (E-075); `None` = free play without rules.
    pub rules: Option<Rules>,
    /// Notices of the rules (end of round etc.).
    pub notices: std::collections::VecDeque<ChatLine>,
    /// Enemies from the map (training, E-293): living id or tick of return.
    trainees: Vec<(Option<u32>, u64)>,
    trainee_tick: u64,
}

/// Defeated enemies in training return after this many ticks (5 s).
const TRAINEE_RESPAWN: u64 = 5 * TICKS_PER_SECOND as u64;

impl Sandbox {
    pub fn load(map_path: &Path, tuning: Tuning) -> anyhow::Result<Self> {
        let map = load_map(map_path)?;
        let (mut world, player) = fresh_world(&map, tuning);
        // training: all abilities on (E-293), can be switched off in the panel
        world.set_abilities(player, elora_sim::Abilities::ALL);
        let watcher = MapWatcher::new(map_path)
            .inspect_err(|e| tracing::warn!("hot reload unavailable: {e:#}"))
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
            trainees: Vec::new(),
            trainee_tick: 0,
        };
        s.sync_prev();
        Ok(s)
    }

    /// Place the enemy objects of the map and bring back defeated ones after [`TRAINEE_RESPAWN`]
    /// (only in free play; the adventure manages its enemies itself).
    fn tick_trainees(&mut self) {
        if self.world.adventure {
            return;
        }
        let objects: Vec<(usize, Vec2)> = self
            .map
            .adventure
            .objects
            .iter()
            .filter_map(|o| match &o.kind {
                elora_map::ObjectKind::Creature { kind, .. } => {
                    Some((self.world.creature_kind(kind)?, o.pos))
                }
                _ => None,
            })
            .collect();
        let tick = self.world.tick;
        // new world (map, mode, restart): from the start
        if tick < self.trainee_tick || self.trainees.len() != objects.len() {
            self.trainees = vec![(None, tick); objects.len()];
        }
        self.trainee_tick = tick;
        for (k, &(kind, pos)) in objects.iter().enumerate() {
            let (id, due) = &mut self.trainees[k];
            if let Some(i) = *id {
                if !self.world.creatures.iter().any(|c| c.id == i) {
                    *id = None;
                    *due = tick + TRAINEE_RESPAWN;
                }
            } else if tick >= *due {
                *id = self.world.add_creature(kind, pos);
            }
        }
    }

    /// Elora, if she is alive.
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

    /// Reloads the map when the file has changed. Elora keeps her
    /// state; pickups and dummies come from the new map.
    pub fn poll_reload(&mut self) {
        if !self.watcher.as_ref().is_some_and(MapWatcher::changed) {
            return;
        }
        match load_map(&self.map_path) {
            Ok(map) => {
                self.stop_recording("map changed");
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
                tracing::info!("map reloaded");
            }
            Err(e) => {
                tracing::warn!("map invalid: {e:#}");
                self.reload_error = Some(format!("{e:#}"));
            }
        }
    }

    /// Load another map from a file (debug panel); tuning, abilities and game mode
    /// stay, hot reload watches the new file.
    ///
    /// # Errors
    /// If the map is unreadable or invalid.
    pub fn switch_map(&mut self, path: &Path) -> anyhow::Result<()> {
        let map = load_map(path)?;
        self.stop_recording("map switched");
        let abilities = self.abilities();
        let adventure = self.world.adventure;
        let (mut world, player) = fresh_world(&map, self.world.tuning.clone());
        world.set_abilities(player, abilities);
        world.adventure = adventure;
        self.world = world;
        self.player = player;
        self.map = map;
        self.map_path = path.to_path_buf();
        self.watcher = MapWatcher::new(path)
            .inspect_err(|e| tracing::warn!("hot reload unavailable: {e:#}"))
            .ok();
        self.reload_error = None;
        self.notices.clear();
        self.accumulator = Duration::ZERO;
        if let Some(cfg) = self.rules.as_ref().map(|r| r.cfg.clone()) {
            self.rules = Some(Rules::new(cfg, &mut self.world, false));
        }
        self.sync_prev();
        Ok(())
    }

    /// Play a fully built world (adventure, A1.6); without hot reload, rules and
    /// recording. Returns the previous map for restoring.
    pub fn play_world(&mut self, map: Map, world: World, player: usize) -> Map {
        self.stop_recording("adventure");
        self.world = world;
        self.player = player;
        self.rules = None;
        self.watcher = None;
        self.reload_error = None;
        self.notices.clear();
        self.accumulator = Duration::ZERO;
        self.sync_prev();
        std::mem::replace(&mut self.map, map)
    }

    /// Switch hot reload of the current map file back on (after the adventure).
    pub fn watch_again(&mut self) {
        self.watcher = MapWatcher::new(&self.map_path)
            .inspect_err(|e| tracing::warn!("hot reload unavailable: {e:#}"))
            .ok();
    }

    /// Play a map from memory (test play from the editor, M6.9): fresh world in
    /// free play. Returns the previous map for restoring.
    pub fn play_map(&mut self, map: Map) -> Map {
        self.stop_recording("test game");
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

    /// Set or switch off the game mode (E-075). Starts a new match with a countdown.
    pub fn set_mode(&mut self, cfg: Option<RulesConfig>) {
        self.stop_recording("mode");
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

    /// Abilities of Elora (panel, A1.1) – kept when the map is reloaded.
    fn abilities(&self) -> elora_sim::Abilities {
        self.world
            .player(self.player)
            .map_or(elora_sim::Abilities::NONE, |p| p.abilities)
    }

    /// Tuning without instagib adjustments (sliders in the panel).
    fn base_tuning(&self) -> Tuning {
        let mut t = self.world.tuning.clone();
        if self.rules.as_ref().is_some_and(|r| r.cfg.instagib) {
            t.laser_damage = Tuning::default().laser_damage;
        }
        t
    }

    /// Names of the slots (sandbox: Elora and dummies).
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

    /// Game state for the display.
    pub fn game(&self) -> Option<GameView> {
        let r = self.rules.as_ref()?;
        elora_protocol::Snapshot::from_world(&self.world)
            .with_rules(&self.world, r)
            .game_view()
    }

    /// `kill` (key K, E-078).
    pub fn kill(&mut self) {
        match &self.rules {
            Some(r) => r.kill(&mut self.world, self.player),
            None => self.world.kill(self.player),
        }
    }

    /// Switch team.
    pub fn set_team(&mut self, team: elora_sim::Team) {
        if let Some(r) = &mut self.rules {
            r.set_team(&mut self.world, self.player, team);
        }
    }

    /// Notice in the chat history (translatable like the messages of a server).
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

    /// Starts a recording with a fresh world from the map (dummies and
    /// pickups in their initial state).
    pub fn start_recording(&mut self) {
        let (world, player) = fresh_world(&self.map, self.world.tuning.clone());
        self.world = world;
        self.player = player;
        self.sync_prev();
        self.recording = Some(Recording::new(&self.world));
    }

    /// Ends the recording and saves it. Returns a status message.
    pub fn stop_recording(&mut self, reason: &str) -> Option<String> {
        let rec = self.recording.take()?;
        if rec.is_empty() {
            return Some("recording empty – discarded".into());
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
                "recording ({reason}): {} ticks → {} (create the golden with ELORA_BLESS=1)",
                rec.len(),
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            Err(e) => format!("recording not saved: {e:#}"),
        })
    }

    /// Puts Elora at the best spawn point right away (key R).
    pub fn spawn_now(&mut self) {
        let pos = self.world.best_spawn().unwrap_or(self.last_pos);
        self.world.spawn_character(self.player, pos);
        self.sync_prev();
    }

    /// Runs the simulation for the elapsed real time.
    pub fn advance(&mut self, elapsed: Duration, controls: &mut Controls) {
        self.advance_with(elapsed, controls, |_| {});
    }

    /// Like [`Self::advance`]; `after` runs after every tick (adventure session, A1.6).
    pub fn advance_with(
        &mut self,
        elapsed: Duration,
        controls: &mut Controls,
        mut after: impl FnMut(&mut World),
    ) {
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
            after(&mut self.world);
            self.tick_trainees();
            if let Some(r) = &mut self.rules {
                r.update(&mut self.world);
            }
            self.rule_events();
            // broken tiles also in the map (graphics), E-230
            for e in &self.world.events {
                let (tx, ty, tile) = match *e {
                    Event::TileBroken { tx, ty } => (tx, ty, elora_sim::Tile::Air),
                    // root walls come and go (R2-M2.2)
                    Event::TileSet { tx, ty, tile } => (tx, ty, tile),
                    _ => continue,
                };
                if let (Ok(x), Ok(y)) = (usize::try_from(tx), usize::try_from(ty))
                    && x < self.map.width
                    && let Some(t) = self.map.tiles.get_mut(y * self.map.width + x)
                {
                    *t = tile;
                }
            }
            self.pending_events
                .extend(self.world.events.iter().cloned());
            if let Some(c) = self.character() {
                self.last_pos = c.core.pos;
            }
        }
    }

    /// Events since the last call.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.pending_events)
    }

    /// Fraction of the current tick (0..1) for interpolation.
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / TICK.as_secs_f32()
    }

    /// Core of the character in slot `i`: (current, before the last tick).
    pub fn cores(&self, i: usize) -> Option<(&CharacterCore, &CharacterCore)> {
        let cur = &self.world.character(i)?.core;
        let prev = self.prev.get(i).and_then(Option::as_ref).unwrap_or(cur);
        Some((cur, prev))
    }

    /// What to draw now.
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
        scene.add_shots(&self.world, alpha, 1.0, |_| true);
        scene.add_creatures(&self.world, &self.prev_creatures, alpha);
        scene.add_pickups(&self.world);
        scene.add_flags(&self.world);
        scene
    }

    /// Camera position: interpolated position of Elora, otherwise the last known one.
    pub fn render_pos(&self) -> Vec2 {
        self.cores(self.player)
            .map_or(self.last_pos, |(cur, prev)| {
                prev.pos.lerp(cur.pos, self.alpha())
            })
    }
}

/// World from the map plus a human player at the best spawn point.
/// Enemy kinds of the adventure (A1.2).
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
        std::fs::read(path).with_context(|| format!("map {} not readable", path.display()))?;
    elora_map::decode(&data).with_context(|| format!("map {}", path.display()))
}

/// Watches the map file. The directory is watched because many editors
/// replace the file on saving instead of writing to it.
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
        let dir = file.parent().context("map has no directory")?.to_path_buf();
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

    /// Was the file changed since the last call?
    fn changed(&self) -> bool {
        self.events.try_iter().count() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn training_map_brings_creatures_back() {
        let path = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../maps/training.emap"
        ));
        let mut sb = Sandbox::load(path, Tuning::default()).unwrap();
        let mut controls = Controls::default();
        sb.advance(TICK * 2, &mut controls);
        assert_eq!(sb.world.creatures.len(), 5, "enemies from the map");
        assert_eq!(
            sb.world.character(sb.player).unwrap().core.abilities,
            elora_sim::Abilities::ALL
        );
        sb.world.creatures.remove(0);
        sb.advance(TICK * 10, &mut controls);
        assert_eq!(sb.world.creatures.len(), 4);
        for _ in 0..TRAINEE_RESPAWN {
            sb.advance(TICK, &mut controls);
        }
        assert_eq!(sb.world.creatures.len(), 5, "back after 5 s");
    }

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
            "Elora stands at the spawn of the test map"
        );
        assert_eq!(sb.world.collision.width(), 5);
        sb.play_map(previous);
        assert_eq!(sb.map.name, "Sandbox");
        assert_eq!(sb.world.collision.width(), 48);
    }
}
