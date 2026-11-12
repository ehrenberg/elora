//! Floating star islands and updrafts (R2-M2.5, M2.5.2, E-364).

mod support;

use elora_sim::{Abilities, Ability, Behavior, Island, PHYS_SIZE, Tuning, Vec2, World};
use support::{FLOOR, core, input, kind, run, spawn, standing, test_world as world};

/// Top edge of the test islands (row 20, eight rows above the floor).
const TOP: f32 = 640.0;

fn island(x: f32, to_x: f32, sinks: bool) -> Island {
    Island::new(
        Vec2::new(x, TOP),
        Vec2::new(to_x, TOP),
        Vec2::new(160.0, 32.0),
        sinks,
    )
}

/// World with the given islands and Elora dropped onto the first one.
fn on_island(islands: Vec<Island>, abilities: Abilities) -> World {
    let mut w = world(|_| {});
    let first = islands[0].start;
    w.set_islands(islands, Vec::new());
    spawn(&mut w, first + Vec2::new(80.0, -60.0), abilities);
    run(&mut w, input(0, false, false), 40);
    w
}

fn feet(w: &World) -> f32 {
    core(w).pos.y + PHYS_SIZE / 2.0
}

#[test]
fn elora_lands_on_an_island_and_stands() {
    let w = on_island(vec![island(400.0, 400.0, false)], Abilities::NONE);
    assert!((feet(&w) - TOP).abs() <= 1.0, "feet at {}", feet(&w));
    assert!(core(&w).is_grounded(&w.collision));
}

#[test]
fn islands_are_passable_from_below() {
    let mut w = world(|_| {});
    w.set_islands(vec![island(400.0, 400.0, false)], Vec::new());
    spawn(&mut w, standing(15, FLOOR), Abilities::NONE);
    run(&mut w, input(0, false, false), 5);
    // a big jump from the floor goes through the island (row 20 is 256 units higher)
    w.tuning.ground_jump_impulse = 22.0;
    run(&mut w, input(0, true, false), 1);
    let mut highest = f32::MAX;
    for _ in 0..120 {
        run(&mut w, input(0, false, false), 1);
        highest = highest.min(feet(&w));
    }
    assert!(highest < TOP, "jumped through: {highest}");
    assert!((feet(&w) - TOP).abs() <= 1.0, "and landed on top");
}

#[test]
fn drifting_island_carries_elora_along() {
    let mut w = on_island(vec![island(400.0, 800.0, false)], Abilities::NONE);
    let x0 = core(&w).pos.x;
    let island_x0 = w.islands[0].pos.x;
    run(&mut w, input(0, false, false), 150);
    let moved = w.islands[0].pos.x - island_x0;
    assert!(moved > 100.0, "the island drifts: {moved}");
    assert!(
        (core(&w).pos.x - x0 - moved).abs() <= 2.0,
        "Elora rides along: {} vs {moved}",
        core(&w).pos.x - x0
    );
    assert!((feet(&w) - TOP).abs() <= 1.0);
}

#[test]
fn sinking_island_sinks_under_elora_and_rises_again() {
    let mut w = on_island(vec![island(400.0, 400.0, true)], Abilities::NONE);
    run(&mut w, input(0, false, false), 60);
    let sunk = w.islands[0].sunk;
    assert!(sunk > 20.0, "sinks: {sunk}");
    assert!(
        (feet(&w) - w.islands[0].pos.y).abs() <= 1.0,
        "Elora sinks with it"
    );
    // deeper than A-46 never
    run(&mut w, input(0, false, false), 600);
    assert!(w.islands[0].sunk <= Tuning::default().island_sink_depth);
    // Elora leaves: the island rises again
    w.remove(0);
    run(&mut w, input(0, false, false), 800);
    assert!(w.islands[0].sunk.abs() < f32::EPSILON);
    assert!((w.islands[0].pos.y - TOP).abs() < f32::EPSILON);
}

#[test]
fn walker_stays_on_its_island() {
    let mut w = world(|_| {});
    w.adventure = true;
    w.creature_kinds = vec![
        kind(
            "walker",
            Behavior::Walker {
                speed: 1.5,
                turn_at_edges: true,
            },
        )
        .build(),
    ];
    w.set_islands(vec![island(400.0, 700.0, false)], Vec::new());
    let id = w.add_creature(0, Vec2::new(480.0, TOP - 30.0)).unwrap();
    run(&mut w, input(0, false, false), 400);
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    let i = &w.islands[0];
    assert!(
        (c.pos.y + 13.0 - i.pos.y).abs() <= 2.5 && c.pos.x >= i.pos.x && c.pos.x <= i.pos.x + 160.0,
        "walker {:?}, island {:?}",
        c.pos,
        i.pos
    );
}

/// Jump, double jump, then hold jump inside an updraft over the whole arena.
fn fall_in_updraft(abilities: Abilities) -> f32 {
    fall_in_updraft_with(abilities, Tuning::default().glide_updraft)
}

/// Like [`fall_in_updraft`], with the factor A-53 of the strengthened star cape.
fn fall_in_updraft_with(abilities: Abilities, glide_updraft: f32) -> f32 {
    let mut w = world(|_| {});
    w.tuning.glide_updraft = glide_updraft;
    w.set_islands(
        Vec::new(),
        vec![(Vec2::new(0.0, 0.0), Vec2::new(60.0 * 32.0, 27.0 * 32.0))],
    );
    spawn(&mut w, standing(30, FLOOR), abilities);
    run(&mut w, input(0, false, false), 5);
    run(&mut w, input(0, true, false), 1);
    run(&mut w, input(0, false, false), 5);
    run(&mut w, input(0, true, false), 1);
    run(&mut w, input(0, false, false), 25);
    run(&mut w, input(0, true, false), 30);
    core(&w).vel.y
}

#[test]
fn updraft_lifts_gliding_elora() {
    let v = fall_in_updraft(Abilities::NONE.with(Ability::Glide));
    assert!(v < -1.0, "rises while gliding: {v}");
    assert!(v >= -Tuning::default().updraft_rise_speed - 0.01);
}

#[test]
fn strengthened_cape_rises_faster_in_updrafts_only_while_gliding() {
    let t = Tuning::default();
    let glide = Abilities::NONE.with(Ability::Glide);
    let (plain, strong) = (fall_in_updraft(glide), fall_in_updraft_with(glide, 1.5));
    assert!(
        (strong + t.updraft_rise_speed * 1.5).abs() < 0.01,
        "one and a half times as fast: {strong}"
    );
    assert!(strong < plain);
    // without gliding nothing changes
    let (a, b) = (
        fall_in_updraft(Abilities::NONE),
        fall_in_updraft_with(Abilities::NONE, 1.5),
    );
    assert!((a - b).abs() < f32::EPSILON, "{a} vs {b}");
}

#[test]
fn updraft_lifts_elora_without_gliding_too_but_slower() {
    let v = fall_in_updraft(Abilities::NONE);
    let t = Tuning::default();
    assert!(v < 0.0, "rises without gliding: {v}");
    assert!(v >= -t.updraft_rise_speed * 0.5 - 0.01, "{v}");
    let glide = fall_in_updraft(Abilities::NONE.with(Ability::Glide));
    assert!(v > glide, "{v} vs gliding {glide}");
}
