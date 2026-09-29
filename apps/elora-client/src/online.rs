//! Online-Client (M3.6/M3.7): Snapshots empfangen, Zeit abgleichen, eigene Figur und
//! eigene Waffen vorhersagen (E-057), fremde Figuren interpolieren.
//!
//! Reine Logik ohne Socket: Ereignisse von `elora-net` rein, Nachrichten raus. So
//! lässt sich der Client zusammen mit dem Server in virtueller Zeit testen.

use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

use elora_map::Map;
use elora_protocol::codec::Reader;
use elora_protocol::snapshot::is_dummy;
use elora_protocol::{
    ClientMsg, GameView, PROTOCOL_VERSION, ServerMsg, Snapshot, VoteInfo, VoteKind,
};
use elora_sim::{Event, PlayerInput, TICKS_PER_SECOND, Team, Tuning, World};

use crate::scene::{Scene, SceneChar};

const TICK_SECS: f64 = 1.0 / TICKS_PER_SECOND as f64;
/// Angestrebte Zeit, die eine Eingabe vor ihrem Tick beim Server ankommt (Original: 10 ms).
const INPUT_MARGIN_MS: f64 = 10.0;
/// Snapshots für Delta-Basis und Interpolation.
const SNAPSHOT_HISTORY: usize = 64;
/// Eingaben pro Paket (Redundanz gegen Verlust).
const INPUT_REDUNDANCY: u64 = 4;
/// Zeitfenster für die Schätzung der Server-Zeit (Minimum = schnellstes Paket).
const OFFSET_WINDOW: usize = 50;

/// Eine Zeile im Chat-Verlauf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatLine {
    /// `None` = Server/Hinweis.
    pub from: Option<String>,
    pub team: bool,
    pub text: String,
    pub at: Instant,
}

/// Länge des Chat-Verlaufs.
const CHAT_HISTORY: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Connecting,
    Joining,
    Playing,
    Disconnected(String),
}

/// Messwerte für das Debug-Panel (M3.9).
#[derive(Debug, Clone, Copy, Default)]
pub struct NetInfo {
    /// Vorhergesagte Ticks vor dem letzten Snapshot.
    pub prediction_ticks: u64,
    /// Vorlauf der Eingaben (ms).
    pub lead_ms: f64,
    /// Zuletzt gemeldete Restzeit der Eingaben beim Server (ms).
    pub input_time_left_ms: i32,
    /// Größe des letzten Snapshots (Bytes, Delta).
    pub snapshot_bytes: usize,
    /// Anzahl verworfener Snapshots (fehlende Basis oder Prüfsumme).
    pub snapshot_errors: u64,
    /// Abweichung Vorhersage ↔ Server beim letzten Snapshot (Einheiten).
    pub correction: f32,
}

#[derive(Debug)]
pub struct OnlineClient {
    name: String,
    pub status: Status,
    pub slot: Option<usize>,
    pub map: Option<Map>,
    pub map_name: String,
    pub high_bandwidth: bool,
    template: Option<World>,
    snapshots: VecDeque<Snapshot>,
    inputs: BTreeMap<u64, PlayerInput>,
    last_input_tick: u64,
    epoch: Instant,
    /// Empfangszeit − Tick·Dauer, Minimum über ein Fenster = Zeit „Tick 0 kommt an“.
    offsets: VecDeque<f64>,
    offset: Option<f64>,
    /// Vorlauf der Vorhersage vor der geschätzten Server-Zeit (s).
    lead: f64,
    /// Vorhergesagte Welt beim Tick `pred_tick` und Elora einen Tick davor.
    pred: Option<World>,
    pred_prev: Option<elora_sim::CharacterCore>,
    pred_tick: u64,
    pred_dirty: bool,
    /// Vorhergesagte Position von Elora je Tick (für die Korrektur-Messung).
    pred_history: BTreeMap<u64, elora_sim::Vec2>,
    /// Ereignisse für Effekte (Server + eigene Vorhersage).
    events: Vec<Event>,
    predicted_events_upto: u64,
    outgoing: Vec<(Vec<u8>, bool)>,
    pub info: NetInfo,
    /// Namen der Slots.
    pub names: BTreeMap<usize, String>,
    /// Chat-Verlauf inkl. Server-Hinweise, neueste zuletzt.
    pub chat: VecDeque<ChatLine>,
    /// Laufende Abstimmung.
    pub vote: Option<VoteInfo>,
}

impl OnlineClient {
    pub fn new(name: &str, now: Instant) -> Self {
        Self {
            name: name.to_owned(),
            status: Status::Connecting,
            slot: None,
            map: None,
            map_name: String::new(),
            high_bandwidth: false,
            template: None,
            snapshots: VecDeque::new(),
            inputs: BTreeMap::new(),
            last_input_tick: 0,
            epoch: now,
            offsets: VecDeque::new(),
            offset: None,
            lead: 0.1,
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
            chat: VecDeque::new(),
            vote: None,
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
            at,
        });
        while self.chat.len() > CHAT_HISTORY {
            self.chat.pop_front();
        }
    }

    /// Spielzustand aus dem neuesten Snapshot (Scoreboard, Timer).
    pub fn game(&self) -> Option<GameView> {
        self.snapshots.back()?.game_view()
    }

    /// Team eines Slots laut neuestem Snapshot.
    pub fn team_of(&self, slot: usize) -> Team {
        let (Some(t), Some(s)) = (&self.template, self.snapshots.back()) else {
            return Team::None;
        };
        let mut w = t.clone();
        s.apply_to(&mut w, None);
        w.team(slot)
    }

    /// Aktueller Server-Tick (geschätzt, für Timer).
    pub fn server_tick(&self, now: Instant) -> Option<u64> {
        #[allow(clippy::cast_sign_loss)] // durch max(0) ausgeschlossen
        self.arrival_tick(now).map(|t| t.max(0.0) as u64)
    }

    fn secs(&self, t: Instant) -> f64 {
        t.saturating_duration_since(self.epoch).as_secs_f64()
    }

    pub fn tuning(&self) -> Option<&Tuning> {
        self.template.as_ref().map(|w| &w.tuning)
    }

    /// Verbindung steht → beitreten.
    pub fn on_connected(&mut self) {
        self.status = Status::Joining;
        let msg = ClientMsg::Join {
            version: PROTOCOL_VERSION,
            name: self.name.clone(),
        };
        self.outgoing.push((msg.encode(), true));
    }

    pub fn on_disconnected(&mut self, reason: String) {
        self.status = Status::Disconnected(reason);
    }

    /// Nachrichten, die gesendet werden sollen (Daten, zuverlässig?).
    pub fn take_outgoing(&mut self) -> Vec<(Vec<u8>, bool)> {
        std::mem::take(&mut self.outgoing)
    }

    /// Ereignisse seit dem letzten Abholen.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Verarbeitet eine Nachricht vom Server, empfangen zum Zeitpunkt `at`.
    pub fn on_message(&mut self, data: &[u8], at: Instant) {
        let Ok(msg) = ServerMsg::decode(data) else {
            tracing::warn!("ungültige Server-Nachricht");
            return;
        };
        match msg {
            ServerMsg::Welcome {
                slot,
                tick,
                map_name,
                map_source,
                tuning,
                high_bandwidth,
            } => {
                // auch nach einem Kartenwechsel: Zustand der alten Karte verwerfen
                match elora_map::parse_text_map(&map_source) {
                    Ok(map) => {
                        self.template = Some(map.world(tuning));
                        self.map = Some(map);
                        self.map_name = map_name;
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
                    Err(e) => {
                        self.status =
                            Status::Disconnected(format!("Karte vom Server ungültig: {e}"));
                    }
                }
            }
            ServerMsg::Chat { from, team, text } => {
                let from = from.map(|f| {
                    self.names
                        .get(&(f as usize))
                        .cloned()
                        .unwrap_or_else(|| format!("Spieler {f}"))
                });
                self.push_chat(
                    Some(from.unwrap_or_else(|| "Server".into())),
                    team,
                    text,
                    at,
                );
            }
            ServerMsg::PlayerInfo { slot, name } => match name {
                Some(n) => {
                    self.names.insert(slot as usize, n);
                }
                None => {
                    self.names.remove(&(slot as usize));
                }
            },
            ServerMsg::Vote(v) => self.vote = v,
            ServerMsg::Notice(text) => self.push_chat(None, false, text, at),
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
                    // zu spät: sofort mehr Vorlauf
                    self.lead += (INPUT_MARGIN_MS - left) / 1000.0 * 0.5;
                } else if left > INPUT_MARGIN_MS + 15.0 {
                    // zu früh: langsam weniger Vorlauf
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
            return; // veraltet oder doppelt
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

        // Zeitabgleich
        let sample = self.secs(at) - tick as f64 * TICK_SECS;
        self.offsets.push_back(sample);
        while self.offsets.len() > OFFSET_WINDOW {
            self.offsets.pop_front();
        }
        self.offset = self.offsets.iter().copied().reduce(f64::min);

        // Abweichung der Vorhersage messen (für das Panel)
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

    /// Geschätzter Server-Tick, wie er gerade beim Client ankommt (mit Bruchteil).
    fn arrival_tick(&self, now: Instant) -> Option<f64> {
        Some((self.secs(now) - self.offset?) / TICK_SECS)
    }

    /// Tick, bis zu dem vorhergesagt wird (mit Bruchteil).
    fn prediction_time(&self, now: Instant) -> Option<f64> {
        Some(self.arrival_tick(now)? + self.lead / TICK_SECS)
    }

    /// Einmal pro Frame: fällige Eingaben erzeugen und senden, Vorhersage aktualisieren.
    /// `sample_input` liefert die aktuelle Eingabe des Spielers.
    pub fn update(&mut self, now: Instant, mut sample_input: impl FnMut() -> PlayerInput) {
        if self.status != Status::Playing || self.snapshots.is_empty() {
            return;
        }
        let Some(target) = self.prediction_time(now) else {
            return;
        };
        #[allow(clippy::cast_sign_loss)] // durch max(0) ausgeschlossen
        let target_tick = target.floor().max(0.0) as u64;
        let latest = self.snapshots.back().map_or(0, |s| s.tick);
        // Nie mehr als 1 s vorausrechnen (Schutz bei Hängern)
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
        // alte Eingaben vergessen
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

    /// Vom neuesten Snapshot bis `to` vorwärtsrechnen (E-057).
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
            // Eingabe, die der Server für den Snapshot-Tick verwendet hat (Klick-Erkennung)
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

    /// Was jetzt zu zeichnen ist.
    pub fn scene(&self, now: Instant) -> Option<Scene> {
        let template = self.template.as_ref()?;
        let slot = self.slot?;
        let latest = self.snapshots.back()?;
        let mut scene = Scene::default();

        // Fremde Figuren: zwischen den Snapshots um die Renderzeit interpolieren
        let interval = if self.high_bandwidth { 1.0 } else { 2.0 };
        let render = self.arrival_tick(now).unwrap_or(latest.tick as f64) - (interval + 1.0);
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
            });
        }
        scene.add_shots(&wb, alpha, |owner| owner != slot);

        // Eigene Figur und eigene Schüsse aus der Vorhersage
        let pred_alpha = self.prediction_time(now).map_or(1.0, |t| {
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
                });
            }
            scene.add_shots(pred, pred_alpha, |owner| owner == slot);
        }
        if scene.local().is_none() {
            // tot: Kamera an die letzte Position
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
        Some(scene)
    }

    /// Snapshots unmittelbar vor und nach `render`.
    fn bracket(&self, render: f64) -> (&Snapshot, &Snapshot) {
        let latest = self.snapshots.back().expect("mindestens ein Snapshot");
        let mut a = self.snapshots.front().expect("mindestens ein Snapshot");
        for s in &self.snapshots {
            if s.tick as f64 <= render {
                a = s;
            } else {
                return (a, s);
            }
        }
        (latest, latest)
    }

    /// Server-Tick, bis zu dem die Vorhersage reicht (für Tests/Panel).
    pub fn predicted_tick(&self) -> u64 {
        self.pred_tick
    }

    /// Vorhergesagte Welt (für Tests).
    pub fn predicted_world(&self) -> Option<&World> {
        self.pred.as_ref()
    }

    /// Letzter empfangener Snapshot-Tick.
    pub fn latest_snapshot_tick(&self) -> Option<u64> {
        self.snapshots.back().map(|s| s.tick)
    }

    /// Zeitspanne seit der Verbindung (für die Anzeige).
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
