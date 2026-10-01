//! Socket-Abstraktion und Netzwerk-Simulator (M3.3).
//!
//! [`UdpSocket`] für echten Verkehr; [`MemNetwork`] für Tests im Speicher mit
//! virtueller Zeit. Beide können über [`Conditions`] Ping, Jitter, Verlust und
//! Umordnung simulieren.

use std::collections::{HashMap, VecDeque};
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Größter Datagramm-Puffer.
pub const MAX_DATAGRAM: usize = 1500;

pub trait Socket {
    fn send_to(&mut self, data: &[u8], addr: SocketAddr, now: Instant);
    fn recv_from(&mut self, buf: &mut [u8], now: Instant) -> Option<(usize, SocketAddr)>;
    fn local_addr(&self) -> SocketAddr;
}

/// Simulierte Leitungseigenschaften (pro Richtung, auf ausgehende Pakete angewendet).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Conditions {
    /// Einfache Verzögerung (halbe Round-Trip-Zeit bei beidseitiger Anwendung).
    pub latency: Duration,
    /// Zusätzliche zufällige Verzögerung 0..jitter.
    pub jitter: Duration,
    /// Verlustrate 0..1.
    pub loss: f32,
    /// Anteil doppelt gesendeter Pakete 0..1.
    pub duplicate: f32,
}

impl Conditions {
    pub fn is_ideal(&self) -> bool {
        self.latency.is_zero() && self.jitter.is_zero() && self.loss <= 0.0 && self.duplicate <= 0.0
    }
}

/// Verzögert und verwirft Pakete nach [`Conditions`].
#[derive(Debug)]
pub struct Conditioner {
    pub conditions: Conditions,
    queue: Vec<(Instant, Vec<u8>, SocketAddr)>,
    rng: u64,
}

impl Conditioner {
    pub fn new(conditions: Conditions, seed: u64) -> Self {
        Self {
            conditions,
            queue: Vec::new(),
            rng: seed | 1,
        }
    }

    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn push(&mut self, data: &[u8], addr: SocketAddr, now: Instant) {
        let c = self.conditions;
        if self.rand() < c.loss {
            return;
        }
        let copies = if self.rand() < c.duplicate { 2 } else { 1 };
        for _ in 0..copies {
            let delay = c.latency + c.jitter.mul_f32(self.rand());
            self.queue.push((now + delay, data.to_vec(), addr));
        }
    }

    /// Alle fälligen Pakete (in Reihenfolge ihres Fälligkeitszeitpunkts).
    pub fn pop_ready(&mut self, now: Instant) -> Vec<(Vec<u8>, SocketAddr)> {
        let (mut ready, rest): (Vec<_>, Vec<_>) =
            self.queue.drain(..).partition(|(t, ..)| *t <= now);
        self.queue = rest;
        ready.sort_by_key(|(t, ..)| *t);
        ready.into_iter().map(|(_, d, a)| (d, a)).collect()
    }
}

/// Echter, nicht blockierender UDP-Socket mit optionalem Simulator.
///
/// Entweder für eine Adressfamilie ([`UdpSocket::bind`]) oder für beide ([`UdpSocket::bind_dual`]:
/// je ein Socket für IPv4 und IPv6 auf demselben Port – wichtig für Anschlüsse ohne eigene
/// IPv4-Adresse wie DS-Lite).
#[derive(Debug)]
pub struct UdpSocket {
    v4: Option<std::net::UdpSocket>,
    v6: Option<std::net::UdpSocket>,
    pub conditioner: Conditioner,
}

fn v6_only(port: u16) -> io::Result<std::net::UdpSocket> {
    use socket2::{Domain, Protocol, Socket as RawSocket, Type};
    let s = RawSocket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    // nur IPv6: IPv4 läuft über den eigenen Socket auf demselben Port
    s.set_only_v6(true)?;
    s.bind(&SocketAddr::from(([0u16; 8], port)).into())?;
    let s: std::net::UdpSocket = s.into();
    s.set_nonblocking(true)?;
    Ok(s)
}

impl UdpSocket {
    fn with(v4: Option<std::net::UdpSocket>, v6: Option<std::net::UdpSocket>) -> Self {
        Self {
            v4,
            v6,
            conditioner: Conditioner::new(Conditions::default(), 0x9e37_79b9),
        }
    }

    /// Socket für die Adressfamilie von `addr`.
    ///
    /// # Errors
    /// Wenn der Socket nicht gebunden werden kann.
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = std::net::UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(if addr.is_ipv4() {
            Self::with(Some(socket), None)
        } else {
            Self::with(None, Some(socket))
        })
    }

    /// IPv4 und IPv6 auf `port` (0 = beliebig). Fehlt IPv6 auf dem Rechner, nur IPv4.
    ///
    /// # Errors
    /// Wenn weder IPv4 noch IPv6 gebunden werden kann.
    pub fn bind_dual(port: u16) -> io::Result<Self> {
        let v4 = std::net::UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], port)));
        // beliebiger Port: IPv6 auf demselben Port wie IPv4, damit beide gleich erreichbar sind
        let port6 = match &v4 {
            Ok(s) if port == 0 => s.local_addr().map_or(0, |a| a.port()),
            _ => port,
        };
        let v6 = v6_only(port6).or_else(|e| if port == 0 { v6_only(0) } else { Err(e) });
        match (v4, v6) {
            (Err(e), Err(_)) => Err(e),
            (v4, v6) => {
                let v4 = v4.ok();
                if let Some(s) = &v4 {
                    s.set_nonblocking(true)?;
                }
                if v6.is_err() {
                    tracing::info!("kein IPv6 verfügbar – nur IPv4");
                }
                Ok(Self::with(v4, v6.ok()))
            }
        }
    }

    /// Lauscht der Socket auf IPv6?
    pub fn has_ipv6(&self) -> bool {
        self.v6.is_some()
    }

    /// Lauscht der Socket auf IPv4?
    pub fn has_ipv4(&self) -> bool {
        self.v4.is_some()
    }

    /// Broadcast erlauben (LAN-Suche des Server-Browsers, nur IPv4).
    ///
    /// # Errors
    /// Wenn das Betriebssystem die Option ablehnt.
    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        match &self.v4 {
            Some(s) => s.set_broadcast(on),
            None => Ok(()),
        }
    }

    fn raw_send(&self, data: &[u8], addr: SocketAddr) {
        let socket = if addr.is_ipv4() { &self.v4 } else { &self.v6 };
        if let Some(s) = socket {
            let _ = s.send_to(data, addr);
        }
    }

    fn flush_delayed(&mut self, now: Instant) {
        for (data, addr) in self.conditioner.pop_ready(now) {
            self.raw_send(&data, addr);
        }
    }
}

fn recv(socket: &std::net::UdpSocket, buf: &mut [u8]) -> Option<(usize, SocketAddr)> {
    loop {
        match socket.recv_from(buf) {
            Ok(v) => return Some(v),
            // z. B. ICMP „Port nicht erreichbar“ unter Windows: überspringen
            Err(e) if e.kind() == io::ErrorKind::ConnectionReset => {}
            Err(_) => return None,
        }
    }
}

impl Socket for UdpSocket {
    fn send_to(&mut self, data: &[u8], addr: SocketAddr, now: Instant) {
        if self.conditioner.conditions.is_ideal() {
            self.raw_send(data, addr);
        } else {
            self.conditioner.push(data, addr, now);
            self.flush_delayed(now);
        }
    }

    fn recv_from(&mut self, buf: &mut [u8], now: Instant) -> Option<(usize, SocketAddr)> {
        self.flush_delayed(now);
        self.v4
            .as_ref()
            .and_then(|s| recv(s, buf))
            .or_else(|| self.v6.as_ref().and_then(|s| recv(s, buf)))
    }

    fn local_addr(&self) -> SocketAddr {
        self.v4
            .as_ref()
            .or(self.v6.as_ref())
            .and_then(|s| s.local_addr().ok())
            .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 0)))
    }
}

#[cfg(test)]
mod udp_tests {
    use super::*;

    #[test]
    fn dual_socket_talks_v4_and_v6() {
        let mut server = UdpSocket::bind_dual(0).unwrap();
        let port = server.local_addr().port();
        let now = Instant::now();
        let mut buf = [0u8; 64];
        let wait = |s: &mut UdpSocket, buf: &mut [u8]| {
            for _ in 0..200 {
                if let Some(v) = s.recv_from(buf, Instant::now()) {
                    return Some(v);
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            None
        };
        let mut c4 = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        c4.send_to(b"v4", SocketAddr::from(([127, 0, 0, 1], port)), now);
        let (n, from) = wait(&mut server, &mut buf).expect("IPv4 kommt an");
        assert_eq!(&buf[..n], b"v4");
        server.send_to(b"ok", from, now);
        assert!(wait(&mut c4, &mut buf).is_some(), "Antwort über IPv4");
        if server.has_ipv6()
            && let Ok(mut c6) = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 0)))
        {
            c6.send_to(
                b"v6",
                SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], port)),
                now,
            );
            let (n, from) = wait(&mut server, &mut buf).expect("IPv6 kommt an");
            assert_eq!(&buf[..n], b"v6");
            assert!(from.is_ipv6());
        }
    }
}

type Inbox = HashMap<SocketAddr, VecDeque<(Vec<u8>, SocketAddr)>>;

/// Netzwerk im Speicher für Tests.
#[derive(Debug, Clone, Default)]
pub struct MemNetwork {
    inboxes: Arc<Mutex<Inbox>>,
}

impl MemNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    /// Neuer Socket an `addr`.
    ///
    /// # Panics
    /// Wenn der interne Lock vergiftet ist (nur nach einem Panic in einem Test).
    pub fn socket(&self, addr: SocketAddr, conditions: Conditions, seed: u64) -> MemSocket {
        self.inboxes.lock().expect("Lock").entry(addr).or_default();
        MemSocket {
            addr,
            net: self.clone(),
            conditioner: Conditioner::new(conditions, seed),
        }
    }
}

#[derive(Debug)]
pub struct MemSocket {
    addr: SocketAddr,
    net: MemNetwork,
    pub conditioner: Conditioner,
}

impl MemSocket {
    fn deliver(&mut self, now: Instant) {
        let ready = self.conditioner.pop_ready(now);
        let mut inboxes = self.net.inboxes.lock().expect("Lock");
        for (data, to) in ready {
            if let Some(inbox) = inboxes.get_mut(&to) {
                inbox.push_back((data, self.addr));
            }
        }
    }
}

impl Socket for MemSocket {
    fn send_to(&mut self, data: &[u8], addr: SocketAddr, now: Instant) {
        self.conditioner.push(data, addr, now);
        self.deliver(now);
    }

    fn recv_from(&mut self, buf: &mut [u8], now: Instant) -> Option<(usize, SocketAddr)> {
        self.deliver(now);
        let (data, from) = self
            .net
            .inboxes
            .lock()
            .expect("Lock")
            .get_mut(&self.addr)?
            .pop_front()?;
        let n = data.len().min(buf.len());
        buf[..n].copy_from_slice(&data[..n]);
        Some((n, from))
    }

    fn local_addr(&self) -> SocketAddr {
        self.addr
    }
}
