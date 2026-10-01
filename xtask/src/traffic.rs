//! Synthetischer Spielverkehr zum Trainieren und Messen der Kompression (E-063).

use elora_protocol::{ClientMsg, ServerMsg, Snapshot};
use elora_sim::{PlayerInput, Tuning, Weapon};

const MAP: &[u8] = include_bytes!("../../maps/sandbox.emap");

/// Aufgezeichnete, unkomprimierte Nachrichten.
pub struct Traffic {
    pub server: Vec<Vec<u8>>,
    pub client: Vec<Vec<u8>>,
    /// Dieselben Snapshots, kodiert wie im Original (zum Vergleich).
    pub original: Vec<Vec<u8>>,
}

fn rng(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// Simuliert `players` Menschen (plus Dummies der Karte) für `ticks` Ticks.
/// Snapshots werden wie beim Server alle 2 Ticks als Delta gegen einen um
/// `ack_lag` Snapshots älteren Stand erzeugt.
pub fn generate(players: usize, seed: u64, ticks: u64, ack_lag: usize) -> Traffic {
    let map = elora_map::decode(MAP).expect("Sandbox-Karte gültig");
    let mut world = map.world(Tuning::default());
    let slots: Vec<usize> = (0..players).map(|_| world.join()).collect();
    let mut state = seed | 1;
    let mut inputs = vec![PlayerInput::default(); world.players.len() + players];
    let mut history: Vec<Snapshot> = Vec::new();
    let mut traffic = Traffic {
        server: Vec::new(),
        client: Vec::new(),
        original: Vec::new(),
    };
    for t in 0..ticks {
        for &s in &slots {
            // Waffen verteilen, damit geschossen wird
            if let Some(ch) = world.character_mut(s) {
                for w in [Weapon::Grenade, Weapon::Laser] {
                    ch.arsenal.give(w, 10, 10);
                }
            }
            let r = rng(&mut state);
            let i = &mut inputs[s];
            if r.is_multiple_of(20) {
                i.direction = [-1, 0, 1][(r / 20 % 3) as usize];
            }
            i.jump = r % 37 < 3;
            i.hook = (r / 7) % 50 < 12;
            i.target_x = ((r >> 8) % 600) as i32 - 300;
            i.target_y = ((r >> 20) % 400) as i32 - 300;
            if r.is_multiple_of(15) {
                i.fire = i.fire.wrapping_add(1) & 0x3f;
            }
            if r.is_multiple_of(200) {
                i.wanted_weapon = (r / 200 % 3) as u8 + 1;
            } else {
                i.wanted_weapon = 0;
            }
            let msg = ClientMsg::Input {
                ack: Some(t),
                inputs: vec![(t + 1, *i), (t + 2, *i), (t + 3, *i), (t + 4, *i)],
            };
            traffic.client.push(msg.encode_raw());
        }
        world.step(&inputs);
        if t % 2 == 0 {
            let snap = Snapshot::from_world(&world);
            let base = history.len().checked_sub(ack_lag).map(|i| &history[i]);
            let msg = ServerMsg::snapshot(&snap, base, world.events.clone());
            traffic.server.push(msg.encode_raw());
            let mut w = elora_protocol::codec::Writer::new();
            snap.encode_delta_like_original(base, &mut w);
            traffic.original.push(w.into_bytes());
            history.push(snap);
        }
    }
    traffic
}
