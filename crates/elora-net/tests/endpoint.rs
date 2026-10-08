//! Connection tests over an in-memory network with virtual time.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use elora_net::{
    ClientEndpoint, ClientEvent, Conditions, DisconnectReason, Keypair, MemNetwork, MemSocket,
    ServerEndpoint, ServerEvent,
};

fn addr(port: u16) -> SocketAddr {
    SocketAddr::from(([10, 0, 0, 1], port))
}

struct Harness {
    server: ServerEndpoint<MemSocket>,
    clients: Vec<ClientEndpoint<MemSocket>>,
    net: MemNetwork,
    now: Instant,
    server_events: Vec<ServerEvent>,
    client_events: Vec<Vec<ClientEvent>>,
    conditions: Conditions,
}

impl Harness {
    fn new(conditions: Conditions, max_clients: usize) -> Self {
        let net = MemNetwork::new();
        let server = ServerEndpoint::new(
            net.socket(addr(8303), conditions, 1),
            Keypair::generate(),
            max_clients,
        );
        Self {
            server,
            clients: Vec::new(),
            net,
            now: Instant::now(),
            server_events: Vec::new(),
            client_events: Vec::new(),
            conditions,
        }
    }

    fn connect(&mut self, expected_key: Option<Vec<u8>>) -> usize {
        let i = self.clients.len();
        let sock = self
            .net
            .socket(addr(9000 + i as u16), self.conditions, 100 + i as u64);
        self.clients.push(ClientEndpoint::connect(
            sock,
            addr(8303),
            expected_key,
            self.now,
        ));
        self.client_events.push(Vec::new());
        i
    }

    fn step(&mut self, ms: u64) {
        self.now += Duration::from_millis(ms);
        for (i, c) in self.clients.iter_mut().enumerate() {
            self.client_events[i].extend(c.poll(self.now));
            c.flush(self.now);
        }
        self.server_events.extend(self.server.poll(self.now));
        self.server.flush(self.now);
    }

    fn run(&mut self, ms: u64) {
        for _ in 0..ms / 10 {
            self.step(10);
        }
    }

    fn client_messages(&self, i: usize) -> Vec<(bool, Vec<u8>)> {
        self.client_events[i]
            .iter()
            .filter_map(|e| match e {
                ClientEvent::Message { reliable, data } => Some((*reliable, data.clone())),
                _ => None,
            })
            .collect()
    }

    fn server_messages(&self) -> Vec<Vec<u8>> {
        self.server_events
            .iter()
            .filter_map(|e| match e {
                ServerEvent::Message {
                    reliable: true,
                    data,
                    ..
                } => Some(data.clone()),
                _ => None,
            })
            .collect()
    }
}

#[test]
fn handshake_and_messages_both_ways() {
    let mut n = Harness::new(Conditions::default(), 8);
    let c = n.connect(None);
    n.run(200);
    let key = n.server.public_key().to_vec();
    assert!(n.clients[c].is_connected());
    assert!(n.client_events[c].contains(&ClientEvent::Connected { server_key: key }));
    let id = match n.server_events[0] {
        ServerEvent::Connected { id, .. } => id,
        ref e => panic!("unerwartet: {e:?}"),
    };

    n.clients[c].send(b"hallo", true);
    n.server.send(id, b"welt", true);
    n.server.send(id, &[7; 3000], false);
    n.run(100);
    assert_eq!(n.server_messages(), vec![b"hallo".to_vec()]);
    let got = n.client_messages(c);
    assert!(got.contains(&(true, b"welt".to_vec())));
    assert!(
        got.contains(&(false, vec![7; 3000])),
        "großes unzuverlässiges Paket"
    );
}

#[test]
fn survives_latency_jitter_and_loss() {
    let bad = Conditions {
        latency: Duration::from_millis(50),
        jitter: Duration::from_millis(20),
        loss: 0.2,
        duplicate: 0.05,
    };
    let mut n = Harness::new(bad, 8);
    let c = n.connect(None);
    n.run(3000);
    assert!(n.clients[c].is_connected(), "Handshake trotz 20 % Verlust");
    for k in 0..100u32 {
        n.clients[c].send(&k.to_le_bytes(), true);
    }
    n.run(5000);
    let expected: Vec<Vec<u8>> = (0..100u32).map(|k| k.to_le_bytes().to_vec()).collect();
    assert_eq!(
        n.server_messages(),
        expected,
        "vollständig, in Reihenfolge, ohne Duplikate"
    );
    let rtt = n.clients[c].stats().unwrap().rtt;
    assert!(
        rtt >= Duration::from_millis(90),
        "RTT ≈ 2 × 50 ms + Jitter: {rtt:?}"
    );
}

#[test]
fn key_mismatch_is_reported() {
    let mut n = Harness::new(Conditions::default(), 8);
    let c = n.connect(Some(vec![1; 32]));
    n.run(200);
    assert!(!n.clients[c].is_connected());
    assert!(matches!(
        n.client_events[c].last(),
        Some(ClientEvent::Disconnected(
            DisconnectReason::KeyMismatch { .. }
        ))
    ));
    // matching key connects
    let key = n.server.public_key().to_vec();
    let c2 = n.connect(Some(key));
    n.run(200);
    assert!(n.clients[c2].is_connected());
}

#[test]
fn full_server_rejects() {
    let mut n = Harness::new(Conditions::default(), 1);
    let a = n.connect(None);
    n.run(200);
    let b = n.connect(None);
    n.run(200);
    assert!(n.clients[a].is_connected());
    assert_eq!(
        n.client_events[b].last(),
        Some(&ClientEvent::Disconnected(DisconnectReason::Rejected(
            "#server-full".into()
        )))
    );
}

#[test]
fn disconnect_reason_and_timeout() {
    let mut n = Harness::new(Conditions::default(), 8);
    let a = n.connect(None);
    let b = n.connect(None);
    n.run(200);
    let id_a = n
        .server_events
        .iter()
        .find_map(|e| match e {
            ServerEvent::Connected { id, addr: from } if from.port() == 9000 => Some(*id),
            _ => None,
        })
        .unwrap();
    n.server.disconnect(id_a, "gekickt", n.now);
    n.run(100);
    assert!(
        n.client_events[a].contains(&ClientEvent::Disconnected(DisconnectReason::Remote(
            "gekickt".into()
        )))
    );

    // client b goes silent → server reports a timeout after 10 s
    n.clients.remove(b);
    n.client_events.remove(b);
    n.run(11_000);
    assert!(n.server_events.iter().any(|e| matches!(
        e,
        ServerEvent::Disconnected {
            reason: DisconnectReason::Timeout,
            ..
        }
    )));
    assert_eq!(n.server.client_count(), 0);
}

#[test]
fn keepalive_holds_idle_connection() {
    let mut n = Harness::new(Conditions::default(), 8);
    let c = n.connect(None);
    n.run(30_000);
    assert!(n.clients[c].is_connected());
    assert_eq!(n.server.client_count(), 1);
}
