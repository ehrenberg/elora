//! Hüterin der Frostspitzen (Eiskönigin Kristella, R2-M2.4, E-341): Frostwellen, Erschöpfung,
//! Eiszapfen, Schneesturm.

use elora_sim::creature::queen;
use elora_sim::{
    Abilities, Behavior, Collision, CreatureAct, CreatureKind, Event, PlayerInput, QueenDef, Tile,
    Tuning, Vec2, World,
};

const W: usize = 50;
const H: usize = 30;
const FLOOR: usize = 28;
/// Mitte der Halle.
const MID: f32 = 25.0 * 32.0;

fn kind(name: &str, size: [f32; 2], health: i32, behavior: Behavior) -> CreatureKind {
    CreatureKind {
        name: name.into(),
        size,
        health,
        touch_damage: 2,
        small: false,
        boss: name == "kristella",
        xp: 0,
        loot: Vec::new(),
        daze_ms: 0,
        freeze_ms: 0,
        armor: false,
        behavior,
    }
}

/// Halle mit Decke in Zeile 8 und einem Sims (Zeile 25, Spalten 10–14) über dem Boden.
fn hall() -> World {
    let mut tiles = vec![Tile::Air; W * H];
    for x in 0..W {
        tiles[8 * W + x] = Tile::Solid;
        tiles[FLOOR * W + x] = Tile::Ice;
        tiles[(H - 1) * W + x] = Tile::Solid;
    }
    for y in 0..H {
        tiles[y * W] = Tile::Climb;
        tiles[y * W + W - 1] = Tile::Climb;
    }
    for x in 10..15 {
        tiles[25 * W + x] = Tile::Solid;
    }
    let mut w = World::new(Tuning::default(), Collision::new(W, H, tiles));
    w.adventure = true;
    w.creature_kinds = vec![
        kind(
            "kristella",
            [90.0, 110.0],
            36,
            Behavior::Queen(Box::new(QueenDef {
                sight: 640.0,
                width: 1100.0,
                hover_ms: 2600,
                waves: 3,
                wave_speed: 6.0,
                fresh_len: 170.0,
                wave_damage: 2,
                stun_ms: 3400,
                enrage_at: 0.5,
                icicles: 3,
                storm_at: 0.25,
            })),
        ),
        kind(
            "eiszapfen",
            [20.0, 48.0],
            1,
            Behavior::Icicle {
                sight: 48.0,
                reach: 420.0,
                warn_ms: 450,
            },
        ),
    ];
    w
}

fn spawn(w: &mut World, at: Vec2) {
    let i = w.join();
    w.set_abilities(i, Abilities::NONE);
    w.spawn_character(i, at);
    w.character_mut(i).unwrap().invulnerable_until = 0;
}

/// Auf dem Boden der Halle in Spalte `tx`.
fn floor_at(tx: usize) -> Vec2 {
    #[allow(clippy::cast_precision_loss)]
    Vec2::new(tx as f32 * 32.0 + 16.0, FLOOR as f32 * 32.0 - 15.0)
}

fn queen_id(w: &mut World) -> u32 {
    w.add_creature(0, Vec2::new(MID, 14.0 * 32.0)).unwrap()
}

fn mode(w: &World, id: u32) -> u8 {
    w.creatures.iter().find(|c| c.id == id).unwrap().mode
}

fn health(w: &World, id: u32) -> i32 {
    w.creatures.iter().find(|c| c.id == id).unwrap().health
}

/// Läuft, bis `until` gilt (höchstens `max` Ticks); sammelt die Ereignisse.
fn run_until(w: &mut World, max: u32, until: impl Fn(&World) -> bool) -> Vec<Event> {
    let mut all = Vec::new();
    for _ in 0..max {
        w.step(&[PlayerInput::default()]);
        all.extend(w.events.iter().cloned());
        if until(w) {
            break;
        }
    }
    all
}

#[test]
fn sleeps_until_elora_comes_then_wakes() {
    let mut w = hall();
    spawn(
        &mut w,
        Vec2::new(MID + 900.0, 0.0) + Vec2::new(0.0, floor_at(0).y),
    );
    w.spawn_character(0, floor_at(47));
    let id = queen_id(&mut w);
    run_until(&mut w, 50, |_| false);
    // Spalte 47 ist über 640 entfernt
    assert_eq!(mode(&w, id), queen::SLEEP);
    w.spawn_character(0, floor_at(30));
    let ev = run_until(&mut w, 5, |_| false);
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::CreatureAct {
            act: CreatureAct::Wake,
            ..
        }
    )));
    assert_eq!(mode(&w, id), queen::HOVER);
}

#[test]
fn frost_wave_hurts_on_the_floor_but_not_on_a_ledge() {
    // Elora am Boden
    let mut w = hall();
    spawn(&mut w, floor_at(20));
    let id = queen_id(&mut w);
    let before = w.character(0).unwrap().health;
    run_until(&mut w, 600, |w| {
        w.character(0).is_none_or(|c| c.health < before)
    });
    assert!(
        w.character(0).unwrap().health < before,
        "frischer Frost trifft"
    );
    assert_eq!(mode(&w, id), queen::WAVE);
    // Elora auf dem Sims: die Welle läuft unter ihr durch
    let mut w = hall();
    #[allow(clippy::cast_precision_loss)]
    let ledge = Vec2::new(12.0 * 32.0 + 16.0, 25.0 * 32.0 - 15.0);
    spawn(&mut w, ledge);
    let id = queen_id(&mut w);
    let before = w.character(0).unwrap().health;
    let mut waves = 0;
    let mut last = queen::HOVER;
    for _ in 0..900 {
        w.step(&[PlayerInput::default()]);
        let m = mode(&w, id);
        if last == queen::WAVE && m != queen::WAVE {
            waves += 1;
        }
        last = m;
    }
    assert!(waves >= 2, "Wellen sind gelaufen: {waves}");
    assert_eq!(w.character(0).unwrap().health, before, "oben sicher");
}

#[test]
fn only_vulnerable_while_tired_after_three_waves() {
    let mut w = hall();
    #[allow(clippy::cast_precision_loss)]
    let ledge = Vec2::new(12.0 * 32.0 + 16.0, 25.0 * 32.0 - 15.0);
    spawn(&mut w, ledge);
    let id = queen_id(&mut w);
    run_until(&mut w, 200, |w| mode(w, w.creatures[0].id) == queen::HOVER);
    w.hurt_creature(id, 5);
    assert_eq!(health(&w, id), 36, "schwebend unverwundbar");
    run_until(&mut w, 1500, |w| mode(w, w.creatures[0].id) == queen::TIRED);
    assert_eq!(mode(&w, id), queen::TIRED, "nach drei Wellen erschöpft");
    // sinkt bis auf den Boden
    run_until(&mut w, 60, |_| false);
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    assert!(
        (c.pos.y + 55.0 - FLOOR as f32 * 32.0).abs() < 2.0,
        "liegt am Boden"
    );
    w.hurt_creature(id, 5);
    assert_eq!(health(&w, id), 31, "erschöpft verwundbar");
    run_until(&mut w, 400, |w| mode(w, w.creatures[0].id) == queen::HOVER);
    assert_eq!(mode(&w, id), queen::HOVER, "steigt wieder auf");
}

#[test]
fn angry_queen_alternates_sides_drops_icicles_and_calls_a_storm() {
    let mut w = hall();
    #[allow(clippy::cast_precision_loss)]
    let ledge = Vec2::new(12.0 * 32.0 + 16.0, 25.0 * 32.0 - 15.0);
    spawn(&mut w, ledge);
    let id = queen_id(&mut w);
    run_until(&mut w, 10, |_| false);
    // auf ein Viertel herunter (nur erschöpft möglich: Leben direkt setzen)
    w.creatures.iter_mut().find(|c| c.id == id).unwrap().health = 8;
    let mut sides = Vec::new();
    let mut icicles = false;
    let mut storm = false;
    for _ in 0..1500 {
        w.step(&[PlayerInput::default()]);
        let c = w.creatures.iter().find(|c| c.id == id).unwrap().clone();
        if c.mode == queen::WAVE && sides.last().is_none_or(|&(m, _)| m != c.count) {
            sides.push((c.count, c.facing));
        }
        icicles |= w.creatures.iter().any(|c| c.kind == 1);
        storm |= w.events.iter().any(|e| {
            matches!(
                e,
                Event::CreatureAct {
                    act: CreatureAct::Storm,
                    ..
                }
            )
        });
    }
    assert!(storm, "Schneesturm im letzten Viertel");
    assert!(icicles, "Eiszapfen zwischen den Wellen");
    let dirs: Vec<i8> = sides.iter().map(|s| s.1).collect();
    assert!(
        dirs.windows(2).any(|p| p[0] != p[1]),
        "Wellen von beiden Seiten: {dirs:?}"
    );
}
