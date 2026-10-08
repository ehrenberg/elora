//! Reliability layer of a connection (without socket and without cryptography).
//!
//! Every packet has a sequential number (which is also the encryption nonce) and
//! acknowledges the most recently received packets (highest number + 32-bit field of
//! its predecessors). Reliable messages are resent until a packet carrying them is
//! acknowledged, and are delivered in order. Unreliable messages are split into
//! fragments if needed; if a fragment is missing, the message is lost.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Payload per packet (below a typical MTU, minus header and encryption).
pub const MAX_PAYLOAD: usize = 1200;
/// Size of a fragment or chunk.
const PIECE: usize = 1000;
/// Upper limit of a single message.
pub const MAX_MESSAGE: usize = 8 * 1024 * 1024;
/// This many sent packets are remembered for acks and RTT.
const SENT_HISTORY: usize = 512;
/// Upper limit of buffered reliable messages at the receiver.
const MAX_REORDER: u64 = 16 * 1024;
const MAX_ASSEMBLIES: usize = 32;

/// Error caused by malformed or malicious packets; the connection is closed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SessionError {
    #[error("malformed packet")]
    Malformed,
    #[error("message too large")]
    TooLarge,
}

/// What the peer sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    Reliable(Vec<u8>),
    Unreliable(Vec<u8>),
    Disconnect(String),
}

#[derive(Debug)]
struct SentPacket {
    seq: u64,
    time: Instant,
    reliable: Vec<u64>,
    acked: bool,
}

#[derive(Debug)]
struct Pending {
    data: Vec<u8>,
    /// More chunks follow (large reliable message).
    more: bool,
    last_sent: Option<Instant>,
}

#[derive(Debug)]
struct Assembly {
    pieces: Vec<Option<Vec<u8>>>,
    started: Instant,
}

/// Measurements of a connection.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Stats {
    /// Smoothed round-trip time.
    pub rtt: Duration,
    /// Share of unacknowledged packets (0..1) among the most recent packets.
    pub loss: f32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug)]
pub struct Session {
    sent: VecDeque<SentPacket>,
    recv_highest: Option<u64>,
    recv_bits: u32,
    /// Received something since our last own packet (ack due).
    ack_due: bool,
    next_reliable: u64,
    unacked: BTreeMap<u64, Pending>,
    next_expected: u64,
    reorder: BTreeMap<u64, (bool, Vec<u8>)>,
    partial: Vec<u8>,
    unreliable_out: VecDeque<Vec<u8>>,
    next_fragment_msg: u64,
    assemblies: HashMap<u64, Assembly>,
    disconnect: Option<String>,
    pub stats: Stats,
    rtt_known: bool,
    last_send: Option<Instant>,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

// Frame identifiers
const F_RELIABLE: u8 = 0;
const F_UNRELIABLE: u8 = 1;
const F_FRAGMENT: u8 = 2;
const F_DISCONNECT: u8 = 3;

fn put_uvar(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

fn uvar_len(v: u64) -> usize {
    let bits = 64 - v.max(1).leading_zeros() as usize;
    bits.div_ceil(7).max(1)
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl Cursor<'_> {
    fn u8(&mut self) -> Result<u8, SessionError> {
        let v = *self.data.get(self.pos).ok_or(SessionError::Malformed)?;
        self.pos += 1;
        Ok(v)
    }

    fn uvar(&mut self) -> Result<u64, SessionError> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let b = self.u8()?;
            v |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(SessionError::Malformed)
    }

    fn bytes(&mut self) -> Result<&[u8], SessionError> {
        let len = usize::try_from(self.uvar()?).map_err(|_| SessionError::Malformed)?;
        let end = self.pos.checked_add(len).ok_or(SessionError::Malformed)?;
        let v = self
            .data
            .get(self.pos..end)
            .ok_or(SessionError::Malformed)?;
        self.pos = end;
        Ok(v)
    }
}

impl Session {
    pub fn new() -> Self {
        Self {
            sent: VecDeque::new(),
            recv_highest: None,
            recv_bits: 0,
            ack_due: false,
            next_reliable: 0,
            unacked: BTreeMap::new(),
            next_expected: 0,
            reorder: BTreeMap::new(),
            partial: Vec::new(),
            unreliable_out: VecDeque::new(),
            next_fragment_msg: 0,
            assemblies: HashMap::new(),
            disconnect: None,
            stats: Stats::default(),
            rtt_known: false,
            last_send: None,
        }
    }

    /// Send a message reliably and ordered.
    ///
    /// # Errors
    /// If the message is larger than [`MAX_MESSAGE`].
    pub fn send_reliable(&mut self, data: &[u8]) -> Result<(), SessionError> {
        if data.len() > MAX_MESSAGE {
            return Err(SessionError::TooLarge);
        }
        let pieces: Vec<&[u8]> = if data.is_empty() {
            vec![data]
        } else {
            data.chunks(PIECE).collect()
        };
        let n = pieces.len();
        for (k, piece) in pieces.into_iter().enumerate() {
            let id = self.next_reliable;
            self.next_reliable += 1;
            self.unacked.insert(
                id,
                Pending {
                    data: piece.to_vec(),
                    more: k + 1 < n,
                    last_sent: None,
                },
            );
        }
        Ok(())
    }

    /// Send a message unreliably (only with the next packet, no resending).
    ///
    /// # Errors
    /// If the message is larger than [`MAX_MESSAGE`].
    pub fn send_unreliable(&mut self, data: &[u8]) -> Result<(), SessionError> {
        if data.len() > MAX_MESSAGE {
            return Err(SessionError::TooLarge);
        }
        self.unreliable_out.push_back(data.to_vec());
        Ok(())
    }

    /// Announce disconnection; sent with the next packet.
    pub fn send_disconnect(&mut self, reason: &str) {
        self.disconnect = Some(reason.chars().take(200).collect());
    }

    fn resend_timeout(&self) -> Duration {
        let base = if self.rtt_known {
            self.stats.rtt.mul_f32(1.25)
        } else {
            Duration::from_millis(200)
        };
        base.max(Duration::from_millis(40)) + Duration::from_millis(10)
    }

    /// Should a packet be sent now (data, due ack or keepalive)?
    pub fn wants_send(&self, now: Instant, keepalive: Duration) -> bool {
        let timeout = self.resend_timeout();
        self.ack_due
            || self.disconnect.is_some()
            || !self.unreliable_out.is_empty()
            || self
                .unacked
                .values()
                .any(|p| p.last_sent.is_none_or(|t| now - t >= timeout))
            || self.last_send.is_none_or(|t| now - t >= keepalive)
    }

    /// Builds the packets to send. `next_seq` hands out packet numbers.
    pub fn build_packets(
        &mut self,
        now: Instant,
        mut next_seq: impl FnMut() -> u64,
    ) -> Vec<(u64, Vec<u8>)> {
        let mut packets = Vec::new();
        let timeout = self.resend_timeout();
        let mut due: Vec<u64> = self
            .unacked
            .iter()
            .filter(|(_, p)| p.last_sent.is_none_or(|t| now - t >= timeout))
            .map(|(&id, _)| id)
            .collect();
        due.reverse(); // as a stack: smallest ID first
        let mut frames: Vec<Vec<u8>> = Vec::new();
        if let Some(reason) = self.disconnect.take() {
            let mut f = vec![F_DISCONNECT];
            put_uvar(&mut f, reason.len() as u64);
            f.extend_from_slice(reason.as_bytes());
            frames.push(f);
        }
        // Unreliable messages (fragmented if needed)
        let mut unreliable = Vec::new();
        while let Some(msg) = self.unreliable_out.pop_front() {
            if msg.len() + 8 <= PIECE {
                let mut f = vec![F_UNRELIABLE];
                put_uvar(&mut f, msg.len() as u64);
                f.extend_from_slice(&msg);
                unreliable.push(f);
            } else {
                let id = self.next_fragment_msg;
                self.next_fragment_msg += 1;
                let count = msg.len().div_ceil(PIECE);
                for (k, piece) in msg.chunks(PIECE).enumerate() {
                    let mut f = vec![F_FRAGMENT];
                    put_uvar(&mut f, id);
                    put_uvar(&mut f, k as u64);
                    put_uvar(&mut f, count as u64);
                    put_uvar(&mut f, piece.len() as u64);
                    f.extend_from_slice(piece);
                    unreliable.push(f);
                }
            }
        }
        frames.extend(unreliable);

        let header_max = 1 + 10 + 4;
        let mut frames = frames.into_iter().peekable();
        loop {
            let mut body = Vec::with_capacity(MAX_PAYLOAD);
            let mut reliable = Vec::new();
            // due reliable messages first
            while let Some(&id) = due.last() {
                let p = &self.unacked[&id];
                let size = 1 + uvar_len(id) + 1 + uvar_len(p.data.len() as u64) + p.data.len();
                if header_max + body.len() + size > MAX_PAYLOAD && !body.is_empty() {
                    break;
                }
                due.pop();
                body.push(F_RELIABLE);
                put_uvar(&mut body, id);
                body.push(u8::from(p.more));
                put_uvar(&mut body, p.data.len() as u64);
                body.extend_from_slice(&p.data);
                reliable.push(id);
            }
            while let Some(f) = frames.peek() {
                if header_max + body.len() + f.len() > MAX_PAYLOAD && !body.is_empty() {
                    break;
                }
                if let Some(f) = frames.next() {
                    body.extend_from_slice(&f);
                }
            }
            let seq = next_seq();
            let mut packet = Vec::with_capacity(header_max + body.len());
            put_uvar(&mut packet, self.recv_highest.map_or(0, |h| h + 1));
            packet.extend_from_slice(&self.recv_bits.to_le_bytes());
            packet.extend_from_slice(&body);
            for id in &reliable {
                if let Some(p) = self.unacked.get_mut(id) {
                    p.last_sent = Some(now);
                }
            }
            self.sent.push_back(SentPacket {
                seq,
                time: now,
                reliable,
                acked: false,
            });
            while self.sent.len() > SENT_HISTORY {
                self.sent.pop_front();
            }
            self.stats.packets_sent += 1;
            self.stats.bytes_sent += packet.len() as u64;
            packets.push((seq, packet));
            if due.is_empty() && frames.peek().is_none() {
                break;
            }
        }
        self.ack_due = false;
        self.last_send = Some(now);
        self.update_loss();
        packets
    }

    fn update_loss(&mut self) {
        // packets old enough for an ack (older than 2 RTT)
        let n = self.sent.len().saturating_sub(4);
        if n == 0 {
            return;
        }
        let lost = self.sent.iter().take(n).filter(|p| !p.acked).count();
        self.stats.loss = lost as f32 / n as f32;
    }

    fn ack(&mut self, seq: u64, now: Instant) {
        let Some(p) = self.sent.iter_mut().find(|p| p.seq == seq) else {
            return;
        };
        if p.acked {
            return;
        }
        p.acked = true;
        for id in std::mem::take(&mut p.reliable) {
            self.unacked.remove(&id);
        }
        let sample = now.saturating_duration_since(p.time);
        self.stats.rtt = if self.rtt_known {
            self.stats.rtt.mul_f32(0.9) + sample.mul_f32(0.1)
        } else {
            sample
        };
        self.rtt_known = true;
    }

    /// Processes a received (already decrypted) packet.
    ///
    /// # Errors
    /// On malformed data; the connection should then be closed.
    pub fn receive(
        &mut self,
        seq: u64,
        payload: &[u8],
        now: Instant,
    ) -> Result<Vec<Delivery>, SessionError> {
        self.stats.packets_received += 1;
        self.stats.bytes_received += payload.len() as u64;
        self.ack_due = true;
        match self.recv_highest {
            None => {
                self.recv_highest = Some(seq);
                self.recv_bits = 0;
            }
            Some(h) if seq > h => {
                let shift = seq - h;
                let shifted = if shift >= 32 {
                    0
                } else {
                    self.recv_bits << shift
                };
                let previous = if shift <= 32 { 1u32 << (shift - 1) } else { 0 };
                self.recv_bits = shifted | previous;
                self.recv_highest = Some(seq);
            }
            Some(h) if seq < h && h - seq <= 32 => self.recv_bits |= 1 << (h - seq - 1),
            _ => {}
        }

        let mut c = Cursor {
            data: payload,
            pos: 0,
        };
        let ack = c.uvar()?;
        let bits = u32::from_le_bytes([c.u8()?, c.u8()?, c.u8()?, c.u8()?]);
        if let Some(highest) = ack.checked_sub(1) {
            self.ack(highest, now);
            for k in 0..32u64 {
                if bits & (1 << k) != 0
                    && let Some(s) = highest.checked_sub(k + 1)
                {
                    self.ack(s, now);
                }
            }
        }

        let mut out = Vec::new();
        while c.pos < payload.len() {
            match c.u8()? {
                F_RELIABLE => {
                    let id = c.uvar()?;
                    let more = c.u8()? != 0;
                    let data = c.bytes()?.to_vec();
                    if id >= self.next_expected {
                        if id - self.next_expected > MAX_REORDER {
                            return Err(SessionError::Malformed);
                        }
                        self.reorder.insert(id, (more, data));
                    }
                }
                F_UNRELIABLE => out.push(Delivery::Unreliable(c.bytes()?.to_vec())),
                F_FRAGMENT => {
                    let id = c.uvar()?;
                    let idx = usize::try_from(c.uvar()?).map_err(|_| SessionError::Malformed)?;
                    let count = usize::try_from(c.uvar()?).map_err(|_| SessionError::Malformed)?;
                    let data = c.bytes()?.to_vec();
                    if count == 0 || idx >= count || count > MAX_MESSAGE / PIECE + 1 {
                        return Err(SessionError::Malformed);
                    }
                    if let Some(msg) = self.fragment(id, idx, count, data, now) {
                        out.push(Delivery::Unreliable(msg));
                    }
                }
                F_DISCONNECT => {
                    let reason = String::from_utf8_lossy(c.bytes()?).into_owned();
                    out.push(Delivery::Disconnect(reason));
                }
                _ => return Err(SessionError::Malformed),
            }
        }
        // deliver reliable messages in order
        while let Some((more, data)) = self.reorder.remove(&self.next_expected) {
            self.next_expected += 1;
            if self.partial.len() + data.len() > MAX_MESSAGE {
                return Err(SessionError::TooLarge);
            }
            self.partial.extend_from_slice(&data);
            if !more {
                out.push(Delivery::Reliable(std::mem::take(&mut self.partial)));
            }
        }
        Ok(out)
    }

    fn fragment(
        &mut self,
        id: u64,
        idx: usize,
        count: usize,
        data: Vec<u8>,
        now: Instant,
    ) -> Option<Vec<u8>> {
        self.assemblies
            .retain(|_, a| now - a.started < Duration::from_secs(2));
        if self.assemblies.len() >= MAX_ASSEMBLIES && !self.assemblies.contains_key(&id) {
            let oldest = *self.assemblies.keys().min()?;
            self.assemblies.remove(&oldest);
        }
        let a = self.assemblies.entry(id).or_insert_with(|| Assembly {
            pieces: vec![None; count],
            started: now,
        });
        if a.pieces.len() != count {
            return None;
        }
        a.pieces[idx] = Some(data);
        if a.pieces.iter().all(Option::is_some) {
            let a = self.assemblies.remove(&id)?;
            Some(a.pieces.into_iter().flatten().flatten().collect())
        } else {
            None
        }
    }

    /// Number of reliable messages not yet acknowledged.
    pub fn pending_reliable(&self) -> usize {
        self.unacked.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two sessions over a lossy, reordering link.
    struct Link {
        a: Session,
        b: Session,
        seq_a: u64,
        seq_b: u64,
        now: Instant,
        rng: u64,
    }

    impl Link {
        fn new() -> Self {
            Self {
                a: Session::new(),
                b: Session::new(),
                seq_a: 0,
                seq_b: 0,
                now: Instant::now(),
                rng: 7,
            }
        }

        fn rand(&mut self) -> u64 {
            self.rng ^= self.rng << 13;
            self.rng ^= self.rng >> 7;
            self.rng ^= self.rng << 17;
            self.rng % 100
        }

        /// One time step; `loss` in percent. Returns what b received from a.
        fn step(&mut self, loss: u64) -> Vec<Delivery> {
            self.now += Duration::from_millis(20);
            let mut sa = self.seq_a;
            let pa = self.a.build_packets(self.now, || {
                sa += 1;
                sa
            });
            self.seq_a = sa;
            let mut sb = self.seq_b;
            let pb = self.b.build_packets(self.now, || {
                sb += 1;
                sb
            });
            self.seq_b = sb;
            let mut got = Vec::new();
            let mut pa: Vec<_> = pa.into_iter().filter(|_| self.rand() >= loss).collect();
            if pa.len() > 1 {
                pa.reverse(); // reordering
            }
            for (seq, p) in pa {
                got.extend(self.b.receive(seq, &p, self.now).unwrap());
            }
            for (seq, p) in pb {
                if self.rand() >= loss {
                    self.a.receive(seq, &p, self.now).unwrap();
                }
            }
            got
        }
    }

    #[test]
    fn reliable_messages_arrive_in_order_despite_loss() {
        let mut l = Link::new();
        for k in 0..200u32 {
            l.a.send_reliable(&k.to_le_bytes()).unwrap();
        }
        let big: Vec<u8> = (0..50_000u32).map(|v| v as u8).collect();
        l.a.send_reliable(&big).unwrap();
        let mut got = Vec::new();
        for _ in 0..400 {
            got.extend(l.step(30));
        }
        let expected: Vec<Delivery> = (0..200u32)
            .map(|k| Delivery::Reliable(k.to_le_bytes().to_vec()))
            .chain(std::iter::once(Delivery::Reliable(big)))
            .collect();
        assert_eq!(got, expected);
        assert_eq!(l.a.pending_reliable(), 0);
    }

    #[test]
    fn large_unreliable_messages_are_fragmented() {
        let mut l = Link::new();
        let msg: Vec<u8> = (0..5000u32).map(|v| (v * 7) as u8).collect();
        l.a.send_unreliable(&msg).unwrap();
        let got = l.step(0);
        assert_eq!(got, vec![Delivery::Unreliable(msg)]);
    }

    #[test]
    fn rtt_is_measured() {
        let mut l = Link::new();
        for _ in 0..20 {
            l.step(0);
        }
        let rtt = l.a.stats.rtt;
        assert!(
            rtt >= Duration::from_millis(15) && rtt <= Duration::from_millis(45),
            "{rtt:?}"
        );
    }

    #[test]
    fn disconnect_is_delivered() {
        let mut l = Link::new();
        l.a.send_disconnect("Tschüss");
        assert_eq!(l.step(0), vec![Delivery::Disconnect("Tschüss".into())]);
    }

    #[test]
    fn rejects_garbage() {
        let mut s = Session::new();
        let now = Instant::now();
        assert!(s.receive(1, &[0xff; 3], now).is_err());
        assert!(s.receive(2, &[0, 0, 0, 0, 0, 9], now).is_err());
    }
}
