//! Server logic of Elora: simulate world and rules, apply inputs, send snapshots
//! (M3.5); game modes, chat, team choice, votes, rotation, console (M4).
//!
//! Independent of the socket so that tests can run the server over an in-memory network.

pub mod config;
mod console;
pub mod paths;
mod vote;

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::net::IpAddr;
use std::time::{Duration, Instant};

use elora_game::{GameEvent, Rules, RulesConfig, Winner};
use elora_net::{DisconnectReason, Keypair, ServerEndpoint, ServerEvent, Socket};
use elora_protocol::msg::MAX_CHAT;
use elora_protocol::{
    ClientMsg, InfoPlayer, MAP_CHUNK, MapChecksum, Message, PROTOCOL_VERSION, ServerInfo,
    ServerMsg, Skin, Snapshot, WinnerName, reason,
};
use elora_sim::{Controller, Event, PlayerInput, TICKS_PER_SECOND, Team, Tuning, World};

pub use config::ServerConfig;
use vote::Vote;

/// Duration of a tick.
pub const TICK: Duration = Duration::from_micros(1_000_000 / TICKS_PER_SECOND as u64);
/// This many snapshots are kept as delta base (≈ 2 s).
const HISTORY: usize = 100;
/// Inputs further than this many ticks in the future are dropped.
const MAX_INPUT_AHEAD: u64 = 2 * TICKS_PER_SECOND as u64;
/// Spam protection: minimum interval between two chat messages.
const CHAT_INTERVAL: Duration = Duration::from_millis(700);
/// The info for the server browser is refreshed at this interval.
const INFO_INTERVAL: Duration = Duration::from_secs(1);
/// Minimum interval between two emotes of a player (spam protection).
const EMOTE_INTERVAL: Duration = Duration::from_millis(1000);
/// Ban after a kick by vote (E-077).
const KICK_BAN: Duration = Duration::from_mins(5);

/// A map known to the server (rotation and votes).
#[derive(Debug, Clone)]
pub struct MapEntry {
    pub name: String,
    /// Map file (`.emap`).
    pub data: Vec<u8>,
    pub checksum: MapChecksum,
}

impl MapEntry {
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            checksum: elora_map::checksum(&data),
            data,
        }
    }

    /// Number of chunks for the download.
    fn chunks(&self) -> usize {
        self.data.len().div_ceil(MAP_CHUNK)
    }
}

#[derive(Debug)]
struct Client {
    slot: Option<usize>,
    /// Has sent `Join` and is loading the map (waits for `MapReady`, M6.5).
    loading: bool,
    /// Has been in the game before (join notice only the first time).
    entered: bool,
    /// Sent map chunks for the current map (limits requests).
    chunks_sent: usize,
    name: String,
    skin: Skin,
    inputs: BTreeMap<u64, PlayerInput>,
    last_input: PlayerInput,
    acked: Option<u64>,
    events: Vec<Event>,
    last_chat: Option<Instant>,
    last_emote: Option<Instant>,
    ip: Option<IpAddr>,
}

impl Client {
    fn new(ip: Option<IpAddr>) -> Self {
        Self {
            slot: None,
            loading: false,
            entered: false,
            chunks_sent: 0,
            name: String::new(),
            skin: Skin::default(),
            inputs: BTreeMap::new(),
            last_input: PlayerInput::default(),
            acked: None,
            events: Vec::new(),
            last_chat: None,
            last_emote: None,
            ip,
        }
    }
}

/// The game server.
pub struct GameServer<S: Socket> {
    pub endpoint: ServerEndpoint<S>,
    pub world: World,
    pub rules: Rules,
    maps: Vec<MapEntry>,
    rotation: Vec<String>,
    map_index: usize,
    /// Current map.
    map: MapEntry,
    base_tuning: Tuning,
    high_bandwidth: bool,
    votes_enabled: bool,
    clients: HashMap<u32, Client>,
    history: VecDeque<Snapshot>,
    start: Instant,
    vote: Option<Vote>,
    bans: HashMap<IpAddr, Instant>,
    /// Output for the console (e.g. chat), to be fetched by the program.
    pub log: Vec<String>,
    /// Display name and maximum count for the server browser (M7.6).
    name: String,
    max_clients: usize,
    /// When the info for the browser was last refreshed.
    info_at: Option<Instant>,
}

impl<S: Socket> std::fmt::Debug for GameServer<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameServer")
            .field("tick", &self.world.tick)
            .field("clients", &self.clients.len())
            .finish_non_exhaustive()
    }
}

fn load_world(data: &[u8], tuning: Tuning) -> anyhow::Result<World> {
    Ok(elora_map::decode(data)?.world(tuning))
}

impl<S: Socket> GameServer<S> {
    /// `maps[0]` is the start map; all maps are available for rotation and votes.
    ///
    /// # Errors
    /// On an invalid start map.
    pub fn new(
        socket: S,
        key: Keypair,
        config: &ServerConfig,
        maps: Vec<MapEntry>,
        tuning: Tuning,
        now: Instant,
    ) -> anyhow::Result<Self> {
        let first = maps
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no map"))?;
        let mut world = load_world(&first.data, tuning.clone())?;
        let rules = Rules::new(config.rules.clone(), &mut world, true);
        let rotation = if config.rotation.is_empty() {
            vec![first.name.clone()]
        } else {
            config.rotation.clone()
        };
        Ok(Self {
            endpoint: ServerEndpoint::new(socket, key, config.max_clients),
            world,
            rules,
            maps,
            rotation,
            map_index: 0,
            map: first,
            base_tuning: tuning,
            high_bandwidth: config.high_bandwidth,
            votes_enabled: config.votes,
            clients: HashMap::new(),
            history: VecDeque::new(),
            start: now,
            vote: None,
            bans: HashMap::new(),
            log: Vec::new(),
            name: config.name.clone(),
            max_clients: config.max_clients,
            info_at: None,
        })
    }

    /// Start time of tick `tick`.
    pub fn tick_start(&self, tick: u64) -> Instant {
        self.start + TICK * u32::try_from(tick).unwrap_or(u32::MAX)
    }

    pub fn player_count(&self) -> usize {
        self.clients.values().filter(|c| c.slot.is_some()).count()
    }

    pub fn map_name(&self) -> &str {
        &self.map.name
    }

    pub fn map_names(&self) -> Vec<String> {
        self.maps.iter().map(|m| m.name.clone()).collect()
    }

    /// Process the network and simulate due ticks. Call often (≥ 500 Hz).
    pub fn update(&mut self, now: Instant) {
        for event in self.endpoint.poll(now) {
            self.handle(event, now);
        }
        while now >= self.tick_start(self.world.tick + 1) {
            self.tick(now);
        }
        self.update_vote(now);
        if self.info_at.is_none_or(|t| now - t >= INFO_INTERVAL) {
            self.info_at = Some(now);
            let info = self.server_info().encode();
            self.endpoint.set_info(info);
        }
        self.endpoint.flush(now);
    }

    /// Info for the server browser: name, map, mode, players (M7.6).
    pub fn server_info(&self) -> ServerInfo {
        let players = self
            .world
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let p = p.as_ref()?;
                Some(InfoPlayer {
                    name: self.name_of(i),
                    score: self.rules.stats.get(&i).map_or(0, |s| s.score),
                    team: p.team,
                    dummy: matches!(p.controller, Controller::Dummy { .. }),
                })
            })
            .collect();
        ServerInfo {
            version: PROTOCOL_VERSION,
            name: self.name.clone(),
            map: self.map.name.clone(),
            mode: self.rules.cfg.title(),
            clients: u32::try_from(self.player_count()).unwrap_or(u32::MAX),
            max_clients: u32::try_from(self.max_clients).unwrap_or(u32::MAX),
            players,
        }
    }

    /// Next point in time at which a tick is due.
    pub fn next_tick_at(&self) -> Instant {
        self.tick_start(self.world.tick + 1)
    }

    fn broadcast(&mut self, msg: &ServerMsg) {
        let data = msg.encode();
        let ids: Vec<u32> = self
            .clients
            .iter()
            .filter(|(_, c)| c.slot.is_some() || c.loading)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.endpoint.send(id, &data, true);
        }
    }

    /// Notice to everyone (and into the server log).
    pub fn notice(&mut self, message: Message) {
        tracing::info!("{message}");
        self.broadcast(&ServerMsg::Notice(message));
    }

    fn name_of(&self, slot: usize) -> String {
        if let Some(c) = self.clients.values().find(|c| c.slot == Some(slot)) {
            return c.name.clone();
        }
        match self.world.player(slot).map(|p| &p.controller) {
            Some(Controller::Dummy { .. }) => format!("Dummy {slot}"),
            _ => format!("Player {slot}"),
        }
    }

    /// Skin of a slot; dummies and unknown slots: default.
    fn skin_of(&self, slot: usize) -> Skin {
        self.clients
            .values()
            .find(|c| c.slot == Some(slot))
            .map_or_else(Skin::default, |c| c.skin)
    }

    fn client_by_slot(&self, slot: usize) -> Option<u32> {
        self.clients
            .iter()
            .find(|(_, c)| c.slot == Some(slot))
            .map(|(id, _)| *id)
    }

    fn handle(&mut self, event: ServerEvent, now: Instant) {
        match event {
            ServerEvent::Connected { id, addr } => {
                self.bans.retain(|_, until| *until > now);
                if self.bans.contains_key(&addr.ip()) {
                    tracing::info!(%addr, "banned");
                    self.endpoint.disconnect(id, reason::BANNED, now);
                    return;
                }
                tracing::info!(%addr, id, "connected");
                self.clients.insert(id, Client::new(Some(addr.ip())));
            }
            ServerEvent::Disconnected { id, reason } => {
                if let Some(c) = self.clients.remove(&id) {
                    tracing::info!(id, name = %c.name, ?reason, "disconnected");
                    if let Some(slot) = c.slot {
                        self.world.die(slot, None, elora_sim::DeathCause::Game);
                        self.world.remove(slot);
                        self.rules.on_leave(slot);
                        let slot = u32::try_from(slot).unwrap_or(0);
                        self.broadcast(&ServerMsg::PlayerInfo {
                            slot,
                            name: None,
                            skin: Skin::default(),
                        });
                        self.notice(Message::Left { name: c.name });
                    }
                    if let Some(v) = &mut self.vote {
                        v.forget(id);
                    }
                }
            }
            ServerEvent::Message { id, data, .. } => match ClientMsg::decode(&data) {
                Ok(msg) => self.message(id, msg, now),
                Err(e) => {
                    tracing::warn!(id, %e, "invalid message");
                    self.endpoint.disconnect(id, reason::INVALID_MESSAGE, now);
                }
            },
        }
    }

    /// Announce the map; the client responds with `MapReady` as soon as it has it (M6.5).
    fn start_loading(&mut self, id: u32) {
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        client.loading = true;
        client.chunks_sent = 0;
        let info = ServerMsg::MapInfo {
            name: self.map.name.clone(),
            checksum: self.map.checksum,
            size: u32::try_from(self.map.data.len()).unwrap_or(u32::MAX),
        };
        self.endpoint.send(id, &info.encode(), true);
    }

    /// Send a map chunk – only to loading clients, each chunk at most twice on average.
    fn on_map_request(&mut self, id: u32, chunk: u32) {
        let total = self.map.chunks();
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        let index = chunk as usize;
        if !client.loading || index >= total || client.chunks_sent >= 2 * total {
            return;
        }
        client.chunks_sent += 1;
        let end = (index * MAP_CHUNK + MAP_CHUNK).min(self.map.data.len());
        let msg = ServerMsg::MapChunk {
            index: chunk,
            data: self.map.data[index * MAP_CHUNK..end].to_vec(),
        };
        self.endpoint.send(id, &msg.encode(), true);
    }

    /// Bring a loading client into the game: slot, `Welcome`, names.
    fn enter(&mut self, id: u32) {
        let slot = self.world.join();
        self.rules.on_join(&mut self.world, slot);
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        client.loading = false;
        client.slot = Some(slot);
        let first = !std::mem::replace(&mut client.entered, true);
        let (name, skin) = (client.name.clone(), client.skin);
        tracing::info!(id, slot, %name, "joined");
        self.welcome(id);
        let info = ServerMsg::PlayerInfo {
            slot: u32::try_from(slot).unwrap_or(0),
            name: Some(name.clone()),
            skin,
        };
        self.broadcast(&info);
        if first {
            self.notice(Message::Joined { name });
        }
    }

    fn welcome(&mut self, id: u32) {
        let Some(slot) = self.clients.get(&id).and_then(|c| c.slot) else {
            return;
        };
        let welcome = ServerMsg::Welcome {
            slot: u32::try_from(slot).unwrap_or(0),
            tick: self.world.tick,
            map_name: self.map.name.clone(),
            map_checksum: self.map.checksum,
            tuning: self.world.tuning.clone(),
            high_bandwidth: self.high_bandwidth,
        };
        self.endpoint.send(id, &welcome.encode(), true);
        // names of all players and dummies
        for (i, p) in self.world.players.iter().enumerate() {
            if p.is_some() {
                let msg = ServerMsg::PlayerInfo {
                    slot: u32::try_from(i).unwrap_or(0),
                    name: Some(self.name_of(i)),
                    skin: self.skin_of(i),
                };
                self.endpoint.send(id, &msg.encode(), true);
            }
        }
        let vote = ServerMsg::Vote(
            self.vote
                .as_ref()
                .map(|v| v.info(self.voters(), Instant::now())),
        );
        self.endpoint.send(id, &vote.encode(), true);
    }

    /// Distribute an emote, at most one per [`EMOTE_INTERVAL`].
    fn on_emote(&mut self, id: u32, emote: u8, now: Instant) {
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        let Some(slot) = client.slot else { return };
        if client.last_emote.is_some_and(|t| now - t < EMOTE_INTERVAL) {
            return;
        }
        client.last_emote = Some(now);
        self.broadcast(&ServerMsg::Emote {
            slot: u32::try_from(slot).unwrap_or(0),
            emote,
        });
    }

    fn on_set_skin(&mut self, id: u32, skin: Skin) {
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        let Some(slot) = client.slot else { return };
        client.skin = skin;
        let info = ServerMsg::PlayerInfo {
            slot: u32::try_from(slot).unwrap_or(0),
            name: Some(client.name.clone()),
            skin,
        };
        self.broadcast(&info);
    }

    fn message(&mut self, id: u32, msg: ClientMsg, now: Instant) {
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        let slot = client.slot;
        match msg {
            ClientMsg::Join {
                version,
                name,
                skin,
            } => {
                if version != PROTOCOL_VERSION {
                    self.endpoint.disconnect(id, reason::WRONG_VERSION, now);
                    return;
                }
                if slot.is_some() || client.loading {
                    return;
                }
                let name: String = name.chars().filter(|c| !c.is_control()).take(16).collect();
                client.name = if name.trim().is_empty() {
                    "Elora".to_owned()
                } else {
                    name
                };
                client.skin = skin;
                self.start_loading(id);
            }
            ClientMsg::MapRequest { chunk } => self.on_map_request(id, chunk),
            ClientMsg::MapReady => {
                if self.clients.get(&id).is_some_and(|c| c.loading) {
                    self.enter(id);
                }
            }
            ClientMsg::Input { ack, inputs } => self.on_input(id, ack, inputs, now),
            ClientMsg::Emote(emote) => self.on_emote(id, emote, now),
            ClientMsg::SetSkin(skin) => self.on_set_skin(id, skin),
            ClientMsg::Leave => self.endpoint.disconnect(id, reason::LEFT, now),
            ClientMsg::Chat { team, text } => {
                let Some(slot) = slot else { return };
                if client.last_chat.is_some_and(|t| now - t < CHAT_INTERVAL) {
                    return;
                }
                client.last_chat = Some(now);
                let text: String = text
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(MAX_CHAT)
                    .collect();
                if text.trim().is_empty() {
                    return;
                }
                self.chat(Some(slot), team, &text);
            }
            ClientMsg::SetTeam(t) => {
                if let Some(slot) = slot {
                    self.rules.set_team(&mut self.world, slot, t);
                    let name = self.name_of(slot);
                    let team = self.world.team(slot);
                    self.notice(Message::TeamJoined { name, team });
                }
            }
            ClientMsg::Kill => {
                if let Some(slot) = slot {
                    self.rules.kill(&mut self.world, slot);
                }
            }
            ClientMsg::CallVote(kind) => {
                if slot.is_some() {
                    self.call_vote(id, kind, now);
                }
            }
            ClientMsg::Vote(yes) => {
                if let Some(v) = &mut self.vote {
                    v.cast(id, yes);
                    self.send_vote_status(now);
                }
            }
        }
    }

    fn on_input(
        &mut self,
        id: u32,
        ack: Option<u64>,
        inputs: Vec<(u64, PlayerInput)>,
        now: Instant,
    ) {
        let tick = self.world.tick;
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        client.acked = ack.max(client.acked);
        let mut newest = None;
        for (t, input) in inputs {
            if t > tick && t <= tick + MAX_INPUT_AHEAD {
                client.inputs.insert(t, input);
            }
            if newest.is_none_or(|n| t > n) {
                newest = Some(t);
            }
        }
        // feedback like `INPUTTIMING`: time until this tick is processed
        if let Some(t) = newest {
            let due = self.start + TICK * u32::try_from(t).unwrap_or(u32::MAX);
            let left = due.saturating_duration_since(now).as_millis() as i64
                - now.saturating_duration_since(due).as_millis() as i64;
            let msg = ServerMsg::InputTiming {
                tick: t,
                time_left_ms: left.clamp(-10_000, 10_000) as i32,
            };
            self.endpoint.send(id, &msg.encode(), false);
        }
    }

    /// Distribute a chat message; team chat only to the own team.
    pub fn chat(&mut self, from: Option<usize>, team: bool, text: &str) {
        let sender_team = from.map_or(Team::None, |s| self.world.team(s));
        let who = from.map_or_else(|| "Server".to_owned(), |s| self.name_of(s));
        tracing::info!("[chat{}] {who}: {text}", if team { "/team" } else { "" });
        self.log.push(format!("{who}: {text}"));
        let msg = ServerMsg::Chat {
            from: from.and_then(|s| u32::try_from(s).ok()),
            team,
            text: text.to_owned(),
        }
        .encode();
        let targets: Vec<u32> = self
            .clients
            .iter()
            .filter(|(_, c)| {
                c.slot
                    .is_some_and(|s| !team || self.world.team(s) == sender_team)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in targets {
            self.endpoint.send(id, &msg, true);
        }
    }

    fn tick(&mut self, now: Instant) {
        let next = self.world.tick + 1;
        let mut inputs = vec![PlayerInput::default(); self.world.players.len()];
        for c in self.clients.values_mut() {
            let Some(slot) = c.slot else { continue };
            // input for exactly this tick, otherwise the last known one
            let stale: Vec<u64> = c.inputs.range(..=next).map(|(t, _)| *t).collect();
            for t in stale {
                if let Some(i) = c.inputs.remove(&t) {
                    c.last_input = i;
                }
            }
            if let Some(i) = inputs.get_mut(slot) {
                *i = c.last_input;
            }
        }
        self.world.step(&inputs);
        self.rules.update(&mut self.world);
        for c in self.clients.values_mut() {
            c.events.extend(self.world.events.iter().cloned());
        }
        for e in self.rules.take_events() {
            self.game_event(&e, now);
        }

        let snap_now = self.high_bandwidth || self.world.tick.is_multiple_of(2);
        if snap_now {
            let snap = Snapshot::from_world(&self.world).with_rules(&self.world, &self.rules);
            for (&id, c) in &mut self.clients {
                if c.slot.is_none() {
                    continue;
                }
                let base = c
                    .acked
                    .and_then(|t| self.history.iter().find(|s| s.tick == t));
                let msg = ServerMsg::snapshot(&snap, base, std::mem::take(&mut c.events));
                self.endpoint.send(id, &msg.encode(), false);
            }
            self.history.push_back(snap);
            while self.history.len() > HISTORY {
                self.history.pop_front();
            }
        }
    }

    fn winner_name(&self, w: Winner) -> WinnerName {
        match w {
            Winner::Player(i) => WinnerName::Player(self.name_of(i)),
            Winner::Team(t @ (Team::Red | Team::Blue)) => WinnerName::Team(t),
            Winner::Team(_) | Winner::Draw => WinnerName::Nobody,
        }
    }

    fn game_event(&mut self, e: &GameEvent, now: Instant) {
        match *e {
            GameEvent::MatchStarted => {
                self.notice(Message::MatchStarted {
                    mode: self.rules.cfg.title(),
                });
            }
            GameEvent::RoundStarted => {}
            GameEvent::RoundOver(w) => {
                let message = match w {
                    Winner::Draw => Message::RoundDraw,
                    w => Message::RoundWon(self.winner_name(w)),
                };
                self.notice(message);
            }
            GameEvent::MatchOver(w) => {
                self.notice(Message::MatchWon(self.winner_name(w)));
            }
            GameEvent::SuddenDeath => self.notice(Message::SuddenDeath),
            GameEvent::TeamChanged { player, team } => {
                self.notice(Message::TeamBalanced {
                    name: self.name_of(player),
                    team,
                });
            }
            GameEvent::NextMap => {
                if self.rotation.len() > 1 {
                    self.map_index = (self.map_index + 1) % self.rotation.len();
                    let name = self.rotation[self.map_index].clone();
                    if let Err(e) = self.change_map(&name, now) {
                        tracing::warn!("Rotation: {e:#}");
                    }
                }
            }
        }
    }

    /// Change the map: new world, all players load the map and rejoin (E-074, M6.5).
    ///
    /// # Errors
    /// If the map is unknown or invalid.
    pub fn change_map(&mut self, name: &str, now: Instant) -> anyhow::Result<()> {
        let entry = self
            .maps
            .iter()
            .find(|m| m.name == name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("map `{name}` unknown"))?;
        let mut world = load_world(&entry.data, self.base_tuning.clone())?;
        // time keeps running: tick of the new world = current server tick
        world.tick = self.world.tick;
        let mut ids: Vec<u32> = self
            .clients
            .iter()
            .filter(|(_, c)| c.slot.is_some() || c.loading)
            .map(|(id, _)| *id)
            .collect();
        ids.sort_unstable();
        for id in &ids {
            if let Some(c) = self.clients.get_mut(id) {
                c.slot = None;
                c.acked = None;
                c.inputs.clear();
                c.last_input = PlayerInput::default();
                c.events.clear();
            }
        }
        self.rules = Rules::new(self.rules.cfg.clone(), &mut world, true);
        self.world = world;
        self.history.clear();
        self.map = entry;
        for id in ids {
            self.start_loading(id);
        }
        self.notice(Message::MapChanged {
            map: self.map.name.clone(),
        });
        let _ = now;
        Ok(())
    }

    /// Change the rules (mode, instagib, limits): new match on the same map.
    pub fn set_rules(&mut self, cfg: RulesConfig) {
        self.world.tuning = self.base_tuning.clone();
        self.rules = Rules::new(cfg, &mut self.world, false);
        self.notice(Message::ModeChanged {
            mode: self.rules.cfg.title(),
        });
    }

    /// Shut down the server: disconnect all clients with a reason.
    pub fn shutdown(&mut self, reason: &str, now: Instant) {
        let ids: Vec<u32> = self.clients.keys().copied().collect();
        for id in ids {
            self.endpoint.disconnect(id, reason, now);
        }
        let _ = self.endpoint.poll(now);
    }

    /// Disconnect the player in slot `slot`; `ban`: ban the address for 5 min.
    pub fn kick(&mut self, slot: usize, reason: &str, ban: bool, now: Instant) -> bool {
        let Some(id) = self.client_by_slot(slot) else {
            return false;
        };
        if ban && let Some(ip) = self.clients.get(&id).and_then(|c| c.ip) {
            self.bans.insert(ip, now + KICK_BAN);
        }
        self.endpoint.disconnect(id, reason, now);
        true
    }
}

/// Is the disconnection a normal event (for the log)?
pub fn is_graceful(reason: &DisconnectReason) -> bool {
    matches!(
        reason,
        DisconnectReason::Remote(_) | DisconnectReason::Local
    )
}
