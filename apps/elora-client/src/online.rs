//! Online client (M3.6/M3.7): receive snapshots, synchronise time, predict the own character
//! and own weapons (E-057), interpolate other characters.
//!
//! Pure logic without a socket: events from `elora-net` in, messages out. This way
//! the client can be tested together with the server in virtual time.

use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

use elora_map::Map;
use elora_protocol::codec::Reader;
use elora_protocol::snapshot::is_dummy;
use elora_protocol::{
    ClientMsg, GameView, MAP_CHUNK, MapChecksum, PROTOCOL_VERSION, ServerMsg, Skin, Snapshot,
    VoteInfo, VoteKind,
};
use elora_sim::{Event, PlayerInput, TICKS_PER_SECOND, Team, Tuning, World};

use crate::map_store::{MapStore, MemoryStore};
use crate::scene::{Scene, SceneChar};

/// Time sync: `offset` follows the target this fast (s per s); beyond this deviation it jumps.
const OFFSET_RISE: f64 = 0.002;
const OFFSET_FALL: f64 = 0.02;
const OFFSET_SNAP: f64 = 0.05;
/// The drawn lead follows the input lead this fast (s per s).
const LEAD_RATE: f64 = 0.01;
const TICK_SECS: f64 = 1.0 / TICKS_PER_SECOND as f64;
/// Target time by which an input arrives at the server before its tick (original: 10 ms).
const INPUT_MARGIN_MS: f64 = 10.0;
/// Snapshots for delta base and interpolation.
const SNAPSHOT_HISTORY: usize = 64;
/// Inputs per packet (redundancy against loss).
const INPUT_REDUNDANCY: u64 = 4;
/// Time window for estimating the server time (minimum = fastest packet).
const OFFSET_WINDOW: usize = 50;

/// A line in the chat history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatLine {
    /// `None` = server/notice.
    pub from: Option<String>,
    pub team: bool,
    /// Displayed text (for server messages the German version as fallback).
    pub text: String,
    /// Server message to translate (M8.1); `None` = chat or free text.
    pub message: Option<elora_protocol::Message>,
    pub at: Instant,
}

impl ChatLine {
    /// Notice without sender from a translatable message.
    pub fn notice(message: elora_protocol::Message, at: Instant) -> Self {
        Self {
            from: None,
            team: false,
            text: message.to_string(),
            message: Some(message),
            at,
        }
    }
}

/// Length of the chat history.
const CHAT_HISTORY: usize = 50;
/// Map parts requested at the same time during download.
const MAP_WINDOW: u32 = 4;

/// Running map download (M6.5).
#[derive(Debug)]
struct Download {
    name: String,
    checksum: MapChecksum,
    size: usize,
    data: Vec<u8>,
    /// Next part to request.
    next_request: u32,
    total: u32,
}

/// Client-side disconnect reasons, written as `<code> <detail>` into [`Status::Disconnected`];
/// the game shows them via the language key `reason.<code without #>` (E-352).
pub mod fail_code {
    pub const MAP_DAMAGED: &str = "#map-damaged";
    pub const MAP_INVALID: &str = "#map-invalid";
    pub const MAP_MISMATCH: &str = "#map-mismatch";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Connecting,
    Joining,
    /// Map is loading (cache or download); bytes received / total.
    Loading {
        map: String,
        received: usize,
        size: usize,
    },
    Playing,
    Disconnected(String),
}

/// Measurements for the debug panel (M3.9).
#[derive(Debug, Clone, Copy, Default)]
pub struct NetInfo {
    /// Predicted ticks before the last snapshot.
    pub prediction_ticks: u64,
    /// Lead of the inputs (ms).
    pub lead_ms: f64,
    /// Last reported remaining time of the inputs at the server (ms).
    pub input_time_left_ms: i32,
    /// Size of the last snapshot (bytes, delta).
    pub snapshot_bytes: usize,
    /// Number of discarded snapshots (missing base or checksum).
    pub snapshot_errors: u64,
    /// Deviation prediction ↔ server at the last snapshot (units).
    pub correction: f32,
}

#[derive(Debug)]
pub struct OnlineClient {
    name: String,
    skin: Skin,
    pub status: Status,
    pub slot: Option<usize>,
    pub map: Option<Map>,
    pub map_name: String,
    map_checksum: Option<MapChecksum>,
    download: Option<Download>,
    store: Box<dyn MapStore>,
    pub high_bandwidth: bool,
    template: Option<World>,
    snapshots: VecDeque<Snapshot>,
    inputs: BTreeMap<u64, PlayerInput>,
    last_input_tick: u64,
    epoch: Instant,
    /// Receive time − tick·duration, minimum over a window = time “tick 0 arrives”.
    offsets: VecDeque<f64>,
    /// Target from the window; `offset` follows it smoothly so the time does not jump (E-294).
    offset_target: Option<f64>,
    offset: Option<f64>,
    /// Lead of the prediction ahead of the estimated server time (s).
    lead: f64,
    /// Smoothed lead for drawing (inputs use `lead`).
    render_lead: f64,
    /// Time of the last `update` (smoothing).
    last_update: Option<Instant>,
    /// Predicted world at tick `pred_tick` and Elora one tick before.
    pred: Option<World>,
    pred_prev: Option<elora_sim::CharacterCore>,
    pred_tick: u64,
    pred_dirty: bool,
    /// Predicted position of Elora per tick (for measuring corrections).
    pred_history: BTreeMap<u64, elora_sim::Vec2>,
    /// Events for effects (server + own prediction).
    events: Vec<Event>,
    predicted_events_upto: u64,
    outgoing: Vec<(Vec<u8>, bool)>,
    pub info: NetInfo,
    /// Names of the slots.
    pub names: BTreeMap<usize, String>,
    /// Skins of the slots (E-095).
    pub skins: BTreeMap<usize, Skin>,
    /// Chat history incl. server notices, newest last.
    pub chat: VecDeque<ChatLine>,
    /// Running vote.
    pub vote: Option<VoteInfo>,
    /// Received emotes, fetched with [`OnlineClient::take_emotes`].
    emotes: Vec<(usize, u8)>,
}

impl OnlineClient {
    pub fn new(name: &str, skin: Skin, now: Instant) -> Self {
        Self {
            name: name.to_owned(),
            skin,
            status: Status::Connecting,
            slot: None,
            map: None,
            map_name: String::new(),
            map_checksum: None,
            download: None,
            store: Box::new(MemoryStore::default()),
            high_bandwidth: false,
            template: None,
            snapshots: VecDeque::new(),
            inputs: BTreeMap::new(),
            last_input_tick: 0,
            epoch: now,
            offsets: VecDeque::new(),
            offset_target: None,
            offset: None,
            lead: 0.1,
            render_lead: 0.1,
            last_update: None,
            pred: None,
            pred_prev: None,
            pred_tick: 0,
            pred_dirty: false,
            pred_history: BTreeMap::new(),
            events: Vec::new(),
            predicted_events_upto: 0,
            outgoing: Vec::new(),
            info: NetInfo::default(),
            names: BTreeMap::new(),
            skins: BTreeMap::new(),
            chat: VecDeque::new(),
            vote: None,
            emotes: Vec::new(),
        }
    }

    /// Take maps from this store and put downloads there (default: memory only).
    #[must_use]
    pub fn with_store(mut self, store: Box<dyn MapStore>) -> Self {
        self.store = store;
        self
    }

    /// Ends the connection with a client-side reason: `code` from [`fail_code`], `detail` is
    /// technical and stays English (E-352).
    fn fail(&mut self, code: &str, detail: &str) {
        let reason = format!("{code} {detail}").trim_end().to_owned();
        tracing::warn!("{reason}");
        self.download = None;
        self.status = Status::Disconnected(reason);
    }

    /// New map announced: discard game state, look up or download the map.
    fn on_map_info(&mut self, name: String, checksum: MapChecksum, size: u32) {
        self.slot = None;
        self.template = None;
        self.snapshots.clear();
        self.inputs.clear();
        self.pred = None;
        self.pred_prev = None;
        self.pred_history.clear();
        let size = size as usize;
        self.status = Status::Loading {
            map: name.clone(),
            received: 0,
            size,
        };
        if let Some(data) = self.store.find(&name, &checksum) {
            tracing::info!(map = %name, "map from the cache");
            self.finish_map(name, checksum, &data);
            return;
        }
        let total = u32::try_from(size.div_ceil(MAP_CHUNK)).unwrap_or(u32::MAX);
        tracing::info!(map = %name, size, "downloading map");
        self.download = Some(Download {
            name,
            checksum,
            size,
            data: Vec::with_capacity(size),
            next_request: 0,
            total,
        });
        for _ in 0..MAP_WINDOW {
            self.request_next_chunk();
        }
    }

    fn request_next_chunk(&mut self) {
        let Some(d) = &mut self.download else { return };
        if d.next_request < d.total {
            let msg = ClientMsg::MapRequest {
                chunk: d.next_request,
            };
            d.next_request += 1;
            self.send(&msg);
        }
    }

    fn on_map_chunk(&mut self, index: u32, data: &[u8]) {
        let Some(d) = &mut self.download else { return };
        let offset = index as usize * MAP_CHUNK;
        // parts arrive in order over the reliable channel
        if offset != d.data.len() {
            return;
        }
        let expected = (d.size - offset).min(MAP_CHUNK);
        if data.len() != expected {
            self.fail(fail_code::MAP_DAMAGED, "wrong chunk size");
            return;
        }
        d.data.extend_from_slice(data);
        let (received, size) = (d.data.len(), d.size);
        if let Status::Loading { received: r, .. } = &mut self.status {
            *r = received;
        }
        if received < size {
            self.request_next_chunk();
            return;
        }
        let d = self.download.take().expect("checked above");
        if elora_map::checksum(&d.data) != d.checksum {
            self.fail(fail_code::MAP_DAMAGED, "checksum mismatch");
            return;
        }
        self.store.store(&d.name, &d.checksum, &d.data);
        self.finish_map(d.name, d.checksum, &d.data);
    }

    /// Map is available: read it and report to the server.
    fn finish_map(&mut self, name: String, checksum: MapChecksum, data: &[u8]) {
        match elora_map::decode(data) {
            Ok(map) => {
                self.map = Some(map);
                self.map_name = name;
                self.map_checksum = Some(checksum);
                self.send(&ClientMsg::MapReady);
            }
            Err(e) => self.fail(fail_code::MAP_INVALID, &e.to_string()),
        }
    }

    fn send(&mut self, msg: &ClientMsg) {
        self.outgoing.push((msg.encode(), true));
    }

    pub fn send_chat(&mut self, team: bool, text: &str) {
        self.send(&ClientMsg::Chat {
            team,
            text: text.to_owned(),
        });
    }

    pub fn set_team(&mut self, team: Team) {
        self.send(&ClientMsg::SetTeam(team));
    }

    pub fn kill(&mut self) {
        self.send(&ClientMsg::Kill);
    }

    /// Send an emote (number `0..EMOTES`); it is shown when the server distributes it.
    pub fn emote(&mut self, emote: u8) {
        self.send(&ClientMsg::Emote(emote));
    }

    /// Emotes received since the last call (slot, number).
    pub fn take_emotes(&mut self) -> Vec<(usize, u8)> {
        std::mem::take(&mut self.emotes)
    }

    pub fn call_vote(&mut self, kind: VoteKind) {
        self.send(&ClientMsg::CallVote(kind));
    }

    pub fn vote(&mut self, yes: bool) {
        self.send(&ClientMsg::Vote(yes));
    }

    fn push_chat(&mut self, from: Option<String>, team: bool, text: String, at: Instant) {
        self.chat.push_back(ChatLine {
            from,
            team,
            text,
            message: None,
            at,
        });
        while self.chat.len() > CHAT_HISTORY {
            self.chat.pop_front();
        }
    }

    /// Game state from the newest snapshot (scoreboard, timer).
    pub fn game(&self) -> Option<GameView> {
        self.snapshots.back()?.game_view()
    }

    /// Teams of all slots according to the newest snapshot.
    pub fn teams(&self) -> BTreeMap<usize, Team> {
        let (Some(t), Some(s)) = (&self.template, self.snapshots.back()) else {
            return BTreeMap::new();
        };
        let mut w = t.clone();
        s.apply_to(&mut w, None);
        w.players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| Some((i, p.as_ref()?.team)))
            .collect()
    }

    /// Team of a slot according to the newest snapshot.
    pub fn team_of(&self, slot: usize) -> Team {
        let (Some(t), Some(s)) = (&self.template, self.snapshots.back()) else {
            return Team::None;
        };
        let mut w = t.clone();
        s.apply_to(&mut w, None);
        w.team(slot)
    }

    /// Current server tick (estimated, for timers).
    pub fn server_tick(&self, now: Instant) -> Option<u64> {
        #[allow(clippy::cast_sign_loss)] // ruled out by max(0)
        self.arrival_tick(now).map(|t| t.max(0.0) as u64)
    }

    fn secs(&self, t: Instant) -> f64 {
        t.saturating_duration_since(self.epoch).as_secs_f64()
    }

    pub fn tuning(&self) -> Option<&Tuning> {
        self.template.as_ref().map(|w| &w.tuning)
    }

    /// Connection established → join.
    pub fn on_connected(&mut self) {
        self.status = Status::Joining;
        let msg = ClientMsg::Join {
            version: PROTOCOL_VERSION,
            name: self.name.clone(),
            skin: self.skin,
        };
        self.outgoing.push((msg.encode(), true));
    }

    /// Change the own skin; sent to the server right away.
    pub fn set_skin(&mut self, skin: Skin) {
        if skin != self.skin {
            self.skin = skin;
            if self.slot.is_some() {
                self.send(&ClientMsg::SetSkin(skin));
            }
        }
    }

    pub fn on_disconnected(&mut self, reason: String) {
        self.status = Status::Disconnected(reason);
    }

    /// Messages to be sent (data, reliable?).
    pub fn take_outgoing(&mut self) -> Vec<(Vec<u8>, bool)> {
        std::mem::take(&mut self.outgoing)
    }

    /// Events since the last fetch.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Processes a message from the server, received at time `at`.
    pub fn on_message(&mut self, data: &[u8], at: Instant) {
        let Ok(msg) = ServerMsg::decode(data) else {
            tracing::warn!("invalid server message");
            return;
        };
        match msg {
            ServerMsg::MapInfo {
                name,
                checksum,
                size,
            } => self.on_map_info(name, checksum, size),
            ServerMsg::MapChunk { index, data } => self.on_map_chunk(index, &data),
            ServerMsg::Welcome {
                slot,
                tick,
                map_name,
                map_checksum,
                tuning,
                high_bandwidth,
            } => {
                let Some(map) = self
                    .map
                    .as_ref()
                    .filter(|_| self.map_checksum == Some(map_checksum))
                else {
                    self.fail(fail_code::MAP_MISMATCH, &map_name);
                    return;
                };
                // also after a map change: discard the state of the old map
                self.template = Some(map.world(tuning));
                self.slot = Some(slot as usize);
                self.high_bandwidth = high_bandwidth;
                self.last_input_tick = tick;
                self.snapshots.clear();
                self.inputs.clear();
                self.pred = None;
                self.pred_prev = None;
                self.pred_history.clear();
                self.names.clear();
                self.status = Status::Playing;
            }
            ServerMsg::Chat { from, team, text } => {
                let from = from.map(|f| {
                    self.names
                        .get(&(f as usize))
                        .cloned()
                        .unwrap_or_else(|| format!("Player {f}"))
                });
                self.push_chat(
                    Some(from.unwrap_or_else(|| "Server".into())),
                    team,
                    text,
                    at,
                );
            }
            ServerMsg::PlayerInfo { slot, name, skin } => {
                let slot = slot as usize;
                if let Some(n) = name {
                    self.names.insert(slot, n);
                    self.skins.insert(slot, skin);
                } else {
                    self.names.remove(&slot);
                    self.skins.remove(&slot);
                }
            }
            ServerMsg::Vote(v) => self.vote = v,
            ServerMsg::Notice(message) => {
                self.chat.push_back(ChatLine::notice(message, at));
                while self.chat.len() > CHAT_HISTORY {
                    self.chat.pop_front();
                }
            }
            ServerMsg::Emote { slot, emote } => self.emotes.push((slot as usize, emote)),
            ServerMsg::Snapshot {
                tick,
                base,
                checksum,
                delta,
                events,
            } => self.snapshot(tick, base, checksum, &delta, events, at),
            ServerMsg::InputTiming { time_left_ms, .. } => {
                self.info.input_time_left_ms = time_left_ms;
                let left = f64::from(time_left_ms);
                if left < INPUT_MARGIN_MS {
                    // too late: more lead right away
                    self.lead += (INPUT_MARGIN_MS - left) / 1000.0 * 0.5;
                } else if left > INPUT_MARGIN_MS + 15.0 {
                    // too early: slowly less lead
                    self.lead -= (left - INPUT_MARGIN_MS - 15.0) / 1000.0 * 0.02;
                }
                self.lead = self.lead.clamp(0.0, 1.0);
            }
            ServerMsg::Tuning(t) => {
                if let Some(w) = &mut self.template {
                    w.tuning = t;
                }
                self.pred_dirty = true;
            }
            ServerMsg::Kick { reason } => self.status = Status::Disconnected(reason),
        }
    }

    fn snapshot(
        &mut self,
        tick: u64,
        base: Option<u64>,
        checksum: u32,
        delta: &[u8],
        events: Vec<Event>,
        at: Instant,
    ) {
        if self.snapshots.back().is_some_and(|s| s.tick >= tick) {
            return; // outdated or duplicate
        }
        let base_snap = match base {
            Some(b) => {
                let Some(s) = self.snapshots.iter().find(|s| s.tick == b) else {
                    self.info.snapshot_errors += 1;
                    return;
                };
                Some(s)
            }
            None => None,
        };
        let mut r = Reader::new(delta);
        let Ok(snap) = Snapshot::decode_delta(tick, base_snap, &mut r) else {
            self.info.snapshot_errors += 1;
            return;
        };
        if snap.checksum() != checksum {
            self.info.snapshot_errors += 1;
            return;
        }
        self.info.snapshot_bytes = delta.len();
        let local = self.slot;
        self.events.extend(
            events
                .into_iter()
                .filter(|e| e.shooter().is_none() || e.shooter() != local),
        );

        // time sync
        let sample = self.secs(at) - tick as f64 * TICK_SECS;
        self.offsets.push_back(sample);
        while self.offsets.len() > OFFSET_WINDOW {
            self.offsets.pop_front();
        }
        self.offset_target = self.offsets.iter().copied().reduce(f64::min);
        if self.offset.is_none() {
            self.offset = self.offset_target;
        }

        // measure the deviation of the prediction (for the panel)
        if let Some(slot) = local
            && let Some(predicted) = self.pred_history.get(&tick)
            && let Some(server) = snapshot_core(&snap, self.template.as_ref(), slot)
        {
            self.info.correction = predicted.distance(server.pos);
        }
        self.pred_history = self.pred_history.split_off(&tick);

        self.snapshots.push_back(snap);
        while self.snapshots.len() > SNAPSHOT_HISTORY {
            self.snapshots.pop_front();
        }
        self.pred_dirty = true;
    }

    /// Estimated server tick as it is currently arriving at the client (with fraction).
    fn arrival_tick(&self, now: Instant) -> Option<f64> {
        Some((self.secs(now) - self.offset?) / TICK_SECS)
    }

    /// Tick up to which is predicted (with fraction).
    fn prediction_time(&self, now: Instant) -> Option<f64> {
        Some(self.arrival_tick(now)? + self.lead / TICK_SECS)
    }

    /// Time of the own character when drawing: like [`Self::prediction_time`], with
    /// smoothed lead.
    fn render_time(&self, now: Instant) -> Option<f64> {
        Some(self.arrival_tick(now)? + self.render_lead / TICK_SECS)
    }

    /// Adjust time sync and lead slowly instead of letting them jump (E-294).
    fn smooth_clock(&mut self, now: Instant) {
        let dt = self
            .last_update
            .map_or(0.0, |t| now.saturating_duration_since(t).as_secs_f64())
            .min(0.1);
        self.last_update = Some(now);
        if let (Some(target), Some(offset)) = (self.offset_target, self.offset) {
            let diff = target - offset;
            self.offset = Some(if diff.abs() > OFFSET_SNAP {
                target
            } else if diff < 0.0 {
                // packets arrive earlier than expected: follow quickly
                offset + diff.max(-OFFSET_FALL * dt)
            } else {
                offset + diff.min(OFFSET_RISE * dt)
            });
        }
        let diff = self.lead - self.render_lead;
        self.render_lead += diff.clamp(-LEAD_RATE * dt, LEAD_RATE * dt);
        if diff.abs() > 0.1 {
            self.render_lead = self.lead;
        }
    }

    /// Once per frame: create and send due inputs, update the prediction.
    /// `sample_input` returns the player's current input.
    pub fn update(&mut self, now: Instant, mut sample_input: impl FnMut() -> PlayerInput) {
        self.smooth_clock(now);
        if self.status != Status::Playing || self.snapshots.is_empty() {
            return;
        }
        let Some(target) = self.prediction_time(now) else {
            return;
        };
        #[allow(clippy::cast_sign_loss)] // ruled out by max(0)
        // one tick ahead: drawing happens between this and the previous tick (like the
        // original), otherwise the own character would stand still every sixth frame (E-294)
        let target_tick = target.floor().max(0.0) as u64 + 1;
        let latest = self.snapshots.back().map_or(0, |s| s.tick);
        // never compute more than 1 s ahead (protection against hangs)
        let target_tick = target_tick.clamp(latest, latest + u64::from(TICKS_PER_SECOND));
        if target_tick > self.last_input_tick {
            let input = sample_input();
            for t in self.last_input_tick + 1..=target_tick {
                self.inputs.insert(t, input);
            }
            self.last_input_tick = target_tick;
            let from = target_tick.saturating_sub(INPUT_REDUNDANCY - 1).max(1);
            let inputs: Vec<(u64, PlayerInput)> = (from..=target_tick)
                .filter_map(|t| self.inputs.get(&t).map(|i| (t, *i)))
                .collect();
            let msg = ClientMsg::Input {
                ack: Some(latest),
                inputs,
            };
            self.outgoing.push((msg.encode(), false));
            self.pred_dirty = true;
        }
        // forget old inputs
        let keep_from = self.snapshots.front().map_or(0, |s| s.tick);
        self.inputs = self.inputs.split_off(&keep_from);

        if self.pred_dirty {
            self.predict(self.last_input_tick);
            self.pred_dirty = false;
        }
        self.info.lead_ms = self.lead * 1000.0;
    }

    fn input_at(&self, tick: u64) -> PlayerInput {
        self.inputs
            .range(..=tick)
            .next_back()
            .map(|(_, i)| *i)
            .unwrap_or_default()
    }

    /// Simulate forward from the newest snapshot to `to` (E-057).
    fn predict(&mut self, to: u64) {
        let (Some(template), Some(slot), Some(snap)) =
            (&self.template, self.slot, self.snapshots.back())
        else {
            return;
        };
        let mut world = template.clone();
        snap.apply_to(&mut world, Some(slot));
        world.prediction = true;
        if let Some(p) = world.players.get_mut(slot).and_then(Option::as_mut) {
            // input the server used for the snapshot tick (click detection)
            p.input = self.input_at(snap.tick);
        }
        let mut inputs = vec![PlayerInput::default(); world.players.len()];
        let mut prev = world.core(slot).cloned();
        for t in snap.tick + 1..=to {
            prev = world.core(slot).cloned();
            if let Some(i) = inputs.get_mut(slot) {
                *i = self.input_at(t);
            }
            world.step(&inputs);
            if let Some(c) = world.core(slot) {
                self.pred_history.insert(t, c.pos);
            }
            if t > self.predicted_events_upto {
                self.events.extend(
                    world
                        .events
                        .iter()
                        .filter(|e| e.shooter() == Some(slot))
                        .cloned(),
                );
            }
        }
        self.predicted_events_upto = self.predicted_events_upto.max(to);
        self.info.prediction_ticks = to.saturating_sub(snap.tick);
        self.pred_tick = to;
        self.pred_prev = prev;
        self.pred = Some(world);
    }

    /// What to draw now.
    pub fn scene(&self, now: Instant) -> Option<Scene> {
        let template = self.template.as_ref()?;
        let slot = self.slot?;
        let latest = self.snapshots.back()?;
        let mut scene = Scene::default();

        // other characters: interpolate between the snapshots around the render time
        let interval = if self.high_bandwidth { 1.0 } else { 2.0 };
        // one tick of reserve for fluctuating latencies (E-294)
        let render = self.arrival_tick(now).unwrap_or(latest.tick as f64) - (interval + 2.0);
        let (a, b) = self.bracket(render);
        let alpha = if b.tick > a.tick {
            ((render - a.tick as f64) / (b.tick - a.tick) as f64).clamp(0.0, 1.0) as f32
        } else {
            1.0
        };
        let mut wa = template.clone();
        a.apply_to(&mut wa, None);
        let mut wb = template.clone();
        b.apply_to(&mut wb, None);
        for (i, p) in wb.players.iter().enumerate() {
            let Some(ch) = p.as_ref().and_then(|p| p.character.as_ref()) else {
                continue;
            };
            if i == slot {
                continue;
            }
            let prev = wa.core(i).cloned().unwrap_or_else(|| ch.core.clone());
            scene.chars.push(SceneChar {
                slot: i,
                ch: ch.clone(),
                prev,
                alpha,
                dummy: is_dummy(b, i),
                local: false,
                team: wb.team(i),
            });
        }
        #[allow(clippy::cast_possible_truncation)]
        scene.add_shots(&wb, alpha, interval as f32, |owner| owner != slot);

        // own character and own shots from the prediction
        let pred_alpha = self.render_time(now).map_or(1.0, |t| {
            (t - (self.pred_tick as f64 - 1.0)).clamp(0.0, 1.0) as f32
        });
        if let Some(pred) = &self.pred {
            if let Some(ch) = pred.character(slot) {
                let prev = self.pred_prev.clone().unwrap_or_else(|| ch.core.clone());
                scene.camera = prev.pos.lerp(ch.core.pos, pred_alpha);
                scene.chars.push(SceneChar {
                    slot,
                    ch: ch.clone(),
                    prev,
                    alpha: pred_alpha,
                    dummy: false,
                    local: true,
                    team: pred.team(slot),
                });
            }
            scene.add_shots(pred, pred_alpha, 1.0, |owner| owner == slot);
        }
        if scene.local().is_none() {
            // dead: camera at the last position
            scene.camera = self.pred.as_ref().and_then(|w| w.core(slot)).map_or_else(
                || {
                    latest_death_pos(&self.snapshots, template, slot)
                        .unwrap_or(template.spawn_points.first().copied().unwrap_or_default())
                },
                |c| c.pos,
            );
        }
        let mut wl = template.clone();
        latest.apply_to(&mut wl, None);
        scene.add_pickups(&wl);
        scene.add_flags(&wb);
        Some(scene)
    }

    /// Snapshots immediately before and after `render`.
    fn bracket(&self, render: f64) -> (&Snapshot, &Snapshot) {
        let latest = self.snapshots.back().expect("at least one snapshot");
        let mut a = self.snapshots.front().expect("at least one snapshot");
        for s in &self.snapshots {
            if s.tick as f64 <= render {
                a = s;
            } else {
                return (a, s);
            }
        }
        (latest, latest)
    }

    /// Server tick up to which the prediction reaches (for tests/panel).
    pub fn predicted_tick(&self) -> u64 {
        self.pred_tick
    }

    /// Predicted world (for tests).
    pub fn predicted_world(&self) -> Option<&World> {
        self.pred.as_ref()
    }

    /// Last received snapshot tick.
    pub fn latest_snapshot_tick(&self) -> Option<u64> {
        self.snapshots.back().map(|s| s.tick)
    }

    /// Time span since connecting (for the display).
    pub fn uptime(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.epoch)
    }
}

fn snapshot_core(
    snap: &Snapshot,
    template: Option<&World>,
    slot: usize,
) -> Option<elora_sim::CharacterCore> {
    let mut w = template?.clone();
    snap.apply_to(&mut w, None);
    w.core(slot).cloned()
}

fn latest_death_pos(
    snaps: &VecDeque<Snapshot>,
    template: &World,
    slot: usize,
) -> Option<elora_sim::Vec2> {
    snaps
        .iter()
        .rev()
        .find_map(|s| snapshot_core(s, Some(template), slot))
        .map(|c| c.pos)
}
