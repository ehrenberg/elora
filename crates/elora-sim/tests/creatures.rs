//! Kreaturen (R2-M1, A1.2, E-232 bis E-239): Verhalten, Treffer, Berührung, Beute.

use elora_sim::character::events;
use elora_sim::{
    Abilities, Ability, Behavior, Collision, CreatureKind, Event, HookState, LootEntry,
    PlayerInput, Tile, Tuning, Vec2, Weapon, World,
};

const W: usize = 60;
const H: usize = 30;
const FLOOR: usize = 28;

fn kinds() -> Vec<CreatureKind> {
    let kind = |name: &str, behavior| CreatureKind {
        name: name.into(),
        size: [40.0, 26.0],
        health: 6,
        touch_damage: 2,
        small: true,
        boss: false,
        xp: 5,
        loot: vec![LootEntry {
            item: "glanztropfen".into(),
            min: 3,
            max: 3,
            chance: 1.0,
        }],
        behavior,
    };
    vec![
        kind(
            "walker",
            Behavior::Walker {
                speed: 1.5,
                turn_at_edges: true,
            },
        ),
        kind(
            "turret",
            Behavior::Turret {
                interval_ms: 500,
                range: 400.0,
                shot_speed: 6.0,
                shot_damage: 2,
            },
        ),
        kind(
            "hopper",
            Behavior::Hopper {
                wait_ms: 200,
                jump_x: 6.0,
                jump_y: 10.0,
                sight: 300.0,
            },
        ),
        kind(
            "flyer",
            Behavior::Flyer {
                speed: 3.0,
                sight: 300.0,
            },
        ),
    ]
}

/// Arena mit Boden in Zeile 28, Abenteuer-Regeln an.
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
    w.creature_kinds = kinds();
    w.adventure = true;
    w
}

fn set(tiles: &mut [Tile], x: usize, y: usize, t: Tile) {
    tiles[y * W + x] = t;
}

/// Mitte über dem Boden von Spalte `tx` für eine Box der Höhe `h`.
fn on_floor(tx: usize, h: f32) -> Vec2 {
    #[allow(clippy::cast_precision_loss)]
    Vec2::new(tx as f32 * 32.0 + 16.0, FLOOR as f32 * 32.0 - h / 2.0 - 1.0)
}

fn run(w: &mut World, i: PlayerInput, ticks: u32) -> Vec<Event> {
    let mut all = Vec::new();
    for _ in 0..ticks {
        w.step(&[i]);
        all.extend(w.events.iter().cloned());
    }
    all
}

fn elora(w: &mut World, tx: usize) -> usize {
    let i = w.join();
    w.spawn_character(i, on_floor(tx, 28.0));
    i
}

fn health(w: &World) -> i32 {
    w.character(0).map_or(0, |c| c.health)
}

fn aim(x: i32, y: i32) -> PlayerInput {
    PlayerInput {
        target_x: x,
        target_y: y,
        ..PlayerInput::default()
    }
}

/// Feuer-Zähler: ungerade = gedrückt.
fn fire(x: i32, y: i32, counter: u8) -> PlayerInput {
    PlayerInput {
        fire: counter,
        ..aim(x, y)
    }
}

#[test]
fn walker_turns_at_walls_and_edges() {
    // Podest Zeilen 20, Spalten 20..30: der Läufer bleibt oben
    let mut w = world(|t| (20..30).for_each(|x| set(t, x, 20, Tile::Solid)));
    let start = Vec2::new(25.0 * 32.0, 20.0 * 32.0 - 14.0);
    w.add_creature(0, start).unwrap();
    let mut xs = Vec::new();
    for _ in 0..600 {
        w.step(&[]);
        xs.push(w.creatures[0].pos.x);
    }
    let (min, max) = xs
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    assert!(
        min > 20.0 * 32.0 && max < 30.0 * 32.0,
        "bleibt auf dem Podest: {min}..{max}"
    );
    assert!(max - min > 100.0, "läuft hin und her");
    assert!(w.creatures[0].pos.y < 20.0 * 32.0);
}

#[test]
fn touch_hurts_once_then_protects_with_knockback() {
    let mut w = world(|_| {});
    elora(&mut w, 30);
    w.add_creature(0, on_floor(31, 26.0)).unwrap();
    w.step(&[]);
    let after_hit = health(&w);
    assert_eq!(after_hit, 8, "Berührung: 2 Schaden");
    let inv = w.character(0).unwrap().invulnerable_until;
    assert!(inv > w.tick, "danach geschützt");
    run(&mut w, PlayerInput::default(), 20);
    assert_eq!(health(&w), 8, "kein zweiter Treffer im Schutzfenster");
}

#[test]
fn hammer_kills_creature_and_loot_flies_to_elora() {
    let mut w = world(|_| {});
    w.tuning.hit_invulnerable = 100_000; // Berührung soll den Test nicht stören
    elora(&mut w, 30);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    // stehender Gegner, damit jeder Schlag trifft
    let id = w.add_creature(1, on_floor(31, 26.0)).unwrap();
    let mut events = Vec::new();
    let mut counter = 0u8;
    for _ in 0..12 {
        counter += 1;
        events.extend(run(&mut w, fire(100, 0, counter), 1));
        counter += 1;
        events.extend(run(&mut w, fire(100, 0, counter), 12));
    }
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CreatureHit { id: i, from: Some(0), .. } if *i == id))
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::CreatureDeath {
                killer: Some(0),
                ..
            }
        )),
        "besiegt"
    );
    assert!(w.creatures.is_empty());
    let more = run(&mut w, PlayerInput::default(), 120);
    let collected: u32 = events
        .iter()
        .chain(&more)
        .filter_map(|e| match e {
            Event::LootCollect { item, count, .. } if item == "glanztropfen" => Some(*count),
            _ => None,
        })
        .sum();
    assert_eq!(collected, 3, "Beute eingesammelt");
    assert!(w.loot.is_empty());
}

#[test]
fn no_self_damage_in_adventure_but_knockback() {
    let mut w = world(|_| {});
    elora(&mut w, 30);
    w.character_mut(0)
        .unwrap()
        .arsenal
        .give(Weapon::Grenade, 10, 10);
    w.character_mut(0).unwrap().arsenal.active = Weapon::Grenade;
    let before = w.character(0).unwrap().core.vel;
    run(&mut w, fire(0, 100, 1), 1);
    run(&mut w, fire(0, 100, 2), 5);
    assert_eq!(health(&w), 10);
    assert_ne!(w.character(0).unwrap().core.vel, before, "Rückstoß wirkt");
}

#[test]
fn turret_shoots_at_elora_in_sight() {
    let mut w = world(|_| {});
    elora(&mut w, 20);
    w.add_creature(1, on_floor(28, 26.0)).unwrap();
    let events = run(&mut w, PlayerInput::default(), 120);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CreatureFire { .. }))
    );
    assert!(health(&w) < 10, "Geschoss trifft");
    // hinter einer Wand: kein Schuss
    let mut w = world(|t| (20..FLOOR).for_each(|y| set(t, 24, y, Tile::Solid)));
    elora(&mut w, 20);
    w.add_creature(1, on_floor(28, 26.0)).unwrap();
    let events = run(&mut w, PlayerInput::default(), 120);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::CreatureFire { .. }))
    );
}

#[test]
fn hopper_jumps_toward_elora() {
    let mut w = world(|_| {});
    elora(&mut w, 20);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    w.add_creature(2, on_floor(26, 26.0)).unwrap();
    let start = w.creatures[0].pos.x;
    run(&mut w, PlayerInput::default(), 40);
    assert!(
        w.creatures[0].pos.x < start - 20.0,
        "springt nach links zu Elora"
    );
}

#[test]
fn flyer_chases_and_returns() {
    let mut w = world(|_| {});
    let home = Vec2::new(40.0 * 32.0, 22.0 * 32.0);
    w.add_creature(3, home).unwrap();
    run(&mut w, PlayerInput::default(), 50);
    assert!(
        w.creatures[0].pos.distance(home) < 8.0,
        "schwebt am Startpunkt"
    );
    elora(&mut w, 35);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    let d0 = w.creatures[0]
        .pos
        .distance(w.character(0).unwrap().core.pos);
    run(&mut w, PlayerInput::default(), 40);
    let d1 = w.creatures[0]
        .pos
        .distance(w.character(0).unwrap().core.pos);
    assert!(d1 < d0, "verfolgt Elora");
}

fn hook_at(w: &mut World, x: i32, y: i32) {
    let i = PlayerInput {
        hook: true,
        ..aim(x, y)
    };
    for _ in 0..20 {
        w.step(&[i]);
        if w.character(0).unwrap().core.hook_state == HookState::Grabbed {
            return;
        }
    }
    panic!("Hook greift nicht");
}

#[test]
fn hook_grabs_creature_and_pulls_elora() {
    let mut w = world(|_| {});
    elora(&mut w, 20);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    // stehender Gegner (Geschütz außer Reichweite des Schießens: Reichweite 400)
    w.add_creature(1, on_floor(28, 26.0)).unwrap();
    w.creature_kinds[1].behavior = Behavior::Turret {
        interval_ms: 1_000_000,
        range: 0.0,
        shot_speed: 1.0,
        shot_damage: 0,
    };
    hook_at(&mut w, 100, 0);
    let core = &w.character(0).unwrap().core;
    assert!(core.hooked_creature.is_some() && !core.pulling);
    assert!(core.triggered_events & events::HOOK_ATTACH_PLAYER != 0 || core.vel.x >= 0.0);
    let x0 = w.character(0).unwrap().core.pos.x;
    run(
        &mut w,
        PlayerInput {
            hook: true,
            ..aim(100, 0)
        },
        15,
    );
    assert!(
        w.character(0).unwrap().core.pos.x > x0 + 20.0,
        "Elora wird hingezogen"
    );
}

#[test]
fn pull_brings_small_creature_to_elora() {
    let mut w = world(|_| {});
    let i = elora(&mut w, 20);
    w.set_abilities(i, Abilities::NONE.with(Ability::Pull));
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    w.add_creature(2, on_floor(28, 26.0)).unwrap();
    w.creature_kinds[2].behavior = Behavior::Hopper {
        wait_ms: 1_000_000,
        jump_x: 0.0,
        jump_y: 0.0,
        sight: 0.0,
    };
    hook_at(&mut w, 100, 0);
    assert!(w.character(0).unwrap().core.pulling);
    let (ex, cx) = (w.character(0).unwrap().core.pos.x, w.creatures[0].pos.x);
    run(
        &mut w,
        PlayerInput {
            hook: true,
            ..aim(100, 0)
        },
        25,
    );
    assert!(w.creatures[0].pos.x < cx - 40.0, "Gegner kommt näher");
    assert!(
        (w.character(0).unwrap().core.pos.x - ex).abs() < 8.0,
        "Elora bleibt"
    );
}

#[test]
fn stomp_damages_and_stuns() {
    let mut w = world(|_| {});
    let i = elora(&mut w, 30);
    w.set_abilities(i, Abilities::NONE.with(Ability::Stomp));
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    w.add_creature(2, on_floor(31, 26.0)).unwrap();
    w.creature_kinds[2].behavior = Behavior::Hopper {
        wait_ms: 1_000_000,
        jump_x: 0.0,
        jump_y: 0.0,
        sight: 0.0,
    };
    run(&mut w, PlayerInput::default(), 2);
    run(
        &mut w,
        PlayerInput {
            jump: true,
            ..PlayerInput::default()
        },
        1,
    );
    run(&mut w, PlayerInput::default(), 12);
    let ev = run(
        &mut w,
        PlayerInput {
            down: true,
            ..PlayerInput::default()
        },
        30,
    );
    assert!(ev.iter().any(|e| matches!(e, Event::Stomp { .. })));
    let c = &w.creatures[0];
    assert_eq!(c.health, 6 - Tuning::default().stomp_damage);
    assert!(c.stun > 0, "betäubt");
}

#[test]
fn laser_hits_creature() {
    let mut w = world(|_| {});
    elora(&mut w, 20);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    w.character_mut(0)
        .unwrap()
        .arsenal
        .give(Weapon::Laser, 10, 10);
    w.character_mut(0).unwrap().arsenal.active = Weapon::Laser;
    w.add_creature(1, on_floor(28, 26.0)).unwrap();
    let ev = run(&mut w, fire(100, 0, 1), 1);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::CreatureHit { damage: 5, .. }))
    );
}

#[test]
fn without_creatures_multiplayer_is_unchanged() {
    // Ohne Gegnerarten und ohne Abenteuer bleibt alles beim Alten (Golden-Tests prüfen den Rest)
    let mut w = World::new(
        Tuning::default(),
        Collision::new(3, 3, vec![Tile::Solid; 9]),
    );
    assert!(w.creatures.is_empty() && !w.adventure);
    w.step(&[]);
    assert!(w.events.is_empty());
}
