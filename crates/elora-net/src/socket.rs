//! Socket abstraction and network simulator (M3.3).
//!
//! [`UdpSocket`] for real traffic; [`MemNetwork`] for in-memory tests with virtual
//! time. Both can simulate ping, jitter, loss and reordering via [`Conditions`].

use std::collections::{HashMap, VecDeque};
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Largest datagram buffer.
pub const MAX_DATAGRAM: usize = 1500;

pub trait Socket {
    fn send_to(&mut self, data: &[u8], addr: SocketAddr, now: Instant);
    fn recv_from(&mut self, buf: &mut [u8], now: Instant) -> Option<(usize, SocketAddr)>;
    fn local_addr(&self) -> SocketAddr;
}

/// Simulated link properties (per direction, applied to outgoing packets).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Conditions {
    /// One-way delay (half the round-trip time when applied on both sides).
    pub latency: Duration,
    /// Additional random delay 0..jitter.
    pub jitter: Duration,
    /// Loss rate 0..1.
    pub loss: f32,
    /// Share of duplicated packets 0..1.
    pub duplicate: f32,
}

impl Conditions {
    pub fn is_ideal(&self) -> bool {
        self.latency.is_zero() && self.jitter.is_zero() && self.loss <= 0.0 && self.duplicate <= 0.0
    }
}

/// Delays and drops packets according to [`Conditions`].
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

    /// All due packets (in order of their due time).
    pub fn pop_ready(&mut self, now: Instant) -> Vec<(Vec<u8>, SocketAddr)> {
        let (mut ready, rest): (Vec<_>, Vec<_>) =
            self.queue.drain(..).partition(|(t, ..)| *t <= now);
        self.queue = rest;
        ready.sort_by_key(|(t, ..)| *t);
        ready.into_iter().map(|(_, d, a)| (d, a)).collect()
    }
}

/// Real, non-blocking UDP socket with optional simulator.
///
/// Either for one address family ([`UdpSocket::bind`]) or for both ([`UdpSocket::bind_dual`]:
/// one socket each for IPv4 and IPv6 on the same port – important for connections without
/// their own IPv4 address such as DS-Lite).
#[derive(Debug)]
pub struct UdpSocket {
    v4: Option<std::net::UdpSocket>,
    v6: Option<std::net::UdpSocket>,
    pub conditioner: Conditioner,
}

fn v6_only(port: u16) -> io::Result<std::net::UdpSocket> {
    use socket2::{Domain, Protocol, Socket as RawSocket, Type};
    let s = RawSocket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    // IPv6 only: IPv4 runs over its own socket on the same port
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

    /// Socket for the address family of `addr`.
    ///
    /// # Errors
    /// If the socket cannot be bound.
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = std::net::UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(if addr.is_ipv4() {
            Self::with(Some(socket), None)
        } else {
            Self::with(None, Some(socket))
        })
    }

    /// IPv4 and IPv6 on `port` (0 = any). If the machine lacks IPv6, IPv4 only.
    ///
    /// # Errors
    /// If neither IPv4 nor IPv6 can be bound.
    pub fn bind_dual(port: u16) -> io::Result<Self> {
        let v4 = std::net::UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], port)));
        // any port: IPv6 on the same port as IPv4 so that both are equally reachable
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
                    tracing::info!("no IPv6 available – IPv4 only");
                }
                Ok(Self::with(v4, v6.ok()))
            }
        }
    }

    /// Does the socket listen on IPv6?
    pub fn has_ipv6(&self) -> bool {
        self.v6.is_some()
    }

    /// Does the socket listen on IPv4?
    pub fn has_ipv4(&self) -> bool {
        self.v4.is_some()
    }

    /// Allow broadcast (LAN search of the server browser, IPv4 only).
    ///
    /// # Errors
    /// If the operating system rejects the option.
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
            // e.g. ICMP "port unreachable" on Windows: skip
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
        let (n, from) = wait(&mut server, &mut buf).expect("IPv4 arrives");
        assert_eq!(&buf[..n], b"v4");
        server.send_to(b"ok", from, now);
        assert!(wait(&mut c4, &mut buf).is_some(), "reply over IPv4");
        if server.has_ipv6()
            && let Ok(mut c6) = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 0)))
        {
            c6.send_to(
                b"v6",
                SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], port)),
                now,
            );
            let (n, from) = wait(&mut server, &mut buf).expect("IPv6 arrives");
            assert_eq!(&buf[..n], b"v6");
            assert!(from.is_ipv6());
        }
    }
}

type Inbox = HashMap<SocketAddr, VecDeque<(Vec<u8>, SocketAddr)>>;

/// In-memory network for tests.
#[derive(Debug, Clone, Default)]
pub struct MemNetwork {
    inboxes: Arc<Mutex<Inbox>>,
}

impl MemNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    /// New socket at `addr`.
    ///
    /// # Panics
    /// If the internal lock is poisoned (only after a panic in a test).
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
