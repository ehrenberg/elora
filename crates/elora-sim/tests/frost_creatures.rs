//! Frostspitzen enemies (R2-M2.4): snowball seal, ice-spike bat, frost ghost.

mod support;

use elora_sim::creature::{bat, ghost, seal};
use elora_sim::{Abilities, Behavior, CreatureAct, Event, PlayerInput, Tile, Vec2, World};
use support::{FLOOR, adventure_arena, creature, idle, kind, run, spawn_vulnerable};

const W: usize = 70;
const H: usize = 30;

const SEAL: usize = 0;
const BAT: usize = 1;
const GHOST: usize = 2;

fn world(edit: impl FnOnce(&mut Vec<Tile>)) -> World {
    let kind = |name: &str, behavior| kind(name, behavior).health(3).touch_damage(1);
    let kinds = vec![
        kind(
            "snowball_seal",
            Behavior::Seal {
                sight: 520.0,
                speed: 3.2,
                range: 300.0,
                interval_ms: 1500,
                shot_speed: 5.5,
                shot_damage: 1,
            },
        )
        .size(48.0, 28.0)
        .build(),
        kind(
            "bat",
            Behavior::Bat {
                sight: 90.0,
                reach: 380.0,
                speed: 6.5,
            },
        )
        .size(32.0, 36.0)
        .build(),
        kind(
            "frost_ghost",
            Behavior::Ghost {
                sight: 460.0,
                speed: 1.6,
                flee_ms: 1800,
            },
        )
        .size(40.0, 52.0)
        .freeze_ms(600)
        .build(),
    ];
    adventure_arena(W, H, kinds, edit)
}

fn standing(tx: usize) -> Vec2 {
    support::standing(tx, FLOOR)
}

fn spawn(w: &mut World, tx: usize) {
    spawn_vulnerable(w, standing(tx), Abilities::NONE);
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
        "slides closer: {} → {}",
        start,
        c.pos.x
    );
    let before = w.character(0).unwrap().health;
    let ev = run(&mut w, idle(), 200);
    assert_eq!(
        creature(&w, id).mode,
        seal::THROW,
        "throws from close range"
    );
    assert!(
        (standing(30).x - creature(&w, id).pos.x).abs() <= 300.0,
        "stops at throwing range"
    );
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::CreatureFire { id: i, .. } if *i == id))
    );
    assert!(w.character(0).unwrap().health < before, "snowball hits");
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
        "sleeps, Elora is far away"
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
    assert!(w.character(0).unwrap().health < before, "hits in a dive");
    // Elora walks away, the bat flies home
    w.spawn_character(0, standing(55));
    run(&mut w, idle(), 200);
    let c = creature(&w, id);
    assert_eq!(c.mode, bat::HANG, "back on the ceiling");
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
        "comes through the wall and freezes Elora"
    );
    assert_eq!(creature(&w, id).mode, ghost::FLEE, "backs off");
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
        "frozen {frozen}, free {free}"
    );
}
