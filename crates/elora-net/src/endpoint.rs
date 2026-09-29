//! Verbindungen: Token-Handshake, Noise-Verschlüsselung (E-061, E-062), Timeouts.
//!
//! Ablauf:
//! 1. Client → `TokenRequest` (auf 512 Byte aufgefüllt: Antworten sind nie größer
//!    als Anfragen → kein Verstärkungsangriff)
//! 2. Server → `Token` (an die Absenderadresse gebunden, zustandslos geprüft)
//! 3. Client → `Hello` (Token + Noise-Nachricht 1)
//! 4. Server → `HelloReply` (Noise-Nachricht 2 mit Server-Schlüssel)
//! 5. Client prüft den Server-Schlüssel (TOFU, E-062) → `Confirm` (Noise-Nachricht 3)
//! 6. Danach nur noch `Data`: Paketnummer (= Nonce) + verschlüsselte [`Session`]-Daten.

use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::net::SocketAddr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use snow::{HandshakeState, StatelessTransportState};

use crate::session::{Delivery, Session, Stats};
use crate::socket::{MAX_DATAGRAM, Socket};

pub const NOISE_PARAMS: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";
/// Kennung des Protokolls in der Token-Anfrage.
pub(crate) const MAGIC: &[u8; 8] = b"ELORA\0\0\x01";
pub(crate) const TOKEN_REQUEST_SIZE: usize = 512;
const TIMEOUT: Duration = Duration::from_secs(10);
const KEEPALIVE: Duration = Duration::from_millis(250);
const HANDSHAKE_RESEND: Duration = Duration::from_millis(250);
const MAX_PENDING: usize = 128;
const TOKEN_BUCKET_SECS: u64 = 10;

pub(crate) const P_TOKEN_REQUEST: u8 = 1;
pub(crate) const P_TOKEN: u8 = 2;
const P_HELLO: u8 = 3;
const P_HELLO_REPLY: u8 = 4;
const P_CONFIRM: u8 = 5;
const P_DATA: u8 = 6;
const P_REJECT: u8 = 7;
/// Info-Abfrage ohne Verbindung (M7.6): `[8][Token 8][Nonce 4]` → `[9][Nonce 4][Info]`.
pub(crate) const P_INFO_REQUEST: u8 = 8;
pub(crate) const P_INFO: u8 = 9;
/// Höchstens so viele Info-Antworten je IP-Adresse und Sekunde.
const INFO_RATE: u32 = 20;

/// Statischer Schlüssel eines Servers.
#[derive(Clone, PartialEq, Eq)]
pub struct Keypair {
    pub private: Vec<u8>,
    pub public: Vec<u8>,
}

impl std::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keypair")
            .field("public", &hex(&self.public))
            .finish_non_exhaustive()
    }
}

impl Keypair {
    /// Neuer zufälliger Schlüssel.
    ///
    /// # Panics
    /// Wenn das Betriebssystem keine Zufallszahlen liefert.
    pub fn generate() -> Self {
        let kp = builder().generate_keypair().expect("Schlüsselerzeugung");
        Self {
            private: kp.private,
            public: kp.public,
        }
    }
}

/// Hex-Darstellung (z. B. für Fingerabdrücke von Server-Schlüsseln).
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

fn builder() -> snow::Builder<'static> {
    snow::Builder::new(NOISE_PARAMS.parse().expect("gültige Noise-Parameter"))
}

/// Grund einer Trennung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisconnectReason {
    Timeout,
    /// Von der Gegenseite beendet, mit Grund.
    Remote(String),
    /// Vom Server abgelehnt (vor dem Handshake, unverschlüsselt).
    Rejected(String),
    /// Server-Schlüssel stimmt nicht mit dem gespeicherten überein (E-062).
    KeyMismatch {
        expected: Vec<u8>,
        got: Vec<u8>,
    },
    /// Fehlerhafte Daten oder Krypto-Fehler.
    Protocol,
    /// Lokal beendet.
    Local,
}

/// Schutz gegen wiedereingespielte Pakete (Fenster von 64 Nummern).
#[derive(Debug, Default)]
struct ReplayWindow {
    highest: u64,
    bits: u64,
}

impl ReplayWindow {
    fn accept(&mut self, n: u64) -> bool {
        if n > self.highest {
            let shift = n - self.highest;
            self.bits = if shift >= 64 { 0 } else { self.bits << shift };
            self.bits |= 1;
            self.highest = n;
            true
        } else {
            let back = self.highest - n;
            if back >= 64 || self.bits & (1 << back) != 0 {
                return false;
            }
            self.bits |= 1 << back;
            true
        }
    }
}

/// Eine aufgebaute, verschlüsselte Verbindung.
struct Link {
    transport: StatelessTransportState,
    session: Session,
    send_nonce: u64,
    replay: ReplayWindow,
    last_recv: Instant,
}

impl Link {
    fn new(transport: StatelessTransportState, now: Instant) -> Self {
        Self {
            transport,
            session: Session::new(),
            send_nonce: 0,
            replay: ReplayWindow::default(),
            last_recv: now,
        }
    }

    /// Entschlüsselt ein `Data`-Paket und reicht es an die Session.
    fn receive(&mut self, packet: &[u8], now: Instant) -> Option<Vec<Delivery>> {
        let nonce = u64::from_le_bytes(packet.get(1..9)?.try_into().ok()?);
        let mut plain = vec![0; packet.len()];
        let n = self
            .transport
            .read_message(nonce, &packet[9..], &mut plain)
            .ok()?;
        if !self.replay.accept(nonce) {
            return Some(Vec::new());
        }
        self.last_recv = now;
        self.session.receive(nonce, &plain[..n], now).ok()
    }

    fn flush(&mut self, socket: &mut dyn Socket, addr: SocketAddr, now: Instant, force: bool) {
        if !force && !self.session.wants_send(now, KEEPALIVE) {
            return;
        }
        let mut nonce = self.send_nonce;
        let packets = self.session.build_packets(now, || {
            nonce += 1;
            nonce
        });
        self.send_nonce = nonce;
        let mut out = vec![0; MAX_DATAGRAM + 64];
        for (seq, payload) in packets {
            out[0] = P_DATA;
            out[1..9].copy_from_slice(&seq.to_le_bytes());
            if let Ok(n) = self.transport.write_message(seq, &payload, &mut out[9..]) {
                socket.send_to(&out[..9 + n], addr, now);
            }
        }
    }
}

/// Ereignisse des Servers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerEvent {
    Connected {
        id: u32,
        addr: SocketAddr,
    },
    Message {
        id: u32,
        reliable: bool,
        data: Vec<u8>,
    },
    Disconnected {
        id: u32,
        reason: DisconnectReason,
    },
}

struct PendingHandshake {
    reply: Vec<u8>,
    state: HandshakeState,
    created: Instant,
}

struct ServerConn {
    id: u32,
    link: Link,
    closing: bool,
}

/// Server-Seite: nimmt Verbindungen an und verwaltet sie.
pub struct ServerEndpoint<S: Socket> {
    socket: S,
    keypair: Keypair,
    max_clients: usize,
    token_key: std::hash::RandomState,
    pending: HashMap<SocketAddr, PendingHandshake>,
    conns: HashMap<SocketAddr, ServerConn>,
    by_id: HashMap<u32, SocketAddr>,
    next_id: u32,
    buf: Vec<u8>,
    /// Antwort auf Info-Abfragen (vom Spielserver gesetzt, für das Netz undurchsichtig).
    info: Vec<u8>,
    /// Rate-Grenze der Info-Antworten: IP → (Beginn der Sekunde, Anzahl).
    info_rate: HashMap<std::net::IpAddr, (Instant, u32)>,
}

impl<S: Socket> std::fmt::Debug for ServerEndpoint<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerEndpoint")
            .field("clients", &self.conns.len())
            .finish_non_exhaustive()
    }
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl<S: Socket> ServerEndpoint<S> {
    pub fn new(socket: S, keypair: Keypair, max_clients: usize) -> Self {
        Self {
            socket,
            keypair,
            max_clients,
            token_key: std::hash::RandomState::new(),
            pending: HashMap::new(),
            conns: HashMap::new(),
            by_id: HashMap::new(),
            next_id: 0,
            buf: vec![0; MAX_DATAGRAM],
            info: Vec::new(),
            info_rate: HashMap::new(),
        }
    }

    /// Setzt die Antwort auf Info-Abfragen (Server-Browser, M7.6). Zu große Daten
    /// werden abgeschnitten, damit die Antwort in ein Datagramm passt.
    pub fn set_info(&mut self, mut info: Vec<u8>) {
        info.truncate(MAX_DATAGRAM - 16);
        self.info = info;
    }

    /// Info-Anfrage beantworten, wenn das Token gültig ist und die Rate-Grenze passt.
    fn handle_info_request(&mut self, p: &[u8], addr: SocketAddr, now: Instant) {
        if p.len() != 13 || !self.token_valid(addr, &p[1..9]) {
            return;
        }
        if self.info_rate.len() > 4096 {
            self.info_rate
                .retain(|_, (t, _)| now - *t < Duration::from_secs(1));
        }
        let entry = self.info_rate.entry(addr.ip()).or_insert((now, 0));
        if now - entry.0 >= Duration::from_secs(1) {
            *entry = (now, 0);
        }
        if entry.1 >= INFO_RATE {
            return;
        }
        entry.1 += 1;
        let mut out = Vec::with_capacity(5 + self.info.len());
        out.push(P_INFO);
        out.extend_from_slice(&p[9..13]);
        out.extend_from_slice(&self.info);
        self.socket.send_to(&out, addr, now);
    }

    pub fn socket_mut(&mut self) -> &mut S {
        &mut self.socket
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr()
    }

    pub fn public_key(&self) -> &[u8] {
        &self.keypair.public
    }

    pub fn client_count(&self) -> usize {
        self.conns.len()
    }

    pub fn addr(&self, id: u32) -> Option<SocketAddr> {
        self.by_id.get(&id).copied()
    }

    pub fn stats(&self, id: u32) -> Option<Stats> {
        Some(self.conns.get(self.by_id.get(&id)?)?.link.session.stats)
    }

    fn token(&self, addr: SocketAddr, bucket: u64) -> [u8; 8] {
        let mut h = self.token_key.build_hasher();
        h.write(addr.to_string().as_bytes());
        h.write_u64(bucket);
        h.finish().to_le_bytes()
    }

    fn token_valid(&self, addr: SocketAddr, token: &[u8]) -> bool {
        let bucket = unix_secs() / TOKEN_BUCKET_SECS;
        [bucket, bucket.saturating_sub(1)]
            .iter()
            .any(|&b| self.token(addr, b) == token)
    }

    /// Sendet `data` an Client `id`.
    pub fn send(&mut self, id: u32, data: &[u8], reliable: bool) {
        let Some(conn) = self.by_id.get(&id).and_then(|a| self.conns.get_mut(a)) else {
            return;
        };
        let s = &mut conn.link.session;
        let _ = if reliable {
            s.send_reliable(data)
        } else {
            s.send_unreliable(data)
        };
    }

    /// Trennt Client `id` mit Grund (wird noch zugestellt).
    pub fn disconnect(&mut self, id: u32, reason: &str, now: Instant) {
        let Some(&addr) = self.by_id.get(&id) else {
            return;
        };
        if let Some(conn) = self.conns.get_mut(&addr) {
            conn.link.session.send_disconnect(reason);
            conn.link.flush(&mut self.socket, addr, now, true);
            conn.closing = true;
        }
    }

    /// Empfängt alles Anstehende und prüft Timeouts.
    pub fn poll(&mut self, now: Instant) -> Vec<ServerEvent> {
        let mut events = Vec::new();
        let mut buf = std::mem::take(&mut self.buf);
        while let Some((n, addr)) = self.socket.recv_from(&mut buf, now) {
            self.handle(&buf[..n], addr, now, &mut events);
        }
        self.buf = buf;
        self.pending
            .retain(|_, p| now - p.created < Duration::from_secs(5));
        let dead: Vec<(SocketAddr, u32, bool)> = self
            .conns
            .iter()
            .filter(|(_, c)| c.closing || now - c.link.last_recv > TIMEOUT)
            .map(|(a, c)| (*a, c.id, c.closing))
            .collect();
        for (addr, id, closing) in dead {
            self.conns.remove(&addr);
            self.by_id.remove(&id);
            let reason = if closing {
                DisconnectReason::Local
            } else {
                DisconnectReason::Timeout
            };
            events.push(ServerEvent::Disconnected { id, reason });
        }
        events
    }

    /// Sendet fällige Pakete aller Verbindungen.
    pub fn flush(&mut self, now: Instant) {
        for (addr, conn) in &mut self.conns {
            conn.link.flush(&mut self.socket, *addr, now, false);
        }
    }

    fn reject(&mut self, addr: SocketAddr, reason: &str, now: Instant) {
        let mut out = vec![P_REJECT];
        out.extend_from_slice(reason.as_bytes());
        self.socket.send_to(&out, addr, now);
    }

    fn handle(&mut self, p: &[u8], addr: SocketAddr, now: Instant, events: &mut Vec<ServerEvent>) {
        match p.first().copied() {
            Some(P_TOKEN_REQUEST) if p.len() == TOKEN_REQUEST_SIZE => {
                if &p[1..9] != MAGIC {
                    self.reject(addr, "Falsche Protokollversion", now);
                    return;
                }
                let mut out = vec![P_TOKEN];
                out.extend_from_slice(&self.token(addr, unix_secs() / TOKEN_BUCKET_SECS));
                self.socket.send_to(&out, addr, now);
            }
            Some(P_HELLO) if p.len() > 9 => self.handle_hello(p, addr, now),
            Some(P_CONFIRM) => self.handle_confirm(p, addr, now, events),
            Some(P_INFO_REQUEST) => self.handle_info_request(p, addr, now),
            Some(P_DATA) if p.len() > 9 => self.handle_data(p, addr, now, events),
            _ => {}
        }
    }

    fn handle_hello(&mut self, p: &[u8], addr: SocketAddr, now: Instant) {
        if !self.token_valid(addr, &p[1..9]) {
            return;
        }
        if let Some(pending) = self.pending.get(&addr) {
            // Antwort ging verloren: erneut senden
            let reply = pending.reply.clone();
            self.socket.send_to(&reply, addr, now);
            return;
        }
        if self.conns.contains_key(&addr) {
            return;
        }
        if self.conns.len() >= self.max_clients {
            self.reject(addr, "Server ist voll", now);
            return;
        }
        if self.pending.len() >= MAX_PENDING {
            return;
        }
        let Some(mut state) = builder()
            .local_private_key(&self.keypair.private)
            .and_then(snow::Builder::build_responder)
            .ok()
        else {
            return;
        };
        let mut scratch = [0u8; 256];
        if state.read_message(&p[9..], &mut scratch).is_err() {
            return;
        }
        let mut reply = vec![0u8; 256];
        let Ok(n) = state.write_message(&[], &mut reply[1..]) else {
            return;
        };
        reply[0] = P_HELLO_REPLY;
        reply.truncate(n + 1);
        self.socket.send_to(&reply, addr, now);
        self.pending.insert(
            addr,
            PendingHandshake {
                reply,
                state,
                created: now,
            },
        );
    }

    fn handle_confirm(
        &mut self,
        p: &[u8],
        addr: SocketAddr,
        now: Instant,
        events: &mut Vec<ServerEvent>,
    ) {
        let Some(mut pending) = self.pending.remove(&addr) else {
            return;
        };
        let mut scratch = [0u8; 256];
        if pending.state.read_message(&p[1..], &mut scratch).is_err() {
            return;
        }
        let Ok(transport) = pending.state.into_stateless_transport_mode() else {
            return;
        };
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let mut link = Link::new(transport, now);
        // sofort ein Paket, damit der Client den Aufbau bestätigt sieht
        link.flush(&mut self.socket, addr, now, true);
        self.conns.insert(
            addr,
            ServerConn {
                id,
                link,
                closing: false,
            },
        );
        self.by_id.insert(id, addr);
        events.push(ServerEvent::Connected { id, addr });
    }

    fn handle_data(
        &mut self,
        p: &[u8],
        addr: SocketAddr,
        now: Instant,
        events: &mut Vec<ServerEvent>,
    ) {
        let Some(conn) = self.conns.get_mut(&addr) else {
            return;
        };
        let id = conn.id;
        // Entschlüsselung fehlgeschlagen → ignorieren (könnte gefälscht sein)
        let Some(deliveries) = conn.link.receive(p, now) else {
            return;
        };
        for d in deliveries {
            match d {
                Delivery::Reliable(data) => events.push(ServerEvent::Message {
                    id,
                    reliable: true,
                    data,
                }),
                Delivery::Unreliable(data) => events.push(ServerEvent::Message {
                    id,
                    reliable: false,
                    data,
                }),
                Delivery::Disconnect(reason) => {
                    self.conns.remove(&addr);
                    self.by_id.remove(&id);
                    events.push(ServerEvent::Disconnected {
                        id,
                        reason: DisconnectReason::Remote(reason),
                    });
                    return;
                }
            }
        }
    }
}

/// Ereignisse des Clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientEvent {
    /// Handshake fertig. `server_key` für TOFU speichern (E-062).
    Connected {
        server_key: Vec<u8>,
    },
    Message {
        reliable: bool,
        data: Vec<u8>,
    },
    Disconnected(DisconnectReason),
}

enum ClientState {
    RequestingToken,
    /// `packet` wird bei Verlust unverändert wiederholt (passt zur gespeicherten Server-Antwort).
    Hello {
        packet: Vec<u8>,
        state: Box<HandshakeState>,
    },
    Confirming {
        confirm: Vec<u8>,
        server_key: Vec<u8>,
        link: Box<Link>,
    },
    Connected(Box<Link>),
    Closed,
}

/// Client-Seite: eine Verbindung zu einem Server.
pub struct ClientEndpoint<S: Socket> {
    socket: S,
    server: SocketAddr,
    /// Erwarteter Server-Schlüssel (aus `known_servers`), sonst TOFU.
    expected_key: Option<Vec<u8>>,
    state: ClientState,
    started: Instant,
    last_send: Option<Instant>,
    buf: Vec<u8>,
}

impl<S: Socket> std::fmt::Debug for ClientEndpoint<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientEndpoint")
            .field("server", &self.server)
            .finish_non_exhaustive()
    }
}

impl<S: Socket> ClientEndpoint<S> {
    pub fn connect(
        socket: S,
        server: SocketAddr,
        expected_key: Option<Vec<u8>>,
        now: Instant,
    ) -> Self {
        Self {
            socket,
            server,
            expected_key,
            state: ClientState::RequestingToken,
            started: now,
            last_send: None,
            buf: vec![0; MAX_DATAGRAM],
        }
    }

    pub fn socket_mut(&mut self) -> &mut S {
        &mut self.socket
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.state, ClientState::Connected(_))
    }

    pub fn stats(&self) -> Option<Stats> {
        match &self.state {
            ClientState::Connected(l) => Some(l.session.stats),
            _ => None,
        }
    }

    pub fn send(&mut self, data: &[u8], reliable: bool) {
        if let ClientState::Connected(link) = &mut self.state {
            let s = &mut link.session;
            let _ = if reliable {
                s.send_reliable(data)
            } else {
                s.send_unreliable(data)
            };
        }
    }

    /// Beendet die Verbindung (Grund wird noch gesendet).
    pub fn disconnect(&mut self, reason: &str, now: Instant) {
        if let ClientState::Connected(link) = &mut self.state {
            link.session.send_disconnect(reason);
            link.flush(&mut self.socket, self.server, now, true);
        }
        self.state = ClientState::Closed;
    }

    fn send_handshake(&mut self, now: Instant) {
        if self.last_send.is_some_and(|t| now - t < HANDSHAKE_RESEND) {
            return;
        }
        self.last_send = Some(now);
        match &mut self.state {
            ClientState::RequestingToken => {
                let mut p = vec![0u8; TOKEN_REQUEST_SIZE];
                p[0] = P_TOKEN_REQUEST;
                p[1..9].copy_from_slice(MAGIC);
                self.socket.send_to(&p, self.server, now);
            }
            ClientState::Hello { packet, .. } => {
                let p = packet.clone();
                self.socket.send_to(&p, self.server, now);
            }
            ClientState::Confirming { confirm, .. } => {
                let c = confirm.clone();
                self.socket.send_to(&c, self.server, now);
            }
            ClientState::Connected(_) | ClientState::Closed => {}
        }
    }

    /// Empfängt, treibt den Handshake voran, prüft Timeouts.
    pub fn poll(&mut self, now: Instant) -> Vec<ClientEvent> {
        let mut events = Vec::new();
        if matches!(self.state, ClientState::Closed) {
            return events;
        }
        let mut buf = std::mem::take(&mut self.buf);
        while let Some((n, from)) = self.socket.recv_from(&mut buf, now) {
            if from == self.server {
                self.handle(&buf[..n], now, &mut events);
            }
        }
        self.buf = buf;
        let timed_out = match &self.state {
            ClientState::Connected(l) => now - l.last_recv > TIMEOUT,
            ClientState::Closed => false,
            _ => now - self.started > TIMEOUT,
        };
        if timed_out {
            self.state = ClientState::Closed;
            events.push(ClientEvent::Disconnected(DisconnectReason::Timeout));
        }
        events
    }

    /// Sendet fällige Pakete (inkl. Handshake-Wiederholungen).
    pub fn flush(&mut self, now: Instant) {
        match &mut self.state {
            ClientState::Connected(link) => link.flush(&mut self.socket, self.server, now, false),
            _ => self.send_handshake(now),
        }
    }

    fn start_hello(&mut self, token: [u8; 8], now: Instant) {
        let kp = Keypair::generate();
        let Ok(mut state) = builder()
            .local_private_key(&kp.private)
            .and_then(snow::Builder::build_initiator)
        else {
            return;
        };
        let mut msg = vec![0u8; 128];
        let Ok(n) = state.write_message(&[], &mut msg) else {
            return;
        };
        let mut p = vec![P_HELLO];
        p.extend_from_slice(&token);
        p.extend_from_slice(&msg[..n]);
        self.socket.send_to(&p, self.server, now);
        self.last_send = Some(now);
        self.state = ClientState::Hello {
            packet: p,
            state: Box::new(state),
        };
    }

    fn handle(&mut self, p: &[u8], now: Instant, events: &mut Vec<ClientEvent>) {
        match (p.first().copied(), &mut self.state) {
            (Some(P_REJECT), ClientState::RequestingToken | ClientState::Hello { .. }) => {
                let reason = String::from_utf8_lossy(&p[1..]).chars().take(200).collect();
                self.state = ClientState::Closed;
                events.push(ClientEvent::Disconnected(DisconnectReason::Rejected(
                    reason,
                )));
            }
            (Some(P_TOKEN), ClientState::RequestingToken) if p.len() == 9 => {
                let mut token = [0u8; 8];
                token.copy_from_slice(&p[1..9]);
                self.start_hello(token, now);
            }
            (Some(P_HELLO_REPLY), ClientState::Hello { state, .. }) => {
                let mut scratch = [0u8; 256];
                if state.read_message(&p[1..], &mut scratch).is_err() {
                    return;
                }
                let Some(server_key) = state.get_remote_static().map(<[u8]>::to_vec) else {
                    return;
                };
                if let Some(expected) = &self.expected_key
                    && *expected != server_key
                {
                    let expected = expected.clone();
                    self.state = ClientState::Closed;
                    events.push(ClientEvent::Disconnected(DisconnectReason::KeyMismatch {
                        expected,
                        got: server_key,
                    }));
                    return;
                }
                let mut msg = vec![0u8; 256];
                let Ok(n) = state.write_message(&[], &mut msg[1..]) else {
                    return;
                };
                msg[0] = P_CONFIRM;
                msg.truncate(n + 1);
                let ClientState::Hello { state, .. } =
                    std::mem::replace(&mut self.state, ClientState::Closed)
                else {
                    return;
                };
                let Ok(transport) = state.into_stateless_transport_mode() else {
                    return;
                };
                self.socket.send_to(&msg, self.server, now);
                self.last_send = Some(now);
                self.state = ClientState::Confirming {
                    confirm: msg,
                    server_key,
                    link: Box::new(Link::new(transport, now)),
                };
            }
            (Some(P_DATA), ClientState::Confirming { .. }) => {
                let ClientState::Confirming {
                    server_key,
                    mut link,
                    ..
                } = std::mem::replace(&mut self.state, ClientState::Closed)
                else {
                    return;
                };
                let Some(deliveries) = link.receive(p, now) else {
                    self.state = ClientState::Closed;
                    events.push(ClientEvent::Disconnected(DisconnectReason::Protocol));
                    return;
                };
                events.push(ClientEvent::Connected { server_key });
                self.state = ClientState::Connected(link);
                self.deliver(deliveries, events);
            }
            (Some(P_DATA), ClientState::Connected(link)) => {
                if let Some(deliveries) = link.receive(p, now) {
                    self.deliver(deliveries, events);
                }
            }
            _ => {}
        }
    }

    fn deliver(&mut self, deliveries: Vec<Delivery>, events: &mut Vec<ClientEvent>) {
        for d in deliveries {
            match d {
                Delivery::Reliable(data) => events.push(ClientEvent::Message {
                    reliable: true,
                    data,
                }),
                Delivery::Unreliable(data) => events.push(ClientEvent::Message {
                    reliable: false,
                    data,
                }),
                Delivery::Disconnect(reason) => {
                    self.state = ClientState::Closed;
                    events.push(ClientEvent::Disconnected(DisconnectReason::Remote(reason)));
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_window_rejects_duplicates_and_old_packets() {
        let mut w = ReplayWindow::default();
        assert!(w.accept(1));
        assert!(w.accept(3));
        assert!(w.accept(2), "umgeordnet, aber neu");
        assert!(!w.accept(2), "Duplikat");
        assert!(w.accept(100));
        assert!(!w.accept(30), "älter als das Fenster");
        assert!(w.accept(99));
        assert!(!w.accept(100));
    }
}
