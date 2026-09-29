//! Server-Logik von Elora (M3.5): Welt simulieren, Eingaben anwenden, Snapshots senden.
//!
//! Unabhängig vom Socket, damit Tests den Server über ein Speicher-Netz betreiben.

pub mod config;

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::time::{Duration, Instant};

use elora_map::Map;
use elora_net::{DisconnectReason, Keypair, ServerEndpoint, ServerEvent, Socket};
use elora_protocol::{ClientMsg, PROTOCOL_VERSION, ServerMsg, Snapshot};
use elora_sim::{Event, PlayerInput, TICKS_PER_SECOND, Tuning, World};

pub use config::ServerConfig;

/// Dauer eines Ticks.
pub const TICK: Duration = Duration::from_micros(1_000_000 / TICKS_PER_SECOND as u64);
/// So viele Snapshots werden als Delta-Basis vorgehalten (≈ 2 s).
const HISTORY: usize = 100;
/// Eingaben weiter als so viele Ticks in der Zukunft werden verworfen.
const MAX_INPUT_AHEAD: u64 = 2 * TICKS_PER_SECOND as u64;

#[derive(Debug)]
struct Client {
    slot: Option<usize>,
    name: String,
    inputs: BTreeMap<u64, PlayerInput>,
    last_input: PlayerInput,
    acked: Option<u64>,
    events: Vec<Event>,
}

/// Der Spielserver.
pub struct GameServer<S: Socket> {
    pub endpoint: ServerEndpoint<S>,
    pub world: World,
    map_name: String,
    map_source: String,
    high_bandwidth: bool,
    clients: HashMap<u32, Client>,
    history: VecDeque<Snapshot>,
    start: Instant,
}

impl<S: Socket> std::fmt::Debug for GameServer<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameServer")
            .field("tick", &self.world.tick)
            .field("clients", &self.clients.len())
            .finish_non_exhaustive()
    }
}

impl<S: Socket> GameServer<S> {
    /// # Errors
    /// Bei ungültiger Karte.
    pub fn new(
        socket: S,
        key: Keypair,
        config: &ServerConfig,
        map_name: &str,
        map_source: String,
        tuning: Tuning,
        now: Instant,
    ) -> anyhow::Result<Self> {
        let map: Map = elora_map::parse_text_map(&map_source)?;
        let world = map.world(tuning);
        Ok(Self {
            endpoint: ServerEndpoint::new(socket, key, config.max_clients),
            world,
            map_name: map_name.to_owned(),
            map_source,
            high_bandwidth: config.high_bandwidth,
            clients: HashMap::new(),
            history: VecDeque::new(),
            start: now,
        })
    }

    /// Startzeit von Tick `tick`.
    pub fn tick_start(&self, tick: u64) -> Instant {
        self.start + TICK * u32::try_from(tick).unwrap_or(u32::MAX)
    }

    pub fn player_count(&self) -> usize {
        self.clients.values().filter(|c| c.slot.is_some()).count()
    }

    /// Netzwerk abarbeiten und fällige Ticks simulieren. Oft aufrufen (≥ 500 Hz).
    pub fn update(&mut self, now: Instant) {
        for event in self.endpoint.poll(now) {
            self.handle(event, now);
        }
        while now >= self.tick_start(self.world.tick + 1) {
            self.tick(now);
        }
        self.endpoint.flush(now);
    }

    /// Nächster Zeitpunkt, zu dem ein Tick fällig ist.
    pub fn next_tick_at(&self) -> Instant {
        self.tick_start(self.world.tick + 1)
    }

    fn handle(&mut self, event: ServerEvent, now: Instant) {
        match event {
            ServerEvent::Connected { id, addr } => {
                tracing::info!(%addr, id, "Verbindung aufgebaut");
                self.clients.insert(
                    id,
                    Client {
                        slot: None,
                        name: String::new(),
                        inputs: BTreeMap::new(),
                        last_input: PlayerInput::default(),
                        acked: None,
                        events: Vec::new(),
                    },
                );
            }
            ServerEvent::Disconnected { id, reason } => {
                if let Some(c) = self.clients.remove(&id) {
                    tracing::info!(id, name = %c.name, ?reason, "getrennt");
                    if let Some(slot) = c.slot {
                        self.world.remove(slot);
                    }
                }
            }
            ServerEvent::Message { id, data, .. } => match ClientMsg::decode(&data) {
                Ok(msg) => self.message(id, msg, now),
                Err(e) => {
                    tracing::warn!(id, %e, "ungültige Nachricht");
                    self.endpoint.disconnect(id, "Ungültige Nachricht", now);
                }
            },
        }
    }

    fn message(&mut self, id: u32, msg: ClientMsg, now: Instant) {
        let tick = self.world.tick;
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        match msg {
            ClientMsg::Join { version, name } => {
                if version != PROTOCOL_VERSION {
                    self.endpoint.disconnect(id, "Falsche Spielversion", now);
                    return;
                }
                if client.slot.is_some() {
                    return;
                }
                let slot = self.world.join();
                client.slot = Some(slot);
                client.name = name.chars().filter(|c| !c.is_control()).take(16).collect();
                tracing::info!(id, slot, name = %client.name, "beigetreten");
                let welcome = ServerMsg::Welcome {
                    slot: u32::try_from(slot).unwrap_or(0),
                    tick,
                    map_name: self.map_name.clone(),
                    map_source: self.map_source.clone(),
                    tuning: self.world.tuning.clone(),
                    high_bandwidth: self.high_bandwidth,
                };
                self.endpoint.send(id, &welcome.encode(), true);
            }
            ClientMsg::Input { ack, inputs } => {
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
                // Rückmeldung wie `INPUTTIMING`: Zeit bis zur Verarbeitung dieses Ticks
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
            ClientMsg::Leave => self.endpoint.disconnect(id, "Verlassen", now),
        }
    }

    fn tick(&mut self, now: Instant) {
        let next = self.world.tick + 1;
        let mut inputs = vec![PlayerInput::default(); self.world.players.len()];
        for c in self.clients.values_mut() {
            let Some(slot) = c.slot else { continue };
            // Eingabe für genau diesen Tick, sonst die letzte bekannte
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
        for c in self.clients.values_mut() {
            c.events.extend(self.world.events.iter().cloned());
        }

        let snap_now = self.high_bandwidth || self.world.tick.is_multiple_of(2);
        if snap_now {
            let snap = Snapshot::from_world(&self.world);
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
        let _ = now;
    }

    /// Server herunterfahren: alle Clients mit Grund trennen.
    pub fn shutdown(&mut self, reason: &str, now: Instant) {
        let ids: Vec<u32> = self.clients.keys().copied().collect();
        for id in ids {
            self.endpoint.disconnect(id, reason, now);
        }
        let _ = self.endpoint.poll(now);
    }
}

/// Ist die Trennung ein normaler Vorgang (für das Log)?
pub fn is_graceful(reason: &DisconnectReason) -> bool {
    matches!(
        reason,
        DisconnectReason::Remote(_) | DisconnectReason::Local
    )
}
