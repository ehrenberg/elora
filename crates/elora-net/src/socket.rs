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
#[derive(Debug)]
pub struct UdpSocket {
    socket: std::net::UdpSocket,
    pub conditioner: Conditioner,
}

impl UdpSocket {
    /// # Errors
    /// Wenn der Socket nicht gebunden werden kann.
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = std::net::UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            conditioner: Conditioner::new(Conditions::default(), 0x9e37_79b9),
        })
    }

    /// Broadcast erlauben (LAN-Suche des Server-Browsers).
    ///
    /// # Errors
    /// Wenn das Betriebssystem die Option ablehnt.
    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.socket.set_broadcast(on)
    }

    fn flush_delayed(&mut self, now: Instant) {
        for (data, addr) in self.conditioner.pop_ready(now) {
            let _ = self.socket.send_to(&data, addr);
        }
    }
}

impl Socket for UdpSocket {
    fn send_to(&mut self, data: &[u8], addr: SocketAddr, now: Instant) {
        if self.conditioner.conditions.is_ideal() {
            let _ = self.socket.send_to(data, addr);
        } else {
            self.conditioner.push(data, addr, now);
            self.flush_delayed(now);
        }
    }

    fn recv_from(&mut self, buf: &mut [u8], now: Instant) -> Option<(usize, SocketAddr)> {
        self.flush_delayed(now);
        loop {
            match self.socket.recv_from(buf) {
                Ok(v) => return Some(v),
                // z. B. ICMP „Port nicht erreichbar“ unter Windows: überspringen
                Err(e) if e.kind() == io::ErrorKind::ConnectionReset => {}
                Err(_) => return None,
            }
        }
    }

    fn local_addr(&self) -> SocketAddr {
        self.socket
            .local_addr()
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)))
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
