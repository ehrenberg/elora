//! Connectionless server info (M7.6) – basis of the server browser.
//!
//! Flow per server: request a token (request padded to 512 bytes, reply smaller – no
//! amplification), then an info request with token and random nonce; the server only
//! answers the address the token belongs to, and at most [`INFO_RATE`] times per second.
//! The ping is the time between info request and reply.
//!
//! [`INFO_RATE`]: crate::endpoint
//!
//! The payload (name, map, players …) is opaque to this layer; it is encoded in
//! `elora-protocol`.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::endpoint::{
    MAGIC, P_INFO, P_INFO_REQUEST, P_TOKEN, P_TOKEN_REQUEST, TOKEN_REQUEST_SIZE,
};
use crate::socket::{MAX_DATAGRAM, Socket};

/// After this time without a reply a server counts as unreachable.
pub const INFO_TIMEOUT: Duration = Duration::from_secs(2);
/// The token request is repeated at this interval in case a packet gets lost.
const RESEND: Duration = Duration::from_millis(500);

/// Reply of a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoReply {
    pub addr: SocketAddr,
    pub ping: Duration,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
enum Stage {
    Token { sent: Instant },
    Info { nonce: [u8; 4], sent: Instant },
}

#[derive(Debug, Clone, Copy)]
struct Query {
    stage: Stage,
    started: Instant,
}

/// Asks servers for their info (also via broadcast in the LAN).
pub struct InfoProbe<S: Socket> {
    socket: S,
    queries: HashMap<SocketAddr, Query>,
    /// Running broadcast search: until when replies from new addresses are accepted.
    discovery_until: Option<Instant>,
    rng: u64,
    buf: Vec<u8>,
}

impl<S: Socket> std::fmt::Debug for InfoProbe<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfoProbe")
            .field("queries", &self.queries.len())
            .finish_non_exhaustive()
    }
}

fn token_request() -> Vec<u8> {
    let mut p = vec![0u8; TOKEN_REQUEST_SIZE];
    p[0] = P_TOKEN_REQUEST;
    p[1..9].copy_from_slice(MAGIC);
    p
}

impl<S: Socket> InfoProbe<S> {
    pub fn new(socket: S, seed: u64) -> Self {
        Self {
            socket,
            queries: HashMap::new(),
            discovery_until: None,
            rng: seed | 1,
            buf: vec![0; MAX_DATAGRAM],
        }
    }

    pub fn socket_mut(&mut self) -> &mut S {
        &mut self.socket
    }

    fn nonce(&mut self) -> [u8; 4] {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        #[allow(clippy::cast_possible_truncation)]
        let v = self.rng as u32;
        v.to_le_bytes()
    }

    /// Query server `addr` (replaces a running query).
    pub fn query(&mut self, addr: SocketAddr, now: Instant) {
        self.queries.insert(
            addr,
            Query {
                stage: Stage::Token { sent: now },
                started: now,
            },
        );
        self.socket.send_to(&token_request(), addr, now);
    }

    /// LAN search: token request to `targets` (e.g. broadcast addresses); every server
    /// that answers within [`INFO_TIMEOUT`] is queried.
    pub fn discover(&mut self, targets: &[SocketAddr], now: Instant) {
        self.discovery_until = Some(now + INFO_TIMEOUT);
        let p = token_request();
        for t in targets {
            self.socket.send_to(&p, *t, now);
        }
    }

    /// Are queries still running?
    pub fn busy(&self) -> bool {
        !self.queries.is_empty() || self.discovery_until.is_some()
    }

    /// Receives replies, repeats requests, drops expired ones. Returns new infos and the
    /// addresses that did not answer.
    pub fn poll(&mut self, now: Instant) -> (Vec<InfoReply>, Vec<SocketAddr>) {
        let mut replies = Vec::new();
        let mut buf = std::mem::take(&mut self.buf);
        while let Some((n, from)) = self.socket.recv_from(&mut buf, now) {
            if let Some(r) = self.handle(&buf[..n], from, now) {
                replies.push(r);
            }
        }
        self.buf = buf;
        if self.discovery_until.is_some_and(|t| now >= t) {
            self.discovery_until = None;
        }
        let mut timed_out = Vec::new();
        let mut resend = Vec::new();
        self.queries.retain(|addr, q| {
            if now - q.started >= INFO_TIMEOUT {
                timed_out.push(*addr);
                return false;
            }
            if let Stage::Token { sent } = q.stage
                && now - sent >= RESEND
            {
                q.stage = Stage::Token { sent: now };
                resend.push(*addr);
            }
            true
        });
        for addr in resend {
            self.socket.send_to(&token_request(), addr, now);
        }
        (replies, timed_out)
    }

    fn handle(&mut self, p: &[u8], from: SocketAddr, now: Instant) -> Option<InfoReply> {
        match p.first().copied()? {
            P_TOKEN if p.len() == 9 => {
                let known = self.queries.contains_key(&from);
                let discovering = self.discovery_until.is_some_and(|t| now < t);
                if !known && !discovering {
                    return None;
                }
                let nonce = self.nonce();
                let started = self.queries.get(&from).map_or(now, |q| q.started);
                self.queries.insert(
                    from,
                    Query {
                        stage: Stage::Info { nonce, sent: now },
                        started,
                    },
                );
                let mut out = Vec::with_capacity(13);
                out.push(P_INFO_REQUEST);
                out.extend_from_slice(&p[1..9]);
                out.extend_from_slice(&nonce);
                self.socket.send_to(&out, from, now);
                None
            }
            P_INFO if p.len() >= 5 => {
                let q = self.queries.get(&from)?;
                let Stage::Info { nonce, sent } = q.stage else {
                    return None;
                };
                if p[1..5] != nonce {
                    return None;
                }
                self.queries.remove(&from);
                Some(InfoReply {
                    addr: from,
                    ping: now - sent,
                    data: p[5..].to_vec(),
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::{Keypair, ServerEndpoint};
    use crate::socket::{Conditions, MemNetwork};

    fn addr(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    fn lag(ms: u64) -> Conditions {
        Conditions {
            latency: Duration::from_millis(ms),
            ..Conditions::default()
        }
    }

    #[test]
    fn query_returns_info_and_ping() {
        let net = MemNetwork::new();
        let mut server =
            ServerEndpoint::new(net.socket(addr(8303), lag(30), 1), Keypair::generate(), 8);
        server.set_info(b"Eloras Wiese".to_vec());
        let mut probe = InfoProbe::new(net.socket(addr(9000), lag(30), 2), 7);
        let mut now = Instant::now();
        probe.query(addr(8303), now);
        let mut got = None;
        for _ in 0..200 {
            now += Duration::from_millis(2);
            server.poll(now);
            let (replies, lost) = probe.poll(now);
            assert!(lost.is_empty());
            if let Some(r) = replies.into_iter().next() {
                got = Some(r);
                break;
            }
        }
        let r = got.expect("reply");
        assert_eq!(r.addr, addr(8303));
        assert_eq!(r.data, b"Eloras Wiese");
        assert!(
            r.ping >= Duration::from_millis(58) && r.ping <= Duration::from_millis(70),
            "{:?}",
            r.ping
        );
        assert!(!probe.busy());
    }

    #[test]
    fn unreachable_server_times_out() {
        let net = MemNetwork::new();
        let mut probe = InfoProbe::new(net.socket(addr(9001), Conditions::default(), 3), 7);
        let now = Instant::now();
        probe.query(addr(8399), now);
        let (_, lost) = probe.poll(now + Duration::from_millis(500));
        assert!(lost.is_empty());
        let (_, lost) = probe.poll(now + INFO_TIMEOUT);
        assert_eq!(lost, vec![addr(8399)]);
    }

    #[test]
    fn discovery_finds_servers_and_invalid_tokens_are_ignored() {
        let net = MemNetwork::new();
        let mut a = ServerEndpoint::new(
            net.socket(addr(8303), Conditions::default(), 1),
            Keypair::generate(),
            8,
        );
        let mut b = ServerEndpoint::new(
            net.socket(addr(8304), Conditions::default(), 2),
            Keypair::generate(),
            8,
        );
        a.set_info(b"A".to_vec());
        b.set_info(b"B".to_vec());
        let mut probe = InfoProbe::new(net.socket(addr(9002), Conditions::default(), 4), 9);
        let mut now = Instant::now();
        // in the in-memory network the list of ports replaces the broadcast
        probe.discover(&[addr(8303), addr(8304), addr(8305)], now);
        let mut found = Vec::new();
        for _ in 0..20 {
            now += Duration::from_millis(5);
            a.poll(now);
            b.poll(now);
            found.extend(probe.poll(now).0.into_iter().map(|r| r.data));
        }
        found.sort();
        assert_eq!(found, vec![b"A".to_vec(), b"B".to_vec()]);

        // info request with a wrong token stays unanswered
        let mut raw = net.socket(addr(9003), Conditions::default(), 5);
        let mut req = vec![P_INFO_REQUEST];
        req.extend_from_slice(&[0; 8]);
        req.extend_from_slice(&[1, 2, 3, 4]);
        raw.send_to(&req, addr(8303), now);
        a.poll(now);
        let mut buf = [0u8; 64];
        assert!(raw.recv_from(&mut buf, now).is_none());
    }
}
