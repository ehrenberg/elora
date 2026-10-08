//! Frostspitzen enemies (R2-M2.4): snowball seal, ice-spike bat, frost ghost.

use elora_sim::creature::{bat, ghost, seal};
use elora_sim::{
    Abilities, Behavior, Collision, CreatureAct, CreatureKind, Event, PlayerInput, Tile, Tuning,
    Vec2, World,
};

const W: usize = 70;
const H: usize = 30;
const FLOOR: usize = 28;

fn kind(name: &str, size: [f32; 2], freeze_ms: u32, behavior: Behavior) -> CreatureKind {
    CreatureKind {
        name: name.into(),
        size,
        health: 3,
        touch_damage: 1,
        small: false,
        boss: false,
        xp: 0,
        loot: Vec::new(),
        daze_ms: 0,
        freeze_ms,
        armor: false,
        behavior,
    }
}

const SEAL: usize = 0;
const BAT: usize = 1;
const GHOST: usize = 2;

fn world(edit: impl FnOnce(&mut Vec<Tile>)) -> World {
    let mut tiles = vec![Tile::Air; W * H];
    for x in 0..W {
        tiles[FLOOR * W + x] = Tile::Solid;
        tiles[(H - 1) * W + x] = Tile::Solid;
    }
    for y in 0..H {
        tiles[y * W] = Tile::Solid;
        tiles[y * W + W - 1] = Tile::Solid;
    }
    edit(&mut tiles);
    let mut w = World::new(Tuning::default(), Collision::new(W, H, tiles));
    w.adventure = true;
    w.creature_kinds = vec![
        kind(
            "schneeballrobbe",
            [48.0, 28.0],
            0,
            Behavior::Seal {
                sight: 520.0,
                speed: 3.2,
                range: 300.0,
                interval_ms: 1500,
                shot_speed: 5.5,
                shot_damage: 1,
            },
        ),
        kind(
            "fledermaus",
            [32.0, 36.0],
            0,
            Behavior::Bat {
                sight: 90.0,
                reach: 380.0,
                speed: 6.5,
            },
        ),
        kind(
            "frostgeist",
            [40.0, 52.0],
            600,
            Behavior::Ghost {
                sight: 460.0,
                speed: 1.6,
                flee_ms: 1800,
            },
        ),
    ];
    w
}

fn standing(tx: usize) -> Vec2 {
    #[allow(clippy::cast_precision_loss)]
    Vec2::new(tx as f32 * 32.0 + 16.0, FLOOR as f32 * 32.0 - 15.0)
}

fn spawn(w: &mut World, tx: usize) {
    let i = w.join();
    w.set_abilities(i, Abilities::NONE);
    w.spawn_character(i, standing(tx));
    w.character_mut(i).unwrap().invulnerable_until = 0;
}

fn run(w: &mut World, input: PlayerInput, ticks: u32) -> Vec<Event> {
    let mut all = Vec::new();
    for _ in 0..ticks {
        w.step(&[input]);
        all.extend(w.events.iter().cloned());
    }
    all
}

fn idle() -> PlayerInput {
    PlayerInput::default()
}

fn creature(w: &World, id: u32) -> &elora_sim::Creature {
    w.creatures.iter().find(|c| c.id == id).expect("lebt")
}

#[test]
fn seal_slides_closer_then_throws_snowballs_in_an_arc() {
    let mut w = world(|_| {});
    spawn(&mut w, 30);
    let floor = FLOOR as f32 * 32.0;
    let id = w
        .add_creature(SEAL, Vec2::new(15.0 * 32.0, floor - 15.0))
        .unwrap();
    let start = creature(&w, id).pos.x;
    run(&mut w, idle(), 60);
    let c = creature(&w, id);
    assert!(
        c.pos.x > start + 100.0,
        "rutscht heran: {} → {}",
        start,
        c.pos.x
    );
    let before = w.character(0).unwrap().health;
    let ev = run(&mut w, idle(), 200);
    assert_eq!(creature(&w, id).mode, seal::THROW, "wirft aus der Nähe");
    assert!(
        (standing(30).x - creature(&w, id).pos.x).abs() <= 300.0,
        "bleibt in Wurfweite stehen"
    );
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::CreatureFire { id: i, .. } if *i == id))
    );
    assert!(w.character(0).unwrap().health < before, "Schneeball trifft");
}

#[test]
fn bat_sleeps_dives_at_elora_and_flies_home() {
    let mut w = world(|t| (0..W).for_each(|x| t[16 * W + x] = Tile::Solid));
    spawn(&mut w, 50);
    let home = Vec2::new(20.0 * 32.0 + 16.0, 17.0 * 32.0 + 18.0);
    let id = w.add_creature(BAT, home).unwrap();
    run(&mut w, idle(), 80);
    assert_eq!(
        creature(&w, id).mode,
        bat::HANG,
        "schläft, Elora ist weit weg"
    );
    assert!(creature(&w, id).pos.distance(home) < 0.5);
    // Elora below
    w.spawn_character(0, standing(20));
    w.character_mut(0).unwrap().invulnerable_until = 0;
    let before = w.character(0).unwrap().health;
    let ev = run(&mut w, idle(), 70);
    assert!(ev.iter().any(|e| matches!(
        e,
        Event::CreatureAct {
            act: CreatureAct::Dive,
            ..
        }
    )));
    assert!(
        w.character(0).unwrap().health < before,
        "trifft im Sturzflug"
    );
    // Elora walks away, the bat flies home
    w.spawn_character(0, standing(55));
    run(&mut w, idle(), 200);
    let c = creature(&w, id);
    assert_eq!(c.mode, bat::HANG, "wieder an der Decke");
    assert!(c.pos.distance(home) < 0.5);
}

#[test]
fn ghost_floats_through_walls_freezes_elora_and_backs_off() {
    // thick wall between ghost and Elora
    let mut w = world(|t| {
        for y in 10..FLOOR {
            for x in 30..34 {
                t[y * W + x] = Tile::Solid;
            }
        }
    });
    spawn(&mut w, 40);
    let id = w
        .add_creature(GHOST, standing(27) - Vec2::new(0.0, 12.0))
        .unwrap();
    let mut frozen_at = None;
    for t in 0..600 {
        w.step(&[idle()]);
        if w.character(0).unwrap().core.frozen > 0 {
            frozen_at = Some(t);
            break;
        }
    }
    assert!(
        frozen_at.is_some(),
        "kommt durch die Wand und lässt Elora erstarren"
    );
    assert_eq!(creature(&w, id).mode, ghost::FLEE, "weicht zurück");
    // frozen: walking has no effect (only the hit's knockback pushes)
    let x = w.character(0).unwrap().core.pos.x;
    let right = PlayerInput {
        direction: 1,
        ..idle()
    };
    run(&mut w, right, 20);
    let frozen = (w.character(0).unwrap().core.pos.x - x).abs();
    // after 0.6 s it walks again
    run(&mut w, idle(), 30);
    let x = w.character(0).unwrap().core.pos.x;
    run(&mut w, right, 20);
    let free = w.character(0).unwrap().core.pos.x - x;
    assert!(
        free > 80.0 && frozen < free / 2.0,
        "erstarrt {frozen}, frei {free}"
    );
}
