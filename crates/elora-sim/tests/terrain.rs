//! Frostspitzen terrain (R2-M2.4, E-343): thin ice, ice water, icicles, snow chunks.

use elora_sim::creature::icicle;
use elora_sim::{
    Abilities, Ability, Behavior, Collision, CreatureKind, Event, PlayerInput, Tile, Tuning, Vec2,
    World,
};

const W: usize = 60;
const H: usize = 30;
const FLOOR: usize = 28;

fn kind(name: &str, size: [f32; 2], behavior: Behavior) -> CreatureKind {
    CreatureKind {
        name: name.into(),
        size,
        health: 1,
        touch_damage: 2,
        small: false,
        boss: false,
        xp: 0,
        loot: Vec::new(),
        daze_ms: 0,
        freeze_ms: 0,
        armor: false,
        behavior,
    }
}

/// Adventure arena with floor in row 28 and walls; `edit` sets further tiles.
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
            "eiszapfen",
            [20.0, 48.0],
            Behavior::Icicle {
                sight: 48.0,
                reach: 420.0,
                warn_ms: 450,
            },
        ),
        kind(
            "schneebrocken",
            [36.0, 36.0],
            Behavior::Roller {
                speed: 5.0,
                life_ms: 6000,
            },
        ),
    ];
    w
}

fn set(tiles: &mut [Tile], x: usize, y: usize, t: Tile) {
    tiles[y * W + x] = t;
}

/// Snow chunk (36 high) on the floor in column `tx`.
fn rock_at(tx: usize) -> Vec2 {
    #[allow(clippy::cast_precision_loss)]
    Vec2::new(tx as f32 * 32.0 + 16.0, FLOOR as f32 * 32.0 - 19.0)
}

/// Centre above tile column `tx`, standing on row `row`.
fn standing(tx: usize, row: usize) -> Vec2 {
    #[allow(clippy::cast_precision_loss)]
    Vec2::new(tx as f32 * 32.0 + 16.0, row as f32 * 32.0 - 15.0)
}

fn spawn(w: &mut World, at: Vec2, abilities: Abilities) {
    let i = w.join();
    w.set_abilities(i, abilities);
    w.spawn_character(i, at);
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

fn health(w: &World) -> i32 {
    w.character(0).map_or(0, |c| c.health)
}

#[test]
fn thin_ice_cracks_breaks_and_grows_back() {
    // walkway of thin ice in row 20 over air
    let mut w = world(|t| (20..26).for_each(|x| set(t, x, 20, Tile::ThinIce)));
    spawn(&mut w, standing(22, 20), Abilities::NONE);
    let ev = run(&mut w, idle(), 10);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::IceCrack { broken: false, .. })),
        "Risse unter Elora"
    );
    assert_eq!(w.collision.tile(22, 20), Tile::ThinIce, "trägt noch");
    let ev = run(&mut w, idle(), 30);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::IceCrack { broken: true, .. })),
        "bricht nach A-36"
    );
    assert_eq!(w.collision.tile(22, 20), Tile::Air);
    assert_eq!(w.collision.tile(25, 20), Tile::ThinIce, "nur unter Elora");
    // Elora falls through, the ice grows back after A-37
    run(&mut w, idle(), 60);
    assert!(w.character(0).unwrap().core.pos.y > 27.0 * 32.0);
    let regrow = elora_sim::tuning::ms_to_ticks(Tuning::default().thin_ice_regrow);
    run(&mut w, idle(), regrow);
    assert_eq!(w.collision.tile(22, 20), Tile::ThinIce, "wieder zu");
    assert!(!Tile::ThinIce.is_hookable() && Tile::ThinIce.is_solid());
}

#[test]
fn thin_ice_does_not_grow_back_into_elora() {
    let mut w = world(|t| set(t, 22, 27, Tile::ThinIce));
    spawn(&mut w, standing(22, 28), Abilities::NONE);
    w.collision.set_tile(22, 27, Tile::Air);
    w.temp_tiles.push((22, 27, Tile::ThinIce, 5));
    run(&mut w, idle(), 20);
    assert_eq!(w.collision.tile(22, 27), Tile::Air, "Elora steht darin");
    // out of the way: now it closes up
    let right = PlayerInput {
        direction: 1,
        ..idle()
    };
    run(&mut w, right, 30);
    run(&mut w, idle(), 5);
    assert_eq!(w.collision.tile(22, 27), Tile::ThinIce);
}

#[test]
fn stomp_breaks_thin_ice_at_once() {
    let mut w = world(|t| (20..26).for_each(|x| set(t, x, 20, Tile::ThinIce)));
    // without a stomp the ice holds here forever
    w.tuning.thin_ice_break = 1_000_000;
    spawn(
        &mut w,
        standing(22, 14),
        Abilities::NONE.with(Ability::Stomp),
    );
    let down = PlayerInput {
        down: true,
        ..idle()
    };
    let ev = run(&mut w, down, 60);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::IceCrack { broken: true, .. })),
        "Stampfen bricht das Eis sofort"
    );
    assert_eq!(w.collision.tile(22, 20), Tile::Air);
}

#[test]
fn ice_water_hurts_and_returns_elora_to_the_edge() {
    // pool of ice water in rows 26–27, Elora walks in
    let mut w = world(|t| {
        for x in 30..36 {
            set(t, x, 28, Tile::Air);
            set(t, x, 27, Tile::IceWater);
            set(t, x, 28, Tile::IceWater);
        }
    });
    spawn(&mut w, standing(25, 28), Abilities::NONE);
    run(&mut w, idle(), 20);
    let before = health(&w);
    let right = PlayerInput {
        direction: 1,
        ..idle()
    };
    let mut back = false;
    for _ in 0..120 {
        w.step(&[right]);
        if let Some(c) = w.character(0)
            && c.health < before
        {
            back = c.core.pos.x < 30.0 * 32.0;
            break;
        }
    }
    assert_eq!(health(&w), before - Tuning::default().ice_water_damage);
    assert!(back, "zurück auf sicherem Boden");
}

#[test]
fn icicle_shakes_falls_hurts_and_shatters() {
    let mut w = world(|t| (0..W).for_each(|x| set(t, x, 18, Tile::Solid)));
    spawn(&mut w, standing(40, 28), Abilities::NONE);
    // icicle hangs under the ceiling (row 18) above column 20 – Elora is far away
    let id = w
        .add_creature(0, Vec2::new(20.0 * 32.0 + 16.0, 19.0 * 32.0 + 24.0))
        .unwrap();
    run(&mut w, idle(), 50);
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.mode, icicle::HANG, "hängt, solange niemand darunter ist");
    // Elora stands beneath it
    let before = health(&w);
    w.spawn_character(0, standing(20, 28));
    w.character_mut(0).unwrap().invulnerable_until = 0;
    run(&mut w, idle(), 3);
    assert_eq!(
        w.creatures.iter().find(|c| c.id == id).unwrap().mode,
        icicle::SHAKE,
        "zittert"
    );
    let ev = run(&mut w, idle(), 80);
    assert!(health(&w) < before, "trifft Elora");
    assert!(w.creatures.iter().all(|c| c.id != id), "zerschellt");
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::CreatureDeath { id: i, killer: None, .. } if *i == id))
    );
}

#[test]
fn icicle_can_be_dodged() {
    let mut w = world(|t| (0..W).for_each(|x| set(t, x, 18, Tile::Solid)));
    spawn(&mut w, standing(17, 28), Abilities::NONE);
    let id = w
        .add_creature(0, Vec2::new(20.0 * 32.0 + 16.0, 19.0 * 32.0 + 24.0))
        .unwrap();
    run(&mut w, idle(), 10);
    let before = health(&w);
    // walk through underneath
    let right = PlayerInput {
        direction: 1,
        ..idle()
    };
    run(&mut w, right, 80);
    run(&mut w, idle(), 60);
    assert_eq!(health(&w), before, "im Lauf entkommen");
    assert!(
        w.creatures.iter().all(|c| c.id != id),
        "am Boden zerschellt"
    );
}

#[test]
fn snow_rock_rolls_downhill_and_bursts_at_a_wall() {
    // wall at column 50
    let mut w = world(|t| (20..28).for_each(|y| set(t, 50, y, Tile::Solid)));
    spawn(&mut w, standing(5, 28), Abilities::NONE);
    let id = w.add_creature(1, rock_at(30)).unwrap();
    w.creatures.iter_mut().find(|c| c.id == id).unwrap().facing = 1;
    run(&mut w, idle(), 20);
    let x = w.creatures.iter().find(|c| c.id == id).unwrap().pos.x;
    assert!(x > 30.0 * 32.0 + 60.0, "rollt nach rechts: {x}");
    let ev = run(&mut w, idle(), 200);
    assert!(
        w.creatures.iter().all(|c| c.id != id),
        "zerplatzt an der Wand"
    );
    assert!(ev.iter().any(|e| matches!(e, Event::CreatureDeath { .. })));
    assert!(health(&w) > 0);
}

#[test]
fn snow_rock_hurts_elora_and_bursts() {
    let mut w = world(|_| {});
    spawn(&mut w, standing(40, 28), Abilities::NONE);
    let before = health(&w);
    let id = w.add_creature(1, rock_at(30)).unwrap();
    w.creatures.iter_mut().find(|c| c.id == id).unwrap().facing = 1;
    run(&mut w, idle(), 60);
    assert!(health(&w) < before, "trifft");
    assert!(w.creatures.iter().all(|c| c.id != id), "zerplatzt an Elora");
}
