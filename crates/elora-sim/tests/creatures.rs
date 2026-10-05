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
        daze_ms: 0,
        armor: false,
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
                lob: false,
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
                hover: 0.0,
                drop_ms: 0,
                drop_damage: 0,
                glow_ms: 0,
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
        lob: false,
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

// ---------------------------------------------------------------- Ausbau (A1.3, A-16 bis A-21)

/// Stehende Gegner, die nicht schießen.
fn still_turret(w: &mut World) {
    w.creature_kinds[1].behavior = Behavior::Turret {
        interval_ms: 1_000_000,
        range: 0.0,
        shot_speed: 1.0,
        shot_damage: 0,
        lob: false,
    };
    w.creature_kinds[1].health = 100;
}

fn hits(ev: &[Event]) -> Vec<u32> {
    ev.iter()
        .filter_map(|e| match e {
            Event::CreatureHit { id, .. } => Some(*id),
            _ => None,
        })
        .collect()
}

#[test]
fn laser_pierce_hits_several_creatures() {
    for (pierce, expected) in [(0, 1), (2, 3)] {
        let mut w = world(|_| {});
        still_turret(&mut w);
        w.tuning.laser_pierce = pierce;
        elora(&mut w, 15);
        w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
        w.character_mut(0)
            .unwrap()
            .arsenal
            .give(Weapon::Laser, 10, 10);
        w.character_mut(0).unwrap().arsenal.active = Weapon::Laser;
        for x in [18, 20, 22, 24] {
            w.add_creature(1, on_floor(x, 26.0)).unwrap();
        }
        let ev = run(&mut w, fire(100, 0, 1), 1);
        let mut ids = hits(&ev);
        ids.dedup();
        assert_eq!(ids.len(), expected, "Durchschlag {pierce}");
    }
}

#[test]
fn hammer_reach_stun_and_shockwave() {
    // Gegner hinter Elora: nur die Schockwelle trifft ihn
    for shockwave in [false, true] {
        let mut w = world(|_| {});
        still_turret(&mut w);
        w.tuning.hammer_shockwave = shockwave;
        w.tuning.hammer_stun = 1000;
        elora(&mut w, 30);
        w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
        w.add_creature(1, on_floor(28, 26.0)).unwrap();
        let ev = run(&mut w, fire(100, 0, 1), 2);
        assert_eq!(!hits(&ev).is_empty(), shockwave, "Schockwelle {shockwave}");
        if shockwave {
            assert!(w.creatures[0].stun > 0, "Hammer betäubt (A-17)");
        }
    }
    // größere Reichweite trifft weiter entfernte Gegner
    for (reach, hit) in [(14.0, false), (60.0, true)] {
        let mut w = world(|_| {});
        still_turret(&mut w);
        w.tuning.hammer_reach = reach;
        elora(&mut w, 30);
        w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
        w.add_creature(
            1,
            Vec2::new(30.0 * 32.0 + 16.0 + 90.0, on_floor(30, 26.0).y),
        )
        .unwrap();
        let ev = run(&mut w, fire(100, 0, 1), 2);
        assert_eq!(!hits(&ev).is_empty(), hit, "Reichweite {reach}");
    }
}

#[test]
fn grenade_shards_add_small_blasts() {
    let mut w = world(|_| {});
    still_turret(&mut w);
    w.tuning.grenade_shards = 3;
    elora(&mut w, 15);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    w.character_mut(0)
        .unwrap()
        .arsenal
        .give(Weapon::Grenade, 10, 10);
    w.character_mut(0).unwrap().arsenal.active = Weapon::Grenade;
    w.add_creature(1, on_floor(22, 26.0)).unwrap();
    // ein Schuss: drücken, loslassen (Granate feuert bei gehaltener Taste weiter)
    let mut ev = run(&mut w, fire(100, 0, 1), 1);
    ev.extend(run(&mut w, fire(100, 0, 2), 40));
    let blasts = ev
        .iter()
        .filter(|e| matches!(e, Event::Explosion { .. }))
        .count();
    assert_eq!(blasts, 4, "Einschlag + 3 Splitter");
}

#[test]
fn thorns_hurt_and_put_elora_back_on_safe_ground() {
    // Grube mit Dornen in Spalte 20 bis 24
    let mut w = world(|t| {
        for x in 20..=24 {
            set(t, x, FLOOR, Tile::Air);
            set(t, x, FLOOR + 1, Tile::Death);
        }
    });
    elora(&mut w, 10);
    let right = PlayerInput {
        direction: 1,
        ..PlayerInput::default()
    };
    run(&mut w, PlayerInput::default(), 10);
    let mut hurt = false;
    for _ in 0..200 {
        let ev = run(&mut w, right, 1);
        if ev.iter().any(|e| matches!(e, Event::Damage { .. })) {
            hurt = true;
            break;
        }
    }
    assert!(hurt, "in die Dornen gefallen");
    let ch = w.character(0).expect("lebt noch (E-283)");
    assert_eq!(ch.health, 10 - Tuning::default().thorn_damage);
    assert!(
        ch.core.pos.x < 18.0 * 32.0,
        "zurück vor der Grube: {:?}",
        ch.core.pos
    );
    assert!(ch.core.pos.y < FLOOR as f32 * 32.0);
    // Ohne Abenteuer bleibt es beim Tod
    let mut w = world(|t| {
        for x in 20..=24 {
            set(t, x, FLOOR, Tile::Air);
            set(t, x, FLOOR + 1, Tile::Death);
        }
    });
    w.adventure = false;
    elora(&mut w, 22);
    run(&mut w, PlayerInput::default(), 40);
    assert!(w.character(0).is_none());
}

// ---------------------------------------------------------------- Hüter aus der Luft (R2-M2.1)

fn diver_def() -> elora_sim::DiverDef {
    elora_sim::DiverDef {
        sight: 900.0,
        circle: [160.0, 40.0],
        speed: 3.0,
        circle_ms: 1000,
        aim_ms: 400,
        dive_speed: 14.0,
        stun_ms: 1500,
        drop_ms: 300,
        drop_speed: 5.0,
        drop_damage: 1,
        enrage_at: 0.5,
        summon_at: 0.34,
        summon: "flyer".into(),
        summon_max: 2,
        summon_ms: 500,
    }
}

/// Welt mit einem Hüter (Art 4) hoch über der Mitte und Elora am Boden.
fn boss_world() -> (World, u32) {
    let mut w = world(|_| {});
    let mut k = w.creature_kinds[0].clone();
    k.name = "hueter".into();
    k.size = [120.0, 100.0];
    k.health = 30;
    k.boss = true;
    k.behavior = Behavior::Diver(Box::new(diver_def()));
    w.creature_kinds.push(k);
    elora(&mut w, 30);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    let id = w
        .add_creature(4, elora_sim::Vec2::new(30.0 * 32.0, 10.0 * 32.0))
        .unwrap();
    (w, id)
}

fn boss(w: &World, id: u32) -> &elora_sim::Creature {
    w.creatures.iter().find(|c| c.id == id).expect("lebt")
}

#[test]
fn diver_circles_aims_dives_and_lies_stunned() {
    use elora_sim::creature::diver;
    let (mut w, id) = boss_world();
    let mut seen = Vec::new();
    let mut dropped = false;
    for _ in 0..400 {
        let ev = run(&mut w, PlayerInput::default(), 1);
        dropped |= ev.iter().any(|e| matches!(e, Event::CreatureFire { .. }));
        let m = boss(&w, id).mode;
        if seen.last() != Some(&m) {
            seen.push(m);
        }
    }
    assert_eq!(
        &seen[..5],
        &[
            diver::CIRCLE,
            diver::AIM,
            diver::DIVE,
            diver::STUNNED,
            diver::RISE
        ],
        "{seen:?}"
    );
    assert!(dropped, "lässt beim Kreisen Pollen fallen");
}

#[test]
fn diver_takes_damage_only_while_stunned() {
    use elora_sim::creature::diver;
    let (mut w, id) = boss_world();
    run(&mut w, PlayerInput::default(), 5);
    w.hurt_creature(id, 5);
    assert_eq!(boss(&w, id).health, 30, "in der Luft prallt es ab");
    // bis er benommen ist
    for _ in 0..400 {
        run(&mut w, PlayerInput::default(), 1);
        if boss(&w, id).mode == diver::STUNNED {
            break;
        }
    }
    assert_eq!(boss(&w, id).mode, diver::STUNNED);
    w.hurt_creature(id, 5);
    assert_eq!(boss(&w, id).health, 25);
}

#[test]
fn angry_diver_dives_twice_and_summons_helpers() {
    use elora_sim::creature::diver;
    let (mut w, id) = boss_world();
    w.creatures.iter_mut().find(|c| c.id == id).unwrap().health = 9;
    let mut dives = 0;
    let mut last = diver::SLEEP;
    for _ in 0..300 {
        run(&mut w, PlayerInput::default(), 1);
        let m = boss(&w, id).mode;
        if m == diver::DIVE && last != diver::DIVE {
            dives += 1;
        }
        if m == diver::STUNNED {
            break;
        }
        last = m;
    }
    assert_eq!(dives, 2, "zwei Sturzflüge vor dem Liegen");
    let helpers = w.creatures.iter().filter(|c| c.kind == 3).count();
    assert!((1..=2).contains(&helpers), "{helpers} Helfer (höchstens 2)");
}

#[test]
fn hook_flowers_wilt_while_the_diver_is_angry() {
    let (mut w, id) = boss_world();
    run(&mut w, PlayerInput::default(), 5);
    assert_eq!(w.collision.hook_wilt, None, "noch nicht wütend");
    w.creatures.iter_mut().find(|c| c.id == id).unwrap().health = 10;
    run(&mut w, PlayerInput::default(), 1);
    let first = w.collision.hook_wilt.expect("welkt");
    run(&mut w, PlayerInput::default(), 125);
    assert_eq!(w.collision.hook_wilt, Some(!first), "die andere Hälfte");
    w.creatures.clear();
    run(&mut w, PlayerInput::default(), 1);
    assert_eq!(w.collision.hook_wilt, None, "nach dem Kampf alle frisch");
}

// ---------------------------------------------------------------- Kapitel 2 (R2-M2.2)

fn add_kind(w: &mut World, name: &str, behavior: Behavior) -> usize {
    let mut k = kinds()[0].clone();
    k.name = name.into();
    k.behavior = behavior;
    w.creature_kinds.push(k);
    w.creature_kinds.len() - 1
}

#[test]
fn burrower_hides_until_elora_comes_and_only_then_is_dangerous() {
    use elora_sim::creature::burrow;
    let mut w = world(|_| {});
    let kind = add_kind(
        &mut w,
        "schlange",
        Behavior::Burrower {
            sight: 150.0,
            out_ms: 1000,
            hide_ms: 600,
            rise_ms: 400,
        },
    );
    let id = w.add_creature(kind, on_floor(30, 26.0)).unwrap();
    elora(&mut w, 10);
    run(&mut w, PlayerInput::default(), 60);
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.mode, burrow::HIDDEN, "Elora ist weit weg");
    assert!(!c.vulnerable(&w.creature_kinds[kind]));
    // Elora kommt näher: die Schlange schießt hoch
    w.spawn_character(0, on_floor(26, 28.0));
    let ev = run(&mut w, PlayerInput::default(), 5);
    assert!(ev.iter().any(|e| matches!(e, Event::CreatureAct { .. })));
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.mode, burrow::RISING, "wächst erst langsam heraus");
    assert!(!c.harmful(&w.creature_kinds[kind]));
    run(&mut w, PlayerInput::default(), 25);
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.mode, burrow::OUT);
    // nach out_ms wieder versteckt
    w.spawn_character(0, on_floor(5, 28.0));
    run(&mut w, PlayerInput::default(), 60);
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.mode, burrow::HIDDEN);
}

#[test]
fn lobbed_nuts_fly_in_an_arc() {
    let mut w = world(|_| {});
    let kind = add_kind(
        &mut w,
        "pirat",
        Behavior::Turret {
            interval_ms: 400,
            range: 600.0,
            shot_speed: 6.0,
            shot_damage: 1,
            lob: true,
        },
    );
    w.add_creature(kind, on_floor(30, 26.0)).unwrap();
    elora(&mut w, 18);
    let mut ys = Vec::new();
    for _ in 0..80 {
        run(&mut w, PlayerInput::default(), 1);
        if let Some(s) = w.creature_shots.first() {
            ys.push(s.vel.y);
        }
    }
    assert!(ys.len() > 5, "Nuss geworfen");
    assert!(ys[0] < 0.0, "erst nach oben");
    assert!(ys.windows(2).any(|v| v[1] > v[0]), "Schwerkraft wirkt");
}

#[test]
fn mushroom_touch_dazes_and_slows_elora() {
    let mut w = world(|_| {});
    let mut k = kinds()[0].clone();
    k.name = "pilzwicht".into();
    k.daze_ms = 2000;
    k.touch_damage = 0;
    k.behavior = Behavior::Walker {
        speed: 0.0,
        turn_at_edges: false,
    };
    w.creature_kinds.push(k);
    let kind = w.creature_kinds.len() - 1;
    w.add_creature(kind, on_floor(12, 26.0)).unwrap();
    elora(&mut w, 11);
    run(&mut w, PlayerInput::default(), 3);
    let dazed = w.character(0).unwrap().core.dazed;
    assert!(dazed > 0, "Rausch");
    assert_eq!(health(&w), 10, "kein Schaden");
    // langsamer laufen
    let right = PlayerInput {
        direction: 1,
        ..PlayerInput::default()
    };
    w.spawn_character(0, on_floor(30, 28.0));
    w.character_mut(0).unwrap().core.dazed = 500;
    run(&mut w, right, 40);
    let slow = w.character(0).unwrap().core.vel.x;
    w.character_mut(0).unwrap().core.dazed = 0;
    run(&mut w, right, 40);
    let fast = w.character(0).unwrap().core.vel.x;
    assert!(slow < fast * 0.7, "{slow} / {fast}");
}

#[test]
fn follower_follows_waits_at_gaps_and_never_hurts() {
    // Lücke mit Dornen in Spalte 30 bis 33
    let mut w = world(|t| {
        for x in 30..=33 {
            set(t, x, FLOOR, Tile::Air);
            set(t, x, FLOOR + 1, Tile::Death);
        }
    });
    let kind = add_kind(
        &mut w,
        "pilzkind",
        Behavior::Follower {
            speed: 4.0,
            jump: 9.0,
        },
    );
    let id = w.add_creature(kind, on_floor(10, 26.0)).unwrap();
    elora(&mut w, 25);
    run(&mut w, PlayerInput::default(), 150);
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    assert!(c.pos.x > 20.0 * 32.0, "folgt: {}", c.pos.x);
    assert_eq!(health(&w), 10, "harmlos");
    // Elora auf der anderen Seite der Lücke: das Kind wartet an der Kante
    w.spawn_character(0, on_floor(40, 28.0));
    run(&mut w, PlayerInput::default(), 200);
    let c = w.creatures.iter().find(|c| c.id == id).expect("lebt noch");
    assert!(c.pos.x < 30.0 * 32.0, "wartet: {}", c.pos.x);
    assert!(!c.vulnerable(&w.creature_kinds[kind]));
}

fn warden_def() -> elora_sim::creature::WardenDef {
    elora_sim::creature::WardenDef {
        sight: 900.0,
        attack_ms: 400,
        warn_ms: 200,
        spike_width: 60.0,
        spike_height: 100.0,
        spike_damage: 2,
        pull_ms: 200,
        open_ms: 600,
        cores: 3,
        enrage_at: 0.5,
        wall_every_ms: 300,
        wall_ms: 400,
        wall_height: 3,
    }
}

fn warden_world() -> (World, u32, usize) {
    let mut w = world(|_| {});
    let mut k = kinds()[0].clone();
    k.name = "waechter".into();
    k.size = [80.0, 200.0];
    k.health = 12;
    k.boss = true;
    k.behavior = Behavior::Warden(Box::new(warden_def()));
    w.creature_kinds.push(k);
    let kind = w.creature_kinds.len() - 1;
    let id = w.add_creature(kind, on_floor(40, 200.0)).unwrap();
    (w, id, kind)
}

#[test]
fn warden_strikes_with_warning_where_elora_stands() {
    use elora_sim::CreatureAct;
    let (mut w, _, _) = warden_world();
    elora(&mut w, 20);
    let ev = run(&mut w, PlayerInput::default(), 40);
    let warn = ev.iter().position(|e| {
        matches!(
            e,
            Event::CreatureAct {
                act: CreatureAct::Warn,
                ..
            }
        )
    });
    let strike = ev.iter().position(|e| {
        matches!(
            e,
            Event::CreatureAct {
                act: CreatureAct::Strike,
                ..
            }
        )
    });
    assert!(warn.is_some() && strike > warn, "erst Warnung, dann Stoß");
    assert!(health(&w) < 10, "Elora stand still und wurde getroffen");
}

#[test]
fn pulling_a_core_opens_the_warden_for_hits() {
    use elora_sim::creature::warden;
    let (mut w, id, kind) = warden_world();
    elora(&mut w, 30);
    run(&mut w, PlayerInput::default(), 3);
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    assert!(!c.vulnerable(&w.creature_kinds[kind]));
    // Hook am Wächter, Elora zieht weg (nach links)
    let away = PlayerInput {
        direction: -1,
        hook: true,
        ..PlayerInput::default()
    };
    let mut opened = false;
    for _ in 0..30 {
        let ch = w.character_mut(0).unwrap();
        ch.core.hook_state = HookState::Grabbed;
        ch.core.hooked_creature = Some(id);
        ch.core.pulling = true;
        ch.core.hook_tick = 0;
        ch.health = 10;
        run(&mut w, away, 1);
        if w.creatures.iter().find(|c| c.id == id).unwrap().mode == warden::OPEN {
            opened = true;
            break;
        }
    }
    assert!(opened, "Kern gelöst");
    let c = w.creatures.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.count, 2, "zwei Kerne übrig");
    assert!(c.vulnerable(&w.creature_kinds[kind]));
}

#[test]
fn angry_warden_raises_root_walls_that_disappear() {
    let (mut w, id, _) = warden_world();
    w.creatures.iter_mut().find(|c| c.id == id).unwrap().health = 5;
    elora(&mut w, 20);
    let mut set = 0;
    let mut reset = 0;
    for _ in 0..120 {
        w.character_mut(0).unwrap().health = 10;
        for e in run(&mut w, PlayerInput::default(), 1) {
            if let Event::TileSet { tile, .. } = e {
                if tile == Tile::Unhookable {
                    set += 1;
                } else {
                    reset += 1;
                }
            }
        }
    }
    assert!(set > 0, "Wurzelwand");
    assert!(reset > 0, "verschwindet wieder");
}

// ---------------------------------------------------------------- Kapitel 3 (R2-M2.3)

fn damage_of(ev: &[Event], id: u32) -> Option<i32> {
    ev.iter().find_map(|e| match e {
        Event::CreatureHit { id: i, damage, .. } if *i == id => Some(*damage),
        _ => None,
    })
}

#[test]
fn armored_crab_only_takes_hits_from_above() {
    let mut w = world(|_| {});
    let kind = add_kind(
        &mut w,
        "krabbe",
        Behavior::Walker {
            speed: 0.0,
            turn_at_edges: true,
        },
    );
    w.creature_kinds[kind].armor = true;
    w.creature_kinds[kind].touch_damage = 0;
    let id = w.add_creature(kind, on_floor(30, 26.0)).unwrap();
    elora(&mut w, 29);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    // Hammer von der Seite prallt ab
    let ev = run(&mut w, fire(100, 0, 1), 2);
    assert_eq!(damage_of(&ev, id), Some(0), "Panzer hält");
    assert_eq!(w.creatures[0].health, 6);
    // Laser von der Seite auch
    w.spawn_character(0, on_floor(24, 28.0));
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    w.character_mut(0)
        .unwrap()
        .arsenal
        .give(Weapon::Laser, 10, 10);
    w.character_mut(0).unwrap().arsenal.active = Weapon::Laser;
    let ev = run(&mut w, fire(100, 0, 3), 2);
    assert_eq!(damage_of(&ev, id), Some(0), "Laser prallt ab");
    // Hammer von oben trifft
    w.character_mut(0).unwrap().arsenal.active = Weapon::Hammer;
    let top = w.creatures[0].pos - Vec2::new(0.0, 13.0 + 14.0 + 4.0);
    w.spawn_character(0, top);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    let ev = run(&mut w, fire(0, 100, 5), 2);
    assert!(
        damage_of(&ev, id).is_some_and(|d| d > 0),
        "von oben verwundbar"
    );
    assert!(w.creatures[0].health < 6);
}

fn worm(w: &mut World) -> usize {
    add_kind(
        w,
        "wurm",
        Behavior::Leaper {
            sight: 400.0,
            speed: 2.0,
            warn_ms: 500,
            jump_x: 6.0,
            jump_y: 11.0,
            rest_ms: 800,
        },
    )
}

#[test]
fn dune_worm_travels_under_sand_warns_and_leaps_at_elora() {
    use elora_sim::creature::leaper;
    let mut w = world(|_| {});
    let kind = worm(&mut w);
    let id = w.add_creature(kind, on_floor(40, 26.0)).unwrap();
    elora(&mut w, 30);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    let get = |w: &World| w.creatures.iter().find(|c| c.id == id).unwrap().clone();
    // unter dem Sand: kommt näher, harmlos und unverwundbar
    run(&mut w, PlayerInput::default(), 10);
    let c = get(&w);
    assert_eq!(c.mode, leaper::UNDER);
    assert!(c.pos.x < on_floor(40, 26.0).x, "wandert auf Elora zu");
    assert!(!c.vulnerable(&w.creature_kinds[kind]) && !c.harmful(&w.creature_kinds[kind]));
    w.hurt_creature(id, 1);
    assert_eq!(get(&w).health, 6, "unter dem Sand nicht zu treffen");
    // Warnung, dann Sprung
    let mut modes = Vec::new();
    let mut top = f32::MAX;
    for _ in 0..200 {
        run(&mut w, PlayerInput::default(), 1);
        let c = get(&w);
        if modes.last() != Some(&c.mode) {
            modes.push(c.mode);
        }
        if c.mode == leaper::LEAP {
            top = top.min(c.pos.y);
            assert!(c.vulnerable(&w.creature_kinds[kind]));
        }
    }
    assert!(
        modes
            .windows(3)
            .any(|m| m == [leaper::WARN, leaper::LEAP, leaper::UNDER]),
        "Warnung, Sprung, Eintauchen: {modes:?}"
    );
    assert!(top < on_floor(40, 26.0).y - 64.0, "springt im Bogen");
}

#[test]
fn dune_worm_lands_where_elora_stood_at_the_warning() {
    use elora_sim::creature::leaper;
    let mut w = world(|_| {});
    let kind = worm(&mut w);
    w.creature_kinds[kind].touch_damage = 0;
    let id = w.add_creature(kind, on_floor(30, 26.0)).unwrap();
    elora(&mut w, 25);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    let stood = w.character(0).unwrap().core.pos.x;
    let mut was_leaping = false;
    let mut landed = None;
    for _ in 0..300 {
        run(&mut w, PlayerInput::default(), 1);
        let c = w.creatures.iter().find(|c| c.id == id).unwrap();
        if was_leaping && c.mode == leaper::UNDER {
            landed = Some(c.pos.x);
            break;
        }
        was_leaping = c.mode == leaper::LEAP;
    }
    let x = landed.expect("gesprungen und gelandet");
    assert!((x - stood).abs() < 40.0, "landet bei Elora: {x} vs {stood}");
}

#[test]
fn spark_moth_hovers_above_elora_and_sparks_glow_on_the_ground() {
    let mut w = world(|_| {});
    let kind = add_kind(
        &mut w,
        "motte",
        Behavior::Flyer {
            speed: 3.0,
            sight: 500.0,
            hover: 120.0,
            drop_ms: 600,
            drop_damage: 1,
            glow_ms: 1000,
        },
    );
    w.creature_kinds[kind].touch_damage = 0;
    w.add_creature(kind, on_floor(20, 26.0) - Vec2::new(0.0, 200.0))
        .unwrap();
    elora(&mut w, 22);
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    run(&mut w, PlayerInput::default(), 150);
    let elora_pos = w.character(0).unwrap().core.pos;
    let m = &w.creatures[0];
    assert!((m.pos.x - elora_pos.x).abs() < 30.0, "über Elora");
    assert!(
        (elora_pos.y - m.pos.y - 120.0).abs() < 20.0,
        "in Schwebehöhe"
    );
    // Elora geht zur Seite: der Funke landet und glüht
    w.spawn_character(0, on_floor(10, 28.0));
    w.character_mut(0).unwrap().invulnerable_until = u64::MAX;
    let mut landed = 0;
    for _ in 0..40 {
        run(&mut w, PlayerInput::default(), 1);
        landed = landed.max(w.creature_shots.iter().filter(|s| s.landed).count());
    }
    assert!(landed > 0, "Funke liegt glühend am Boden");
    let s = w.creature_shots.iter().find(|s| s.landed).unwrap();
    assert!(s.pos.y > on_floor(20, 26.0).y, "am Boden");
    // nach glow_ms verglüht
    w.creatures.clear();
    run(&mut w, PlayerInput::default(), 60);
    assert!(w.creature_shots.is_empty(), "verglüht");
}

#[test]
fn glowing_spark_burns_elora() {
    let mut w = world(|_| {});
    elora(&mut w, 20);
    let feet = w.character(0).unwrap().core.pos;
    w.creature_shots.push(elora_sim::creature::CreatureShot {
        owner: 0,
        pos: feet,
        vel: Vec2::ZERO,
        damage: 1,
        ticks: 0,
        gravity: 0.0,
        glow: 50,
        landed: true,
    });
    let before = health(&w);
    run(&mut w, PlayerInput::default(), 2);
    assert!(health(&w) < before, "glühender Funke brennt");
}
