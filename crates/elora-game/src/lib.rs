//! Game rules of Elora (M4): modes, scores, rounds, win conditions, teams.
//!
//! Runs on top of the simulation: `Rules::update` evaluates the events after every
//! `World::step` and controls game state, respawn and pause of the world.
//! Reference: Teeworlds 0.7 `gamecontroller.cpp` and `gamemodes/*` (analysis §8),
//! with the deviations E-066 to E-079.

use std::collections::BTreeMap;

use elora_sim::entities::Flag;
use elora_sim::{Controller, DeathCause, Event, TICKS_PER_SECOND, Team, Tuning, Weapon, World};
use serde::{Deserialize, Serialize};

/// Game mode (E-014).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Dm,
    Tdm,
    Ctf,
    Lms,
    Lts,
}

impl Mode {
    pub const ALL: [Self; 5] = [Self::Dm, Self::Tdm, Self::Ctf, Self::Lms, Self::Lts];

    pub fn teams(self) -> bool {
        matches!(self, Self::Tdm | Self::Ctf | Self::Lts)
    }

    /// No respawn during a round.
    pub fn survival(self) -> bool {
        matches!(self, Self::Lms | Self::Lts)
    }

    pub fn flags(self) -> bool {
        self == Self::Ctf
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Dm => "DM",
            Self::Tdm => "TDM",
            Self::Ctf => "CTF",
            Self::Lms => "LMS",
            Self::Lts => "LTS",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|m| m.name().eq_ignore_ascii_case(s))
    }

    /// Default win condition (E-066, E-067): CTF 5 captures, otherwise 20 points.
    pub fn default_score_limit(self) -> u32 {
        if self == Self::Ctf { 5 } else { 20 }
    }

    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn from_index(i: u8) -> Option<Self> {
        Self::ALL.get(usize::from(i)).copied()
    }
}

/// Settings of the rules (server configuration, votes, console).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RulesConfig {
    pub mode: Mode,
    /// Laser only, one hit kills, no pickups (E-026, E-076).
    pub instagib: bool,
    /// `None` = default of the mode; `Some(0)` = no limit.
    pub score_limit: Option<u32>,
    /// Minutes, 0 = no time limit.
    pub time_limit: u32,
    /// Warmup after a map change (E-068).
    pub warmup_secs: u32,
    /// Countdown before every match / every round (E-068).
    pub countdown_secs: u32,
    /// Damage to team members (E-069).
    pub friendly_fire: bool,
    /// Minimum time until respawn in TDM (E-070).
    pub tdm_respawn_secs: u32,
    /// Balance unbalanced teams after this many seconds (E-073).
    pub team_balance_secs: u32,
    /// Swap teams after every match (E-074).
    pub match_swap: bool,
    /// Matches per map, then the next map (E-074).
    pub matches_per_map: u32,
}

impl Default for RulesConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Dm,
            instagib: false,
            score_limit: None,
            time_limit: 0,
            warmup_secs: 10,
            countdown_secs: 3,
            friendly_fire: true,
            tdm_respawn_secs: 3,
            team_balance_secs: 60,
            match_swap: true,
            matches_per_map: 1,
        }
    }
}

impl RulesConfig {
    pub fn score_limit(&self) -> u32 {
        self.score_limit
            .unwrap_or_else(|| self.mode.default_score_limit())
    }

    /// Display name, e.g. "iCTF".
    pub fn title(&self) -> String {
        format!(
            "{}{}",
            if self.instagib { "i" } else { "" },
            self.mode.name()
        )
    }
}

/// Game state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Warmup; `until = None`: waiting for enough players.
    Warmup {
        until: Option<u64>,
    },
    /// World frozen until `until`.
    Countdown {
        until: u64,
    },
    Running,
    RoundOver {
        until: u64,
    },
    MatchOver {
        until: u64,
    },
}

impl Phase {
    /// Tick at which the state ends (for the timer in the display).
    pub fn until(self) -> Option<u64> {
        match self {
            Self::Warmup { until } => until,
            Self::Countdown { until } | Self::RoundOver { until } | Self::MatchOver { until } => {
                Some(until)
            }
            Self::Running => None,
        }
    }

    /// Do points count right now?
    pub fn scoring(self) -> bool {
        self == Self::Running
    }
}

/// Score of a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    pub score: i32,
    pub kills: u32,
    pub deaths: u32,
}

/// Winner of a round or a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winner {
    Player(usize),
    Team(Team),
    Draw,
}

/// Events of the rules (for display and server).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    MatchStarted,
    RoundStarted,
    RoundOver(Winner),
    MatchOver(Winner),
    SuddenDeath,
    /// Player was moved to the other team for balancing.
    TeamChanged {
        player: usize,
        team: Team,
    },
    /// Rotation: load the next map (server).
    NextMap,
}

const MATCH_OVER_SECS: u64 = 10;
const ROUND_OVER_SECS: u64 = 5;

fn secs(s: u64) -> u64 {
    s * u64::from(TICKS_PER_SECOND)
}

#[derive(Debug, Clone)]
pub struct Rules {
    pub cfg: RulesConfig,
    pub phase: Phase,
    pub stats: BTreeMap<usize, Stats>,
    /// Team scores (red, blue); in CTF captures (E-067).
    pub team_score: [i32; 2],
    pub sudden_death: bool,
    pub match_start_tick: u64,
    pub match_count: u32,
    base_tuning: Tuning,
    unbalanced_since: Option<u64>,
    events: Vec<GameEvent>,
}

impl Rules {
    /// Rules for a world; `fresh_map`: after a map change with warmup (E-068).
    pub fn new(cfg: RulesConfig, world: &mut World, fresh_map: bool) -> Self {
        let mut r = Self {
            cfg,
            phase: Phase::Warmup { until: None },
            stats: BTreeMap::new(),
            team_score: [0; 2],
            sudden_death: false,
            match_start_tick: world.tick,
            match_count: 0,
            base_tuning: world.tuning.clone(),
            unbalanced_since: None,
            events: Vec::new(),
        };
        r.apply_to_world(world);
        let slots: Vec<usize> = world
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_some())
            .map(|(i, _)| i)
            .collect();
        for i in slots {
            r.assign_team(world, i);
            r.stats.insert(i, Stats::default());
        }
        if fresh_map && r.cfg.warmup_secs > 0 {
            r.phase = Phase::Warmup {
                until: Some(world.tick + secs(u64::from(r.cfg.warmup_secs))),
            };
        }
        world.reset_round();
        r.loadout_all(world);
        r
    }

    /// World settings from the configuration.
    fn apply_to_world(&self, world: &mut World) {
        world.friendly_fire = self.cfg.friendly_fire;
        world.pickups_enabled = !self.cfg.instagib;
        world.tuning = self.base_tuning.clone();
        if self.cfg.instagib {
            // one hit kills (more than health + armor)
            world.tuning.laser_damage = world.tuning.max_health + world.tuning.max_armor + 1;
        }
        world.flags.clear();
        if self.cfg.mode.flags()
            && let [Some(red), Some(blue)] = world.flag_stands
        {
            world.flags = vec![Flag::new(Team::Red, red), Flag::new(Team::Blue, blue)];
        }
        world.paused = false;
    }

    /// Events since the last fetch.
    pub fn take_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }

    fn team_sizes(world: &World) -> [usize; 2] {
        let mut n = [0; 2];
        for p in world.players.iter().flatten() {
            if let Some(t) = p.team.index() {
                n[t] += 1;
            }
        }
        n
    }

    /// Team on joining: into the smaller team (red on a tie, as in the original).
    fn assign_team(&self, world: &mut World, i: usize) {
        let sizes = Self::team_sizes(world);
        let Some(p) = world.players.get_mut(i).and_then(Option::as_mut) else {
            return;
        };
        if p.team == Team::Spectator {
            return;
        }
        p.team = if self.cfg.mode.teams() {
            if p.team.index().is_some() {
                p.team
            } else if sizes[0] > sizes[1] {
                Team::Blue
            } else {
                Team::Red
            }
        } else {
            Team::None
        };
    }

    /// New player (or dummy).
    pub fn on_join(&mut self, world: &mut World, i: usize) {
        self.assign_team(world, i);
        self.stats.insert(i, Stats::default());
        // Survival: whoever joins in the middle of a round waits for the next one
        if self.cfg.mode.survival()
            && self.phase == Phase::Running
            && let Some(p) = world.players.get_mut(i).and_then(Option::as_mut)
        {
            p.respawn_disabled = true;
            p.character = None;
        }
    }

    pub fn on_leave(&mut self, i: usize) {
        self.stats.remove(&i);
    }

    /// Change team or spectate (E-073). The character is removed without scoring.
    pub fn set_team(&mut self, world: &mut World, i: usize, team: Team) {
        let team = match team {
            Team::Spectator => Team::Spectator,
            t if self.cfg.mode.teams() => {
                if t.index().is_some() {
                    t
                } else {
                    Team::Red
                }
            }
            _ => Team::None,
        };
        if world.team(i) == team {
            return;
        }
        world.die(i, None, DeathCause::Game);
        if let Some(p) = world.players.get_mut(i).and_then(Option::as_mut) {
            p.team = team;
            p.respawn_disabled = self.cfg.mode.survival() && self.phase == Phase::Running;
        }
    }

    /// `kill` command (E-055, E-078): only while the game is running or during warmup.
    pub fn kill(&self, world: &mut World, i: usize) {
        if matches!(self.phase, Phase::Running | Phase::Warmup { .. }) {
            world.kill(i);
        }
    }

    fn stat(&mut self, i: usize) -> &mut Stats {
        self.stats.entry(i).or_default()
    }

    fn enough_players(&self, world: &World) -> bool {
        if self.cfg.mode.teams() {
            let n = Self::team_sizes(world);
            n[0] > 0 && n[1] > 0
        } else {
            world
                .players
                .iter()
                .flatten()
                .filter(|p| p.team != Team::Spectator)
                .count()
                > 1
        }
    }

    /// Call after every `World::step`.
    pub fn update(&mut self, world: &mut World) {
        let events = world.events.clone();
        for e in &events {
            self.on_event(world, e, &events);
        }
        self.tick_phase(world);
        self.balance(world);
    }

    fn on_event(&mut self, world: &mut World, e: &Event, all: &[Event]) {
        let scoring = self.phase.scoring();
        match *e {
            Event::Spawn { player, .. } => self.loadout(world, player),
            Event::Death {
                player,
                killer,
                cause,
                ..
            } => {
                if cause == DeathCause::Game {
                    return;
                }
                self.stat(player).deaths += 1;
                let tick = world.tick;
                if let Some(p) = world.players.get_mut(player).and_then(Option::as_mut) {
                    if self.cfg.mode == Mode::Tdm {
                        p.respawn_tick = p
                            .respawn_tick
                            .max(tick + secs(u64::from(self.cfg.tdm_respawn_secs)));
                    }
                    if self.cfg.mode.survival() && self.phase == Phase::Running {
                        p.respawn_disabled = true;
                    }
                }
                if !scoring {
                    return;
                }
                let teams = self.cfg.mode.teams();
                let victim_team = world.team(player);
                match killer {
                    Some(k) if k != player => {
                        let killer_team = world.team(k);
                        let teamkill = teams && killer_team.is_mate(victim_team);
                        if teamkill {
                            self.stat(k).score -= 1;
                        } else {
                            self.stat(k).score += 1;
                            self.stat(k).kills += 1;
                        }
                        if self.cfg.mode == Mode::Tdm
                            && let Some(t) = killer_team.index()
                        {
                            self.team_score[t] += if teamkill { -1 } else { 1 };
                        }
                        // CTF: flag carrier killed
                        let had_flag = all.iter().any(
                            |e| matches!(e, Event::FlagDrop { player: p, .. } if *p == player),
                        );
                        if had_flag && !teamkill {
                            self.stat(k).score += 1;
                        }
                    }
                    _ => {
                        // Suicide or death tile
                        self.stat(player).score -= 1;
                        if self.cfg.mode == Mode::Tdm
                            && let Some(t) = victim_team.index()
                        {
                            self.team_score[t] -= 1;
                        }
                    }
                }
            }
            Event::FlagGrab { player, .. } if scoring => self.stat(player).score += 1,
            Event::FlagReturn {
                player: Some(player),
                ..
            } if scoring => self.stat(player).score += 1,
            Event::FlagCapture { player, .. } if scoring => {
                self.stat(player).score += 5;
                if let Some(t) = world.team(player).index() {
                    self.team_score[t] += 1;
                }
            }
            _ => {}
        }
    }

    /// Starting equipment: normally only the hammer (E-025, E-071); Instagib only the laser
    /// (E-076).
    fn loadout(&self, world: &mut World, i: usize) {
        if !self.cfg.instagib {
            return;
        }
        if let Some(ch) = world.character_mut(i) {
            let a = &mut ch.arsenal;
            a.slots = Default::default();
            a.slots[Weapon::Laser.index()] = elora_sim::weapon::WeaponSlot {
                got: true,
                ammo: None,
            };
            a.active = Weapon::Laser;
            a.queued = None;
        }
    }

    fn loadout_all(&self, world: &mut World) {
        for i in 0..world.players.len() {
            self.loadout(world, i);
        }
    }

    fn start_match(&mut self, world: &mut World) {
        for s in self.stats.values_mut() {
            *s = Stats::default();
        }
        self.team_score = [0; 2];
        self.sudden_death = false;
        self.start_round(world);
        self.events.push(GameEvent::MatchStarted);
    }

    fn start_round(&mut self, world: &mut World) {
        for p in world.players.iter_mut().flatten() {
            p.respawn_disabled = false;
        }
        world.reset_round();
        self.loadout_all(world);
        let countdown = secs(u64::from(self.cfg.countdown_secs));
        if countdown > 0 {
            world.paused = true;
            self.phase = Phase::Countdown {
                until: world.tick + countdown,
            };
        } else {
            self.begin_running(world);
        }
        self.events.push(GameEvent::RoundStarted);
    }

    fn begin_running(&mut self, world: &mut World) {
        world.paused = false;
        self.phase = Phase::Running;
        self.match_start_tick = world.tick;
    }

    fn tick_phase(&mut self, world: &mut World) {
        let now = world.tick;
        match self.phase {
            Phase::Warmup { until: None } => {
                if self.enough_players(world) {
                    self.start_match(world);
                }
            }
            Phase::Warmup { until: Some(t) } => {
                if now >= t {
                    if self.enough_players(world) {
                        self.start_match(world);
                    } else {
                        self.phase = Phase::Warmup { until: None };
                    }
                }
            }
            Phase::Countdown { until } => {
                if now >= until {
                    self.begin_running(world);
                }
            }
            Phase::Running => {
                if !self.enough_players(world) {
                    self.phase = Phase::Warmup { until: None };
                    for p in world.players.iter_mut().flatten() {
                        p.respawn_disabled = false;
                    }
                } else if self.cfg.mode.survival() {
                    self.check_round(world);
                } else {
                    self.check_match(world);
                }
            }
            Phase::RoundOver { until } => {
                if now >= until {
                    self.start_round(world);
                }
            }
            Phase::MatchOver { until } => {
                if now >= until {
                    self.match_count += 1;
                    if self.match_count >= self.cfg.matches_per_map.max(1) {
                        self.events.push(GameEvent::NextMap);
                    }
                    if self.cfg.match_swap && self.cfg.mode.teams() {
                        for p in world.players.iter_mut().flatten() {
                            p.team = p.team.other();
                        }
                    }
                    self.start_match(world);
                }
            }
        }
    }

    fn time_up(&self, world: &World) -> bool {
        self.cfg.time_limit > 0
            && world.tick - self.match_start_tick >= secs(u64::from(self.cfg.time_limit) * 60)
    }

    fn end_match(&mut self, world: &World, winner: Winner) {
        self.phase = Phase::MatchOver {
            until: world.tick + secs(MATCH_OVER_SECS),
        };
        self.events.push(GameEvent::MatchOver(winner));
    }

    /// Win condition (DM/TDM/CTF): score or time limit, sudden death on a tie.
    fn check_match(&mut self, world: &World) {
        let limit = i64::from(self.cfg.score_limit());
        let reached_by = |score: i32| limit > 0 && i64::from(score) >= limit;
        if self.cfg.mode.teams() {
            let [red, blue] = self.team_score;
            if reached_by(red) || reached_by(blue) || self.time_up(world) {
                if red == blue {
                    if !self.sudden_death {
                        self.sudden_death = true;
                        self.events.push(GameEvent::SuddenDeath);
                    }
                } else {
                    let team = if red > blue { Team::Red } else { Team::Blue };
                    self.end_match(world, Winner::Team(team));
                }
            }
        } else {
            let top = self
                .stats
                .iter()
                .filter(|(i, _)| world.player(**i).is_some())
                .map(|(_, s)| s.score)
                .max()
                .unwrap_or(0);
            let leaders: Vec<usize> = self
                .stats
                .iter()
                .filter(|(i, s)| s.score == top && world.player(**i).is_some())
                .map(|(i, _)| *i)
                .collect();
            if reached_by(top) || self.time_up(world) {
                if let [only] = leaders[..] {
                    self.end_match(world, Winner::Player(only));
                } else if !self.sudden_death {
                    self.sudden_death = true;
                    self.events.push(GameEvent::SuddenDeath);
                }
            }
        }
    }

    /// Round win (LMS/LTS): last player or last team.
    fn check_round(&mut self, world: &World) {
        let alive: Vec<(usize, Team)> = world
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let p = p.as_ref()?;
                let in_game =
                    p.team != Team::Spectator && (!p.respawn_disabled || p.character.is_some());
                in_game.then_some((i, p.team))
            })
            .collect();
        let time_up = self.time_up(world);
        let winner = if self.cfg.mode == Mode::Lms {
            if time_up {
                for &(i, _) in &alive {
                    self.stat(i).score += 1;
                }
                Some(Winner::Draw)
            } else {
                match alive[..] {
                    [] => Some(Winner::Draw),
                    [(i, _)] => {
                        self.stat(i).score += 1;
                        Some(Winner::Player(i))
                    }
                    _ => None,
                }
            }
        } else {
            let red = alive.iter().filter(|a| a.1 == Team::Red).count();
            let blue = alive.iter().filter(|a| a.1 == Team::Blue).count();
            if red + blue == 0 || time_up {
                self.team_score[0] += 1;
                self.team_score[1] += 1;
                Some(Winner::Draw)
            } else if red == 0 {
                self.team_score[1] += 1;
                Some(Winner::Team(Team::Blue))
            } else if blue == 0 {
                self.team_score[0] += 1;
                Some(Winner::Team(Team::Red))
            } else {
                None
            }
        };
        let Some(winner) = winner else { return };
        // Match over?
        let limit = i64::from(self.cfg.score_limit());
        let match_winner = if self.cfg.mode.teams() {
            let [red, blue] = self.team_score;
            (limit > 0 && (i64::from(red) >= limit || i64::from(blue) >= limit) && red != blue)
                .then_some(Winner::Team(if red > blue {
                    Team::Red
                } else {
                    Team::Blue
                }))
        } else {
            let top = self.stats.values().map(|s| s.score).max().unwrap_or(0);
            let leaders: Vec<usize> = self
                .stats
                .iter()
                .filter(|(_, s)| s.score == top)
                .map(|(i, _)| *i)
                .collect();
            match leaders[..] {
                [only] if limit > 0 && i64::from(top) >= limit => Some(Winner::Player(only)),
                _ => None,
            }
        };
        if let Some(w) = match_winner {
            self.end_match(world, w);
        } else {
            self.phase = Phase::RoundOver {
                until: world.tick + secs(ROUND_OVER_SECS),
            };
            self.events.push(GameEvent::RoundOver(winner));
        }
    }

    /// Automatic team balancing (E-073, not in survival modes).
    fn balance(&mut self, world: &mut World) {
        if !self.cfg.mode.teams() || self.cfg.mode.survival() || self.phase != Phase::Running {
            self.unbalanced_since = None;
            return;
        }
        let n = Self::team_sizes(world);
        if n[0].abs_diff(n[1]) <= 1 {
            self.unbalanced_since = None;
            return;
        }
        let since = *self.unbalanced_since.get_or_insert(world.tick);
        if world.tick < since + secs(u64::from(self.cfg.team_balance_secs)) {
            return;
        }
        let (big, small) = if n[0] > n[1] {
            (Team::Red, Team::Blue)
        } else {
            (Team::Blue, Team::Red)
        };
        let carriers: Vec<usize> = world.flags.iter().filter_map(|f| f.carrier).collect();
        // Player with the fewest points switches (no flag carrier)
        let candidate = world
            .players
            .iter()
            .enumerate()
            .filter(|(i, p)| {
                p.as_ref()
                    .is_some_and(|p| p.team == big && !matches!(p.controller, Controller::Remote))
                    && !carriers.contains(i)
            })
            .min_by_key(|(i, _)| self.stats.get(i).map_or(0, |s| s.score))
            .map(|(i, _)| i);
        if let Some(i) = candidate {
            self.set_team(world, i, small);
            self.events.push(GameEvent::TeamChanged {
                player: i,
                team: small,
            });
        }
        self.unbalanced_since = None;
    }
}

#[cfg(test)]
mod tests;
