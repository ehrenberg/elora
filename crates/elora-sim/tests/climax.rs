//! Climax “Protect Luma” in the simulation (R2-M2.5, M2.5.5, E-362): star lanterns, grey
//! shadows and the Withered One as a puppet.

mod support;

use elora_sim::creature::shade;
use elora_sim::{Abilities, Behavior, Event, Lantern, Vec2, World};
use support::{FLOOR, adventure_arena, creature, idle, input, kind, run, spawn, standing};

const W: usize = 70;
const H: usize = 30;

const SHADE: usize = 0;
const PUPPET: usize = 1;

/// Floor of the arena (top edge, units).
const GROUND: f32 = FLOOR as f32 * 32.0;

fn world() -> World {
    let kinds = vec![
        kind(
            "grey_shade",
            Behavior::Shade {
                speed: 1.6,
                fade_ms: 600,
            },
        )
        .size(28.0, 28.0)
        .touch_damage(0)
        .build(),
        kind("withered", Behavior::Puppet)
            .size(40.0, 90.0)
            .touch_damage(0)
            .build(),
    ];
    adventure_arena(W, H, kinds, |_| {})
}

/// A lantern standing on the floor at column `tx`.
fn lantern(tx: usize) -> Lantern {
    Lantern::new(
        Vec2::new(tx as f32 * 32.0, GROUND - 64.0),
        Vec2::new(32.0, 64.0),
    )
}

fn lantern_events(events: &[Event]) -> Vec<bool> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Lantern { lit, .. } => Some(*lit),
            _ => None,
        })
        .collect()
}

#[test]
fn touching_a_lantern_lights_it_until_its_time_runs_out() {
    let mut w = world();
    w.set_lanterns(vec![lantern(12)]);
    spawn(&mut w, standing(8, FLOOR), Abilities::NONE);
    assert!(!w.lanterns[0].lit(w.tick));
    let mut events = Vec::new();
    for _ in 0..60 {
        events.extend(run(&mut w, input(1, false, false), 1));
        if w.lanterns[0].lit(w.tick) {
            break;
        }
    }
    assert!(w.lanterns[0].lit(w.tick), "Elora walked into the lantern");
    // walk away again: the light stays for A-51, then goes out without an event
    events.extend(run(&mut w, input(-1, false, false), 40));
    assert_eq!(lantern_events(&events), vec![true], "lit once");
    assert!(w.lanterns[0].lit(w.tick));
    assert!(w.is_lit(w.lanterns[0].centre()));
    let ticks = elora_sim::tuning::ms_to_ticks(w.tuning.lantern_time);
    run(&mut w, idle(), ticks);
    assert!(!w.lanterns[0].lit(w.tick), "out after A-51");
}

#[test]
fn shade_drifts_through_walls_towards_its_goal() {
    let mut w = world();
    let goal = Vec2::new(60.0 * 32.0, GROUND - 40.0);
    let id = w
        .add_creature(SHADE, Vec2::new(5.0 * 32.0, GROUND - 40.0))
        .unwrap();
    w.creatures[0].goal = goal;
    let start = goal.distance(creature(&w, id).pos);
    run(&mut w, idle(), 250);
    let c = creature(&w, id);
    assert!(goal.distance(c.pos) < start - 200.0, "drifts closer");
    assert_eq!(c.mode, shade::DRIFT);
    // faster with a speed factor in percent
    let mut fast = world();
    let f = fast
        .add_creature(SHADE, Vec2::new(5.0 * 32.0, GROUND - 40.0))
        .unwrap();
    fast.creatures[0].goal = goal;
    fast.creatures[0].count = 150;
    run(&mut fast, idle(), 250);
    assert!(goal.distance(creature(&fast, f).pos) < goal.distance(c.pos));
}

#[test]
fn lantern_light_dissolves_a_shade() {
    let mut w = world();
    w.set_lanterns(vec![lantern(30)]);
    w.lanterns[0].lit_until = 10_000;
    let id = w
        .add_creature(SHADE, Vec2::new(20.0 * 32.0, GROUND - 60.0))
        .unwrap();
    w.creatures[0].goal = Vec2::new(40.0 * 32.0, GROUND - 60.0);
    let events = run(&mut w, idle(), 400);
    assert!(w.creatures.iter().all(|c| c.id != id), "dissolved");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CreatureDeath { id: d, killer: None, .. } if *d == id))
    );
}

#[test]
fn shade_reaching_a_lit_lantern_puts_it_out() {
    let mut w = world();
    w.set_lanterns(vec![lantern(30)]);
    w.lanterns[0].lit_until = 10_000;
    let at = w.lanterns[0].centre();
    w.add_creature(SHADE, at + Vec2::new(4.0, 0.0)).unwrap();
    w.creatures[0].goal = at;
    let events = run(&mut w, idle(), 2);
    assert_eq!(lantern_events(&events), vec![false]);
    assert!(!w.lanterns[0].lit(w.tick));
    assert!(w.creatures.is_empty(), "the shade is used up");
}

#[test]
fn hammer_pushes_a_shade_back_but_does_not_hurt_it() {
    let mut w = world();
    let me = spawn(&mut w, standing(20, FLOOR), Abilities::NONE);
    let at = standing(21, FLOOR);
    let id = w.add_creature(SHADE, at).unwrap();
    w.creatures[0].goal = standing(5, FLOOR);
    w.character_mut(me).unwrap().arsenal.set_hammer(true);
    let before = creature(&w, id).pos.x;
    run(&mut w, support::fire(1, 0, 1), 1);
    run(&mut w, idle(), 10);
    let c = creature(&w, id);
    assert_eq!(c.health, 1, "unhurt");
    assert!(c.pos.x > before, "pushed away from Elora");
}

#[test]
fn puppet_stays_where_it_is_put_and_harms_nobody() {
    let mut w = world();
    let me = spawn(&mut w, standing(20, FLOOR), Abilities::NONE);
    let at = Vec2::new(20.0 * 32.0, GROUND - 200.0);
    let id = w.add_creature(PUPPET, at).unwrap();
    run(&mut w, idle(), 100);
    assert_eq!(creature(&w, id).pos, at, "no gravity");
    w.creatures[0].pos = standing(20, FLOOR);
    let health = w.character(me).unwrap().health;
    run(&mut w, idle(), 50);
    assert_eq!(w.character(me).unwrap().health, health);
}
