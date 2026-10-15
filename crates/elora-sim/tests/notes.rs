//! Note stones and star bridges (R2-M2.5, M2.5.3, E-364).

mod support;

use elora_sim::{Abilities, Event, NoteStone, PHYS_SIZE, StarBridge, Tuning, Vec2, World};
use support::{FLOOR, core, input, run, spawn, standing, test_world as world};

/// Top of the bridge: row 20, eight rows above the floor.
const BRIDGE_TOP: f32 = 640.0;

/// Note stone (e) on the floor around column 10, bridge from column 20 to 30 in the air.
fn with_note() -> World {
    let mut w = world(|_| {});
    let stone = NoteStone::new(
        Vec2::new(9.0 * 32.0, (FLOOR as f32 - 2.0) * 32.0),
        Vec2::new(64.0, 64.0),
        4,
        vec![0],
    );
    let bridge = StarBridge::new(Vec2::new(20.0 * 32.0, BRIDGE_TOP), Vec2::new(320.0, 16.0));
    w.set_notes(vec![stone], vec![bridge]);
    w
}

fn notes(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Note { note, .. } => Some(*note),
            _ => None,
        })
        .collect()
}

#[test]
fn touching_a_note_stone_plays_once_and_lights_its_bridge() {
    let mut w = with_note();
    spawn(&mut w, standing(5, FLOOR), Abilities::NONE);
    assert!(!w.bridges[0].active(w.tick));
    // walk right into the stone and stay there
    let mut events = Vec::new();
    for _ in 0..40 {
        events.extend(run(&mut w, input(1, false, false), 1));
        if w.note_stones[0].touched {
            break;
        }
    }
    events.extend(run(&mut w, input(0, false, false), 30));
    assert_eq!(notes(&events), vec![4], "one note while standing at it");
    assert!(w.note_stones[0].lit(w.tick));
    assert!(w.bridges[0].active(w.tick));
    assert!(w.is_lit(w.note_stones[0].centre()));
    assert!(
        w.is_lit(Vec2::new(25.0 * 32.0, BRIDGE_TOP - 40.0)),
        "the bridge glows"
    );
}

#[test]
fn lit_bridge_carries_until_the_light_fades() {
    let mut w = with_note();
    // Elora drops onto the bridge right after the stone was lit
    w.note_stones[0].lit_until = 1000;
    w.bridges[0].until = 1000;
    spawn(
        &mut w,
        Vec2::new(25.0 * 32.0, BRIDGE_TOP - 80.0),
        Abilities::NONE,
    );
    run(&mut w, input(0, false, false), 40);
    let feet = core(&w).pos.y + PHYS_SIZE / 2.0;
    assert!(
        (feet - BRIDGE_TOP).abs() <= 1.0,
        "stands on the bridge: {feet}"
    );
    // the light fades: Elora falls
    w.bridges[0].until = w.tick + 1;
    run(&mut w, input(0, false, false), 60);
    assert!(
        core(&w).pos.y > BRIDGE_TOP + 100.0,
        "fell: {:?}",
        core(&w).pos
    );
}

#[test]
fn unlit_bridge_does_not_carry() {
    let mut w = with_note();
    spawn(
        &mut w,
        Vec2::new(25.0 * 32.0, BRIDGE_TOP - 80.0),
        Abilities::NONE,
    );
    run(&mut w, input(0, false, false), 60);
    assert!(core(&w).pos.y > BRIDGE_TOP + 100.0);
}

#[test]
fn the_light_lasts_a_50() {
    let mut w = with_note();
    spawn(&mut w, standing(10, FLOOR), Abilities::NONE);
    run(&mut w, input(0, false, false), 2);
    let lit_at = w.tick;
    assert!(w.bridges[0].active(lit_at));
    let ticks = u64::from(Tuning::default().note_light_time) * 50 / 1000;
    assert!(w.bridges[0].until <= lit_at + ticks);
    assert!(w.bridges[0].until + 2 >= lit_at + ticks);
}
