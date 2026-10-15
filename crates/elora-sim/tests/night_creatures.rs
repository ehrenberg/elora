//! Creatures of the night (R2-M2.5, E-365): shadow moth, stone goblin, echo.

mod support;

use elora_sim::creature::{echo, goblin, moth};
use elora_sim::{Abilities, Behavior, Event, PlayerInput, Tile, Vec2, World};
use support::{FLOOR, adventure_arena, creature, idle, input, kind, run, spawn_vulnerable};

const W: usize = 70;
const H: usize = 30;

const MOTH: usize = 0;
const GOBLIN: usize = 1;
const ECHO: usize = 2;

fn world(edit: impl FnOnce(&mut Vec<Tile>)) -> World {
    let kinds = vec![
        kind(
            "shadow_moth",
            Behavior::Moth {
                sight: 360.0,
                speed: 3.2,
                flutter: 36.0,
            },
        )
        .size(24.0, 20.0)
        .touch_damage(1)
        .build(),
        kind(
            "stone_goblin",
            Behavior::Goblin {
                sight: 220.0,
                wake_ms: 600,
                speed: 5.5,
                daze_ms: 2000,
            },
        )
        .size(40.0, 30.0)
        .health(3)
        .touch_damage(2)
        .build(),
        kind(
            "echo",
            Behavior::Echo {
                sight: 320.0,
                delay_ms: 1500,
                fade_ms: 700,
            },
        )
        .size(28.0, 28.0)
        .touch_damage(1)
        .build(),
    ];
    adventure_arena(W, H, kinds, edit)
}

fn standing(tx: usize) -> Vec2 {
    support::standing(tx, FLOOR)
}

fn elora(w: &World) -> Vec2 {
    w.character(0).unwrap().core.pos
}

fn floor_y(h: f32) -> f32 {
    FLOOR as f32 * 32.0 - h / 2.0 - 1.0
}

// ---------------------------------------------------------------- Shadow moth

#[test]
fn moth_attacks_elora_in_the_dark() {
    let mut w = world(|_| {});
    spawn_vulnerable(&mut w, standing(30), Abilities::NONE);
    let id = w
        .add_creature(MOTH, elora(&w) + Vec2::new(-200.0, -120.0))
        .unwrap();
    let before = w.character(0).unwrap().health;
    run(&mut w, idle(), 150);
    assert_eq!(creature(&w, id).mode, moth::ATTACK);
    assert!(w.character(0).unwrap().health < before, "the moth bites");
}

#[test]
fn moth_does_not_enter_the_light_around_elora() {
    let mut w = world(|_| {});
    spawn_vulnerable(&mut w, standing(30), Abilities::NONE);
    // a lantern next to Elora
    w.lights = vec![(elora(&w), 150.0)];
    let id = w
        .add_creature(MOTH, elora(&w) + Vec2::new(-260.0, -120.0))
        .unwrap();
    let before = w.character(0).unwrap().health;
    let mut closest = f32::MAX;
    for _ in 0..300 {
        run(&mut w, idle(), 1);
        closest = closest.min(creature(&w, id).pos.distance(elora(&w)));
    }
    assert!(closest > 120.0, "stays out of the light: {closest}");
    assert_eq!(w.character(0).unwrap().health, before);
    assert_ne!(creature(&w, id).mode, moth::ATTACK, "no prey in the light");
}

#[test]
fn moth_flees_from_light() {
    let mut w = world(|_| {});
    spawn_vulnerable(&mut w, standing(5), Abilities::NONE);
    let lamp = Vec2::new(40.0 * 32.0, 15.0 * 32.0);
    w.lights = vec![(lamp, 150.0)];
    let id = w.add_creature(MOTH, lamp + Vec2::new(40.0, 0.0)).unwrap();
    run(&mut w, idle(), 2);
    assert_eq!(creature(&w, id).mode, moth::FLEE);
    run(&mut w, idle(), 80);
    assert!(creature(&w, id).pos.distance(lamp) > 150.0);
}

// ---------------------------------------------------------------- Stone goblin

#[test]
fn goblin_is_a_rock_until_elora_comes_then_rolls_at_her() {
    let mut w = world(|_| {});
    spawn_vulnerable(&mut w, standing(60), Abilities::NONE);
    let id = w
        .add_creature(GOBLIN, Vec2::new(20.0 * 32.0, floor_y(30.0)))
        .unwrap();
    run(&mut w, idle(), 60);
    assert_eq!(creature(&w, id).mode, goblin::ROCK, "Elora is far away");
    // a rock is invulnerable
    w.hurt_creature(id, 3);
    assert_eq!(creature(&w, id).health, 3);
    // Elora comes close
    w.spawn_character(0, standing(26));
    w.character_mut(0).unwrap().invulnerable_until = 0;
    run(&mut w, idle(), 5);
    assert_eq!(creature(&w, id).mode, goblin::WAKE);
    let start = creature(&w, id).pos.x;
    let before = w.character(0).unwrap().health;
    run(&mut w, idle(), 60);
    assert!(creature(&w, id).pos.x > start + 60.0, "rolls towards Elora");
    run(&mut w, idle(), 40);
    assert!(w.character(0).unwrap().health < before, "and runs her over");
}

#[test]
fn a_hit_stops_the_rolling_goblin_which_becomes_a_rock_again() {
    let mut w = world(|_| {});
    spawn_vulnerable(&mut w, standing(40), Abilities::NONE);
    let id = w
        .add_creature(GOBLIN, Vec2::new(36.0 * 32.0, floor_y(30.0)))
        .unwrap();
    run(&mut w, idle(), 40);
    assert_eq!(creature(&w, id).mode, goblin::ROLL);
    w.hurt_creature(id, 1);
    run(&mut w, idle(), 1);
    let c = creature(&w, id);
    assert_eq!(c.mode, goblin::DAZED);
    assert!(c.stun > 0 && c.health == 2, "dazed and hurt");
    // a dazed goblin is vulnerable, then it is a rock again
    w.spawn_character(0, standing(65));
    run(&mut w, idle(), 120);
    assert_eq!(creature(&w, id).mode, goblin::ROCK);
}

#[test]
fn rolling_goblin_is_dazed_by_a_wall() {
    // wall between goblin and Elora
    let mut w = world(|t| {
        for y in 20..FLOOR {
            t[y * W + 30] = Tile::Solid;
        }
    });
    spawn_vulnerable(&mut w, standing(31), Abilities::NONE);
    let id = w
        .add_creature(GOBLIN, Vec2::new(25.0 * 32.0, floor_y(30.0)))
        .unwrap();
    let ev = run(&mut w, idle(), 90);
    assert!(
        ev.iter().any(|e| matches!(
            e,
            Event::CreatureAct {
                act: elora_sim::CreatureAct::Land,
                ..
            }
        )),
        "bumps into the wall"
    );
    assert_eq!(creature(&w, id).mode, goblin::DAZED);
}

// ---------------------------------------------------------------- Echo

#[test]
fn echo_steps_out_of_its_mirror_and_walks_elora_path_behind_her() {
    let mut w = world(|_| {});
    spawn_vulnerable(&mut w, standing(18), Abilities::NONE);
    let mirror = Vec2::new(10.0 * 32.0, floor_y(28.0));
    let id = w.add_creature(ECHO, mirror).unwrap();
    run(&mut w, idle(), 2);
    assert_eq!(creature(&w, id).mode, echo::FOLLOW, "sees Elora");
    // Elora walks right; remember where she was
    let mut path = Vec::new();
    for _ in 0..150 {
        run(&mut w, input(1, false, false), 1);
        path.push(elora(&w));
    }
    let c = creature(&w, id);
    // 1.5 s = 75 ticks behind (the trail starts one tick earlier)
    let back = path[path.len() - 76];
    assert!(c.pos.distance(back) < 12.0, "{:?} vs {:?}", c.pos, back);
    assert!(c.pos.x < elora(&w).x - 100.0, "behind her");
    // weapons pass through
    w.hurt_creature(id, 5);
    assert!(w.creatures.iter().any(|c| c.id == id));
}

#[test]
fn echo_waits_in_the_mirror_and_fades_in_the_light() {
    let mut w = world(|_| {});
    spawn_vulnerable(&mut w, standing(60), Abilities::NONE);
    let mirror = Vec2::new(10.0 * 32.0, floor_y(28.0));
    let id = w.add_creature(ECHO, mirror).unwrap();
    run(&mut w, idle(), 50);
    assert_eq!(creature(&w, id).mode, echo::MIRROR, "Elora is far away");
    // Elora comes close and stands in a lantern's light: the echo reaches the light and fades
    w.spawn_character(0, standing(16));
    w.lights = vec![(standing(16), 140.0)];
    let mut ev = Vec::new();
    for _ in 0..200 {
        ev.extend(run(&mut w, PlayerInput::default(), 1));
    }
    assert!(!w.creatures.iter().any(|c| c.id == id), "gone");
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::CreatureDeath { id: i, killer: Some(0), .. } if *i == id)),
        "counts as defeated by Elora"
    );
}
