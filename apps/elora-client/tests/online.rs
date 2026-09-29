//! Server und Online-Client zusammen über ein simuliertes Netz (virtuelle Zeit).

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use elora_client::online::{OnlineClient, Status};
use elora_net::{ClientEndpoint, ClientEvent, Conditions, Keypair, MemNetwork, MemSocket};
use elora_server::{GameServer, ServerConfig};
use elora_sim::{PlayerInput, Tuning};

const MAP: &str = include_str!("../../../maps/sandbox.emap.toml");

fn addr(port: u16) -> SocketAddr {
    SocketAddr::from(([10, 0, 0, 1], port))
}

struct Player {
    endpoint: ClientEndpoint<MemSocket>,
    online: OnlineClient,
    input: PlayerInput,
}

struct Game {
    server: GameServer<MemSocket>,
    players: Vec<Player>,
    net: MemNetwork,
    conditions: Conditions,
    now: Instant,
    max_correction_after_warmup: f32,
}

impl Game {
    fn new(conditions: Conditions) -> Self {
        let net = MemNetwork::new();
        let now = Instant::now();
        let cfg = ServerConfig::default();
        let server = GameServer::new(
            net.socket(addr(8303), conditions, 1),
            Keypair::generate(),
            &cfg,
            "sandbox",
            MAP.to_owned(),
            Tuning::default(),
            now,
        )
        .unwrap();
        Self {
            server,
            players: Vec::new(),
            net,
            conditions,
            now,
            max_correction_after_warmup: 0.0,
        }
    }

    fn join(&mut self, name: &str) -> usize {
        let i = self.players.len();
        let sock = self
            .net
            .socket(addr(9000 + i as u16), self.conditions, 50 + i as u64);
        self.players.push(Player {
            endpoint: ClientEndpoint::connect(sock, addr(8303), None, self.now),
            online: OnlineClient::new(name, self.now),
            input: PlayerInput::default(),
        });
        i
    }

    fn run(&mut self, ms: u64, warmup_done: bool) {
        for _ in 0..ms / 2 {
            self.now += Duration::from_millis(2);
            let now = self.now;
            for p in &mut self.players {
                for e in p.endpoint.poll(now) {
                    match e {
                        ClientEvent::Connected { .. } => p.online.on_connected(),
                        ClientEvent::Message { data, .. } => p.online.on_message(&data, now),
                        ClientEvent::Disconnected(r) => p.online.on_disconnected(format!("{r:?}")),
                    }
                }
                let input = p.input;
                p.online.update(now, || input);
                for (data, reliable) in p.online.take_outgoing() {
                    p.endpoint.send(&data, reliable);
                }
                p.endpoint.flush(now);
                if warmup_done {
                    self.max_correction_after_warmup = self
                        .max_correction_after_warmup
                        .max(p.online.info.correction);
                }
            }
            self.server.update(now);
        }
    }
}

fn lag(ms: u64) -> Conditions {
    Conditions {
        latency: Duration::from_millis(ms),
        ..Conditions::default()
    }
}

#[test]
fn client_joins_and_receives_snapshots() {
    let mut g = Game::new(Conditions::default());
    g.join("Elora");
    g.run(1000, false);
    let p = &g.players[0];
    assert_eq!(p.online.status, Status::Playing);
    assert_eq!(g.server.player_count(), 1);
    assert!(p.online.latest_snapshot_tick().unwrap() > 30);
    assert_eq!(p.online.info.snapshot_errors, 0);
    assert!(
        p.online.scene(g.now).unwrap().local().is_some(),
        "Elora ist gespawnt"
    );
}

#[test]
fn prediction_matches_server_at_100ms_ping() {
    let mut g = Game::new(lag(50)); // 50 ms pro Richtung = 100 ms Ping
    g.join("Elora");
    g.run(2000, false);
    // Laufen, Springen, Hooken
    for step in 0..10 {
        g.players[0].input = PlayerInput {
            direction: [1, 1, -1, 0, 1][step % 5],
            jump: step % 3 == 0,
            hook: step % 4 == 1,
            target_x: 100,
            target_y: -60,
            ..PlayerInput::default()
        };
        g.run(300, true);
    }
    let p = &g.players[0];
    eprintln!(
        "Info: {:?}, Snapshot-Tick {:?}, Server-Tick {}, max. Korrektur {}",
        p.online.info,
        p.online.latest_snapshot_tick(),
        g.server.world.tick,
        g.max_correction_after_warmup
    );
    assert_eq!(p.online.info.snapshot_errors, 0);
    assert!(
        p.online.info.prediction_ticks >= 5,
        "bei 100 ms Ping ≥ 5 Ticks Vorhersage: {}",
        p.online.info.prediction_ticks
    );
    assert!(
        g.max_correction_after_warmup < 0.01,
        "Vorhersage weicht vom Server ab: {} Einheiten",
        g.max_correction_after_warmup
    );
    assert!(
        p.online.info.input_time_left_ms >= 0,
        "Eingaben kommen rechtzeitig an"
    );
}

#[test]
fn two_players_see_each_other() {
    let mut g = Game::new(lag(20));
    g.join("A");
    g.join("B");
    g.run(1500, false);
    for p in &g.players {
        let scene = p.online.scene(g.now).unwrap();
        let humans = scene.chars.iter().filter(|c| !c.dummy).count();
        assert_eq!(humans, 2, "beide Spieler sichtbar");
        assert!(
            scene.chars.iter().any(|c| c.dummy),
            "Dummies der Karte sichtbar"
        );
    }
}

#[test]
fn survives_loss_and_jitter() {
    let bad = Conditions {
        latency: Duration::from_millis(40),
        jitter: Duration::from_millis(30),
        loss: 0.1,
        duplicate: 0.02,
    };
    let mut g = Game::new(bad);
    g.join("Elora");
    g.run(3000, false);
    let mut corrections = 0;
    for k in 0..40 {
        g.players[0].input = PlayerInput {
            direction: [1, -1][k % 2],
            jump: k % 3 == 0,
            ..PlayerInput::default()
        };
        g.run(100, false);
        if g.players[0].online.info.correction > 0.0 {
            corrections += 1;
        }
    }
    let p = &g.players[0];
    eprintln!(
        "Info bei Verlust: {:?}, Stichproben mit Korrektur: {corrections}/40",
        p.online.info
    );
    assert_eq!(p.online.status, Status::Playing);
    assert!(p.online.latest_snapshot_tick().unwrap() > 200);
}
