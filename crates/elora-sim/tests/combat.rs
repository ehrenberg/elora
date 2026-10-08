//! Combat tests (M2): weapons, damage, pickups, death and respawn.

use elora_sim::{
    Collision, DeathCause, DummyPattern, Event, PickupKind, PlayerInput, Tile, Tuning, Vec2,
    Weapon, World,
};

/// Open arena 60×20 tiles, floor in row 18.
fn arena_tiles() -> Vec<Tile> {
    let (w, h) = (60, 20);
    let mut tiles = vec![Tile::Air; w * h];
    for x in 0..w {
        tiles[x] = Tile::Solid;
        tiles[18 * w + x] = Tile::Solid;
        tiles[19 * w + x] = Tile::Solid;
    }
    for y in 0..h {
        tiles[y * w] = Tile::Solid;
        tiles[y * w + w - 1] = Tile::Solid;
    }
    tiles
}

fn arena() -> World {
    World::new(Tuning::default(), Collision::new(60, 20, arena_tiles()))
}

/// Floor position (figure standing) in tile column `tx`. The bottom of the box sits
/// 1 unit above the floor, otherwise the rounded corner would already be in the floor.
fn ground(tx: i32) -> Vec2 {
    Vec2::new(tx as f32 * 32.0 + 16.0, 18.0 * 32.0 - 15.0)
}

fn idle() -> PlayerInput {
    PlayerInput::default()
}

/// Input "aims at (x, y) relative" with fire counter `fire`.
fn aim(x: i32, y: i32, fire: u8) -> PlayerInput {
    PlayerInput {
        target_x: x,
        target_y: y,
        fire,
        ..PlayerInput::default()
    }
}

fn run(w: &mut World, inputs: &[PlayerInput], ticks: u32) {
    for _ in 0..ticks {
        w.step(inputs);
    }
}

fn health(w: &World, i: usize) -> i32 {
    w.character(i).expect("lebt").health
}

fn give(w: &mut World, i: usize, weapon: Weapon) {
    let ch = w.character_mut(i).unwrap();
    ch.arsenal.give(weapon, 10, 10);
    ch.arsenal.active = weapon;
}

/// Two standing players: 0 at `a`, 1 at `b`.
fn duel(a: i32, b: i32) -> World {
    let mut w = arena();
    w.spawn(ground(a));
    w.spawn(ground(b));
    run(&mut w, &[idle(), idle()], 5);
    w
}

#[test]
fn hammer_hits_and_knocks_up_and_away() {
    let mut w = duel(10, 11);
    w.step(&[aim(100, 0, 1), idle()]);
    assert_eq!(health(&w, 1), 7, "T-18: 3 Schaden");
    let v = w.core(1).unwrap().vel;
    assert!(v.x > 0.0 && v.y < -5.0, "Knockback nach oben/weg: {v:?}");
    assert!(
        w.events
            .iter()
            .any(|e| matches!(e, Event::HammerHit { .. }))
    );
    // after a hit: lockout 1/3 s = 16 ticks; as in the original it is already counted
    // down once in the same tick
    assert_eq!(w.character(0).unwrap().arsenal.reload_timer, 15);
}

#[test]
fn hammer_miss_uses_fire_delay() {
    let mut w = duel(10, 20);
    w.step(&[aim(100, 0, 1), idle()]);
    assert_eq!(health(&w, 1), 10);
    assert_eq!(w.character(0).unwrap().arsenal.reload_timer, 6 - 1);
}

#[test]
fn hammer_needs_a_click_not_just_holding() {
    let mut w = duel(10, 11);
    w.step(&[aim(100, 0, 1), idle()]);
    run(&mut w, &[aim(100, 0, 1), idle()], 60); // key stays pressed
    assert_eq!(health(&w, 1), 7, "Hammer ist kein Dauerfeuer");
}

#[test]
fn hammer_does_not_hit_through_walls() {
    let mut tiles = arena_tiles();
    for y in 10..18 {
        tiles[y * 60 + 11] = Tile::Solid;
    }
    let mut w = World::new(Tuning::default(), Collision::new(60, 20, tiles));
    w.spawn(Vec2::new(11.0 * 32.0 - 15.0, 18.0 * 32.0 - 14.0));
    w.spawn(Vec2::new(12.0 * 32.0 + 15.0, 18.0 * 32.0 - 14.0));
    run(&mut w, &[idle(), idle()], 5);
    w.step(&[aim(100, 0, 1), idle()]);
    assert_eq!(health(&w, 1), 10);
}

#[test]
fn laser_hits_with_damage_and_light_knockback() {
    let mut w = duel(10, 30); // 640 units apart
    give(&mut w, 0, Weapon::Laser);
    w.step(&[aim(100, 0, 1), idle()]);
    assert_eq!(health(&w, 1), 5, "T-20: 5 Schaden");
    // knockback 2 (E-052), ground friction 0.5 already applied once in the same tick → 1
    let vx = w.core(1).unwrap().vel.x;
    assert!((vx - 1.0).abs() < 0.01, "E-052: leichter Stoß, war {vx}");
    assert_eq!(
        w.character(0).unwrap().arsenal.slot(Weapon::Laser).ammo,
        Some(9)
    );
}

#[test]
fn laser_range_is_limited() {
    let mut w = duel(5, 33); // 896 units > 850 (T-21)
    give(&mut w, 0, Weapon::Laser);
    w.step(&[aim(100, 0, 1), idle()]);
    assert_eq!(health(&w, 1), 10);
}

#[test]
fn laser_bounces_off_walls() {
    let mut w = arena();
    w.spawn(ground(10));
    run(&mut w, &[idle()], 5);
    give(&mut w, 0, Weapon::Laser);
    // diagonally at the floor: first bounce right at the shot
    let mut bounced = false;
    for _ in 0..20 {
        w.step(&[aim(100, 60, 1)]);
        bounced |= w
            .events
            .iter()
            .any(|e| matches!(e, Event::LaserBounce { .. }));
    }
    assert!(bounced);
}

#[test]
fn laser_is_full_auto() {
    let mut w = arena();
    w.spawn(ground(10));
    give(&mut w, 0, Weapon::Laser);
    let mut shots = 0;
    for _ in 0..100 {
        w.step(&[aim(100, -50, 1)]); // hold pressed
        shots += w
            .events
            .iter()
            .filter(|e| matches!(e, Event::Fire { .. }))
            .count();
    }
    // fire delay 750 ms = 37 ticks → shots at 0, 38, 76
    assert_eq!(shots, 3);
}

#[test]
fn grenade_rocket_jump_and_self_damage() {
    let mut w = arena();
    w.spawn(ground(20));
    run(&mut w, &[idle()], 5);
    give(&mut w, 0, Weapon::Grenade);
    w.step(&[aim(0, 100, 1)]); // straight down
    let mut min_vy = 0.0f32;
    for _ in 0..10 {
        w.step(&[aim(0, 100, 1)]);
        min_vy = min_vy.min(w.core(0).unwrap().vel.y);
    }
    assert!(min_vy < -10.0, "Rocket-Jump: vy = {min_vy}");
    assert_eq!(health(&w, 0), 7, "Eigenschaden 6/2 = 3 (T-26)");
}

#[test]
fn grenade_direct_hit_explodes_on_player() {
    let mut w = duel(10, 18);
    give(&mut w, 0, Weapon::Grenade);
    let mut exploded = false;
    for _ in 0..40 {
        w.step(&[aim(100, -8, 1), idle()]);
        exploded |= w
            .events
            .iter()
            .any(|e| matches!(e, Event::Explosion { .. }));
        if exploded {
            break;
        }
    }
    assert!(exploded);
    assert_eq!(health(&w, 1), 4, "voller Schaden 6 in der Explosionsmitte");
}

#[test]
fn armor_absorbs_damage_like_original() {
    // 6 damage with 5 armour: 1 to HP, 5 to armour
    let mut w2 = duel(10, 18);
    w2.character_mut(1).unwrap().armor = 5;
    give(&mut w2, 0, Weapon::Grenade);
    for _ in 0..40 {
        w2.step(&[aim(100, -8, 1), idle()]);
        if w2
            .events
            .iter()
            .any(|e| matches!(e, Event::Explosion { .. }))
        {
            break;
        }
    }
    let ch = w2.character(1).unwrap();
    assert_eq!((ch.health, ch.armor), (9, 0));
}

#[test]
fn no_ammo_blocks_and_reports() {
    let mut w = arena();
    w.spawn(ground(10));
    give(&mut w, 0, Weapon::Laser);
    w.character_mut(0).unwrap().arsenal.slots[Weapon::Laser.index()].ammo = Some(0);
    w.step(&[aim(100, 0, 0)]); // first input after joining does not count as a click
    w.step(&[aim(100, 0, 1)]);
    assert!(w.events.iter().any(|e| matches!(e, Event::NoAmmo { .. })));
    assert_eq!(w.character(0).unwrap().arsenal.reload_timer, 6 - 1);
}

#[test]
fn weapon_switch_waits_for_reload() {
    let mut w = arena();
    w.spawn(ground(10));
    give(&mut w, 0, Weapon::Laser);
    w.step(&[aim(100, -50, 1)]); // fire laser → 37 ticks reload
    let switch = PlayerInput {
        wanted_weapon: 1,
        ..aim(100, -50, 2)
    };
    w.step(&[switch]);
    assert_eq!(w.character(0).unwrap().arsenal.active, Weapon::Laser);
    run(&mut w, &[switch], 40);
    assert_eq!(w.character(0).unwrap().arsenal.active, Weapon::Hammer);
}

#[test]
fn mouse_wheel_cycles_owned_weapons() {
    let mut w = arena();
    w.spawn(ground(10));
    w.character_mut(0)
        .unwrap()
        .arsenal
        .give(Weapon::Laser, 10, 10);
    w.step(&[idle()]); // first input after joining does not count as a click
    w.step(&[PlayerInput {
        next_weapon: 2,
        ..idle()
    }]); // one click forward
    assert_eq!(w.character(0).unwrap().arsenal.active, Weapon::Laser);
    w.step(&[PlayerInput {
        next_weapon: 4,
        ..idle()
    }]); // another one: back to the hammer
    assert_eq!(w.character(0).unwrap().arsenal.active, Weapon::Hammer);
}

#[test]
fn pickups_follow_original_rules() {
    let mut w = arena();
    w.spawn(ground(10));
    w.add_pickup(PickupKind::Health, ground(10));
    w.add_pickup(PickupKind::Weapon(Weapon::Laser), ground(10));
    w.step(&[idle()]);
    // full health: heart stays, laser is taken
    assert!(w.pickups[0].available());
    assert!(!w.pickups[1].available());
    assert_eq!(
        w.character(0).unwrap().arsenal.slot(Weapon::Laser).ammo,
        Some(10)
    );
    // hurt: heart is taken
    w.character_mut(0).unwrap().health = 5;
    w.step(&[idle()]);
    assert_eq!(health(&w, 0), 6);
    assert!(!w.pickups[0].available());
    // back after 15 s (T-29)
    w.character_mut(0).unwrap().health = 10;
    run(&mut w, &[idle()], 15 * 50 + 1);
    assert!(w.pickups[0].available());
}

#[test]
fn respawn_on_click_after_half_second_or_auto_after_three() {
    let mut w = arena();
    w.spawn_points.push(ground(30));
    w.spawn(ground(10));
    w.die(0, None, DeathCause::World);
    // click right away: still too early
    run(&mut w, &[aim(1, 0, 1)], 10);
    assert!(w.character(0).is_none());
    run(&mut w, &[aim(1, 0, 1)], 20);
    assert!(
        w.character(0).is_some(),
        "nach 0,5 s mit gedrückter Feuertaste"
    );

    w.die(0, None, DeathCause::World);
    run(&mut w, &[idle()], 149);
    assert!(w.character(0).is_none());
    run(&mut w, &[idle()], 2);
    assert!(w.character(0).is_some(), "Auto-Respawn nach 3 s");
    assert_eq!(
        w.character(0).unwrap().arsenal.active,
        Weapon::Hammer,
        "E-025"
    );
}

#[test]
fn walking_dummy_turns_at_walls() {
    let mut w = arena();
    let d = w.add_dummy(ground(3), DummyPattern::Walk);
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    for _ in 0..600 {
        w.step(&[]);
        let x = w.core(d).unwrap().pos.x;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
    }
    assert!(
        min_x < 80.0 && max_x > 1800.0,
        "läuft hin und her: {min_x}..{max_x}"
    );
}

#[test]
fn jumping_dummy_uses_double_jump() {
    let mut w = arena();
    let d = w.add_dummy(ground(20), DummyPattern::Jump);
    let start = w.core(d).unwrap().pos.y;
    let mut min_y = start;
    for _ in 0..200 {
        w.step(&[]);
        min_y = min_y.min(w.core(d).unwrap().pos.y);
    }
    assert!(
        start - min_y > 250.0,
        "Doppelsprung-Höhe: {}",
        start - min_y
    );
}

#[test]
fn dummy_respawns_at_home_after_three_seconds() {
    let mut w = arena();
    let d = w.add_dummy(ground(20), DummyPattern::Stand);
    w.die(d, None, DeathCause::World);
    run(&mut w, &[], 151);
    assert_eq!(w.core(d).unwrap().pos, ground(20));
}

/// Map change in the adventure (playtest R2-M2.4): the new world gets the client's
/// already incremented fire counter – that is not a click. The next press shoots.
#[test]
fn first_input_after_joining_does_not_fire() {
    let mut w = arena();
    let i = w.spawn(ground(10));
    w.character_mut(i)
        .unwrap()
        .arsenal
        .give(Weapon::Grenade, 10, 10);
    w.character_mut(i).unwrap().arsenal.active = Weapon::Grenade;
    let fires = |w: &World| {
        w.events
            .iter()
            .filter(|e| matches!(e, Event::Fire { .. }))
            .count()
    };
    // counter is at 6 (pressed and released three times in the old world)
    w.step(&[aim(100, 0, 6)]);
    assert_eq!(fires(&w), 0, "kein Schuss beim Betreten");
    for _ in 0..30 {
        w.step(&[aim(100, 0, 6)]);
    }
    w.step(&[aim(100, 0, 7)]);
    assert_eq!(fires(&w), 1, "der nächste Druck schießt");
}
