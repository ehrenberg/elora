//! Server und Online-Client zusammen über ein simuliertes Netz (virtuelle Zeit).

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use elora_client::online::{OnlineClient, Status};
use elora_net::{ClientEndpoint, ClientEvent, Conditions, Keypair, MemNetwork, MemSocket};
use elora_protocol::Skin;
use elora_server::{GameServer, MapEntry, ServerConfig};
use elora_sim::{PlayerInput, Tuning};

const MAP: &[u8] = include_bytes!("../../../maps/sandbox.emap");

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
        Self::with_maps(conditions, vec![MapEntry::new("sandbox", MAP.to_vec())])
    }

    fn with_maps(conditions: Conditions, maps: Vec<MapEntry>) -> Self {
        let net = MemNetwork::new();
        let now = Instant::now();
        let cfg = ServerConfig::default();
        let server = GameServer::new(
            net.socket(addr(8303), conditions, 1),
            Keypair::generate(),
            &cfg,
            maps,
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
        self.join_with(name, Skin::default())
    }

    fn join_with(&mut self, name: &str, skin: Skin) -> usize {
        let i = self.players.len();
        let sock = self
            .net
            .socket(addr(9000 + i as u16), self.conditions, 50 + i as u64);
        self.players.push(Player {
            endpoint: ClientEndpoint::connect(sock, addr(8303), None, self.now),
            online: OnlineClient::new(name, skin, self.now),
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
fn skins_are_shared_and_updated() {
    let a = Skin {
        body: 7,
        feet: 6,
        eyes: 1,
    };
    let mut g = Game::new(lag(20));
    g.join_with("A", a);
    g.join("B");
    g.run(1000, false);
    let slot_a = g.players[0].online.slot.unwrap();
    let slot_b = g.players[1].online.slot.unwrap();
    assert_eq!(
        g.players[1].online.skins.get(&slot_a),
        Some(&a),
        "B kennt Skin von A"
    );
    assert_eq!(
        g.players[0].online.skins.get(&slot_b),
        Some(&Skin::default()),
        "A kennt Skin von B (beigetreten nach A)"
    );
    let changed = Skin {
        body: 4,
        feet: 13,
        eyes: 3,
    };
    g.players[0].online.set_skin(changed);
    g.run(300, false);
    for p in &g.players {
        assert_eq!(p.online.skins.get(&slot_a), Some(&changed));
    }
}

#[test]
fn emotes_reach_everyone_with_spam_protection() {
    let mut g = Game::new(lag(20));
    g.join("A");
    g.join("B");
    g.run(1000, false);
    for p in &mut g.players {
        p.online.take_emotes();
    }
    let slot_a = g.players[0].online.slot.unwrap();
    g.players[0].online.emote(1);
    g.players[0].online.emote(2); // zu schnell hintereinander → verworfen
    g.run(300, false);
    for p in &mut g.players {
        assert_eq!(p.online.take_emotes(), vec![(slot_a, 1)]);
    }
    g.run(800, false);
    g.players[0].online.emote(6);
    g.run(300, false);
    assert_eq!(g.players[1].online.take_emotes(), vec![(slot_a, 6)]);
}

#[test]
fn info_query_lists_server_and_players() {
    let mut g = Game::new(lag(20));
    g.join("Nimbus");
    // die Info wird einmal pro Sekunde erneuert
    g.run(1200, false);
    let mut probe = elora_net::InfoProbe::new(g.net.socket(addr(9500), lag(20), 99), 5);
    probe.query(addr(8303), g.now);
    let mut reply = None;
    for _ in 0..200 {
        g.run(4, false);
        if let Some(r) = probe.poll(g.now).0.into_iter().next() {
            reply = Some(r);
            break;
        }
    }
    let reply = reply.expect("Server antwortet");
    assert!(
        reply.ping >= Duration::from_millis(40),
        "Ping {:?}",
        reply.ping
    );
    let info = elora_protocol::ServerInfo::decode(&reply.data).unwrap();
    assert!(info.compatible());
    assert_eq!(info.map, "sandbox");
    assert_eq!(info.clients, 1);
    assert!(info.players.iter().any(|p| p.name == "Nimbus" && !p.dummy));
    assert!(
        info.players.iter().any(|p| p.dummy),
        "Dummies der Karte stehen in der Liste"
    );
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

impl Game {
    /// Führt eine Aktion am Online-Client aus und lässt die Zeit laufen.
    fn act(&mut self, i: usize, ms: u64, f: impl FnOnce(&mut OnlineClient)) {
        f(&mut self.players[i].online);
        self.run(ms, false);
    }
}

fn chat_texts(o: &OnlineClient) -> Vec<String> {
    o.chat.iter().map(|c| c.text.clone()).collect()
}

#[test]
fn names_chat_and_team_chat() {
    let mut g = Game::new(Conditions::default());
    g.server.set_rules(elora_game::RulesConfig {
        mode: elora_game::Mode::Tdm,
        ..elora_game::RulesConfig::default()
    });
    g.join("Anna");
    g.join("Ben");
    g.join("Cleo");
    g.run(500, false);
    let names: Vec<String> = g.players[0].online.names.values().cloned().collect();
    assert!(
        names.contains(&"Ben".to_owned()) && names.contains(&"Cleo".to_owned()),
        "{names:?}"
    );

    g.act(0, 100, |o| o.send_chat(false, "hallo alle"));
    // Spam-Schutz: zweite Nachricht innerhalb von 0,7 s wird verworfen
    g.act(0, 800, |o| o.send_chat(false, "zu schnell"));
    for p in &g.players {
        assert!(chat_texts(&p.online).contains(&"hallo alle".to_owned()));
        assert!(!chat_texts(&p.online).contains(&"zu schnell".to_owned()));
    }
    // Team-Chat: nur Spieler im gleichen Team
    let anna_slot = g.players[0].online.slot.unwrap();
    let anna_team = g.players[0].online.team_of(anna_slot);
    g.act(0, 200, |o| o.send_chat(true, "nur Team"));
    for p in &g.players {
        let same = p.online.team_of(p.online.slot.unwrap()) == anna_team;
        assert_eq!(chat_texts(&p.online).contains(&"nur Team".to_owned()), same);
    }
}

#[test]
fn vote_changes_mode() {
    let mut g = Game::new(Conditions::default());
    g.join("A");
    g.join("B");
    g.run(500, false);
    g.act(0, 100, |o| {
        o.call_vote(elora_protocol::VoteKind::Mode {
            mode: elora_game::Mode::Ctf,
            instagib: true,
        });
    });
    assert!(g.players[1].online.vote.is_some(), "Abstimmung sichtbar");
    g.act(1, 300, |o| o.vote(true));
    assert_eq!(g.server.rules.cfg.title(), "iCTF");
    let view = g.players[0].online.game().unwrap();
    assert_eq!(view.title(), "iCTF");
    assert!(g.players[0].online.vote.is_none());
}

#[test]
fn kill_and_spectate() {
    let mut g = Game::new(Conditions::default());
    g.join("A");
    g.run(500, false);
    let slot = g.players[0].online.slot.unwrap();
    assert!(g.server.world.character(slot).is_some());
    g.act(0, 100, OnlineClient::kill);
    assert!(g.server.world.character(slot).is_none(), "kill");
    g.act(0, 100, |o| o.set_team(elora_sim::Team::Spectator));
    g.run(4000, false);
    assert!(
        g.server.world.character(slot).is_none(),
        "Zuschauer spawnt nicht"
    );
    assert_eq!(
        g.players[0].online.team_of(slot),
        elora_sim::Team::Spectator
    );
}

#[test]
fn console_commands() {
    let mut g = Game::new(Conditions::default());
    g.join("Konsole");
    g.run(300, false);
    let now = g.now;
    assert!(g.server.command("mode tdm", now).contains("TDM"));
    assert!(g.server.command("status", now).contains("Konsole"));
    assert!(g.server.command("instagib on", now).contains("iTDM"));
    assert!(g.server.command("map sandbox", now).contains("sandbox"));
    assert!(g.server.command("gibtsnicht", now).contains("Unbekannt"));
    g.run(300, false);
    assert_eq!(
        g.players[0].online.status,
        Status::Playing,
        "nach Kartenwechsel weiter verbunden"
    );
    assert!(g.players[0].online.latest_snapshot_tick().is_some());
}

/// Große Karte (mehrere Download-Teile): eingebettetes „SVG“ aus schlecht komprimierbaren Bytes.
fn big_map() -> Vec<u8> {
    let mut m = elora_map::Map::from_rows("Gross", &["#####", "#S.S#", "#####"]).unwrap();
    let mut x: u32 = 1;
    let svg: Vec<u8> = (0..150_000)
        .map(|_| {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
            b'a' + (x >> 16) as u8 % 26
        })
        .collect();
    m.images.push(elora_map::Image {
        name: "rauschen".into(),
        svg,
    });
    let data = elora_map::encode(&m);
    assert!(data.len() > 3 * elora_protocol::MAP_CHUNK, "{}", data.len());
    data
}

#[test]
fn missing_map_is_downloaded_in_parts_and_cached() {
    let data = big_map();
    let sum = elora_map::checksum(&data);
    let mut g = Game::with_maps(lag(20), vec![MapEntry::new("gross", data.clone())]);
    g.join("Lader");
    g.run(3000, false);
    let p = &g.players[0];
    assert_eq!(p.online.status, Status::Playing);
    assert_eq!(p.online.map_name, "gross");
    assert_eq!(p.online.map.as_ref().unwrap().images.len(), 1);
    assert_eq!(g.server.player_count(), 1);

    // zweiter Besuch mit gefülltem Zwischenspeicher: keine Teile mehr nötig
    let mut store = elora_client::map_store::MemoryStore::default();
    store.maps.insert(sum, data);
    let i = g.join("Wiederkehrer");
    g.players[i].online =
        OnlineClient::new("Wiederkehrer", Skin::default(), g.now).with_store(Box::new(store));
    g.run(400, false);
    assert_eq!(
        g.players[i].online.status,
        Status::Playing,
        "sofort aus dem Speicher"
    );
    assert_eq!(g.server.player_count(), 2);
}

#[test]
fn loading_player_is_not_in_the_world_yet() {
    let mut g = Game::with_maps(lag(100), vec![MapEntry::new("gross", big_map())]);
    g.join("Langsam");
    let mut seen_partial = false;
    for _ in 0..300 {
        g.run(10, false);
        if let Status::Loading { received, size, .. } = g.players[0].online.status
            && received < size
        {
            assert_eq!(
                g.server.player_count(),
                0,
                "noch kein Slot während des Downloads"
            );
            seen_partial |= received > 0;
        }
    }
    assert!(seen_partial, "Fortschritt sichtbar");
    assert_eq!(g.players[0].online.status, Status::Playing);
}

#[test]
fn map_change_loads_new_map() {
    let mut g = Game::with_maps(
        Conditions::default(),
        vec![
            MapEntry::new("sandbox", MAP.to_vec()),
            MapEntry::new("gross", big_map()),
        ],
    );
    g.join("A");
    g.join("B");
    g.run(500, false);
    assert_eq!(g.server.player_count(), 2);
    g.server.change_map("gross", g.now).unwrap();
    g.run(2000, false);
    for p in &g.players {
        assert_eq!(p.online.status, Status::Playing);
        assert_eq!(p.online.map_name, "gross");
    }
    assert_eq!(g.server.player_count(), 2);
}
