//! Fähigkeiten (R2-M1, A1.1, E-226 bis E-230): Hook-Ruck, Stampfen, Eisgriff, Gleiten
//! sowie Kletterwand und Bröckelboden.

use elora_sim::character::events;
use elora_sim::{
    Abilities, Ability, Collision, Event, HookState, PlayerInput, Tile, Tuning, Vec2, World,
};

const W: usize = 60;
const H: usize = 30;
/// Bodenzeile der Arena.
const FLOOR: usize = 28;

/// Offene Arena mit Boden in Zeile 28; `edit` setzt weitere Tiles.
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
    World::new(Tuning::default(), Collision::new(W, H, tiles))
}

fn set(tiles: &mut [Tile], x: usize, y: usize, t: Tile) {
    tiles[y * W + x] = t;
}

/// Figur steht auf Zeile `row`, Spalte `tx`.
fn standing(tx: usize, row: usize) -> Vec2 {
    #[allow(clippy::cast_precision_loss)]
    Vec2::new(tx as f32 * 32.0 + 16.0, row as f32 * 32.0 - 15.0)
}

fn input(direction: i8, jump: bool, down: bool) -> PlayerInput {
    PlayerInput {
        direction,
        jump,
        down,
        ..PlayerInput::default()
    }
}

fn run(w: &mut World, i: PlayerInput, ticks: u32) {
    for _ in 0..ticks {
        w.step(&[i]);
    }
}

fn core(w: &World) -> &elora_sim::CharacterCore {
    w.core(0).expect("lebt")
}

fn spawn(w: &mut World, at: Vec2, abilities: Abilities) {
    let i = w.join();
    w.set_abilities(i, abilities);
    w.spawn_character(i, at);
}

fn with(a: Ability) -> Abilities {
    Abilities::NONE.with(a)
}

/// Springt vom Boden und steigt bis kurz vor den Scheitel.
fn jump_up(w: &mut World) {
    run(w, input(0, true, false), 1);
    run(w, input(0, false, false), 12);
}

// ---------------------------------------------------------------- Hook-Ruck

/// Hook an der Decke (Zeile 18) über der Figur.
fn hook_ceiling(abilities: Abilities) -> World {
    let mut w = world(|t| (5..55).for_each(|x| set(t, x, 18, Tile::Solid)));
    spawn(&mut w, standing(30, FLOOR), abilities);
    let up = PlayerInput {
        hook: true,
        target_x: 0,
        target_y: -100,
        ..PlayerInput::default()
    };
    for _ in 0..30 {
        w.step(&[up]);
        if core(&w).hook_state == HookState::Grabbed {
            break;
        }
    }
    assert_eq!(core(&w).hook_state, HookState::Grabbed, "Hook greift");
    w
}

fn ruck(w: &mut World) -> PlayerInput {
    let i = PlayerInput {
        hook: true,
        ability: true,
        target_x: 0,
        target_y: -100,
        ..PlayerInput::default()
    };
    w.step(&[i]);
    i
}

#[test]
fn hook_ruck_pulls_hard_with_cooldown() {
    let mut w = hook_ceiling(with(Ability::HookRuck));
    let held = ruck(&mut w);
    let c = core(&w);
    assert!(c.triggered_events & events::HOOK_RUCK != 0);
    assert!(
        c.vel.y < -Tuning::default().hook_drag_speed,
        "schneller als normaler Hook-Zug: {:?}",
        c.vel
    );
    // Taste halten löst nicht erneut aus, neuer Druck erst nach der Abklingzeit
    w.step(&[held]);
    assert_eq!(core(&w).triggered_events & events::HOOK_RUCK, 0);
    let released = PlayerInput {
        ability: false,
        ..held
    };
    w.step(&[released]);
    w.step(&[held]);
    assert_eq!(
        core(&w).triggered_events & events::HOOK_RUCK,
        0,
        "Abklingzeit"
    );
}

#[test]
fn hook_ruck_needs_ability() {
    let mut w = hook_ceiling(Abilities::NONE);
    let before = hook_ceiling(Abilities::NONE);
    ruck(&mut w);
    let mut plain = before;
    plain.step(&[PlayerInput {
        hook: true,
        target_x: 0,
        target_y: -100,
        ..PlayerInput::default()
    }]);
    assert_eq!(core(&w).vel, core(&plain).vel, "ohne Fähigkeit wirkungslos");
}

// ---------------------------------------------------------------- Stampfen

#[test]
fn stomp_breaks_crumble_floor_and_stays_broken() {
    // Bröckelboden-Insel in Zeile 20, darunter Luft
    let mut w = world(|t| (20..26).for_each(|x| set(t, x, 20, Tile::Crumble)));
    spawn(&mut w, standing(22, 20), with(Ability::Stomp));
    run(&mut w, PlayerInput::default(), 10);
    jump_up(&mut w);
    run(&mut w, input(0, false, true), 1);
    let c = core(&w);
    assert!(c.stomping && c.triggered_events & events::STOMP != 0);
    assert!((c.vel.y - Tuning::default().stomp_speed).abs() < 0.01 && c.vel.x == 0.0);

    let mut broken = Vec::new();
    for _ in 0..30 {
        w.step(&[input(0, false, true)]);
        broken.extend(w.events.iter().filter_map(|e| match *e {
            Event::TileBroken { tx, ty } => Some((tx, ty)),
            _ => None,
        }));
    }
    assert!(
        broken.contains(&(22, 20)),
        "unter der Figur zerbrochen: {broken:?}"
    );
    assert_eq!(w.collision.tile(22, 20), Tile::Air);
    assert!(
        w.collision.tile(25, 20) == Tile::Crumble,
        "außerhalb des Radius bleibt"
    );
    // Elora fällt durch das Loch auf den Boden
    run(&mut w, PlayerInput::default(), 60);
    assert!(core(&w).pos.y > 27.0 * 32.0);
}

#[test]
fn stomp_does_not_break_normal_ground_and_lands_on_platforms() {
    let mut w = world(|t| (20..26).for_each(|x| set(t, x, 20, Tile::Platform)));
    spawn(&mut w, standing(22, 20), with(Ability::Stomp));
    run(&mut w, PlayerInput::default(), 10);
    jump_up(&mut w);
    let mut landed = false;
    for _ in 0..30 {
        w.step(&[input(0, false, true)]);
        landed |= w.events.iter().any(|e| matches!(e, Event::Stomp { .. }));
        if landed {
            break;
        }
    }
    assert!(landed, "Aufprall auf der Plattform");
    assert!((core(&w).pos.y - (20.0 * 32.0 - 14.0)).abs() < 1.5);
    assert_eq!(w.collision.tile(22, 20), Tile::Platform);
}

#[test]
fn down_in_air_without_stomp_is_unchanged() {
    let mut a = world(|_| {});
    spawn(&mut a, standing(30, FLOOR), Abilities::NONE);
    let mut b = a.clone();
    b.set_abilities(0, with(Ability::Glide));
    for w in [&mut a, &mut b] {
        jump_up(w);
        run(w, input(1, false, true), 20);
    }
    assert_eq!(core(&a).pos, core(&b).pos);
    assert!(!core(&a).stomping);
}

// ---------------------------------------------------------------- Eisgriff

/// Kletterwand in Spalte 40 (Zeilen 5..28); Figur springt von links dagegen.
fn climb_world(abilities: Abilities, wall: Tile) -> World {
    let mut w = world(|t| (5..FLOOR).for_each(|y| set(t, 40, y, wall)));
    spawn(&mut w, standing(37, FLOOR), abilities);
    run(&mut w, PlayerInput::default(), 10);
    run(&mut w, input(1, true, false), 1);
    run(&mut w, input(1, false, false), 32);
    w
}

#[test]
fn grip_holds_on_climbing_wall_then_slides_off() {
    let mut w = climb_world(with(Ability::Grip), Tile::Climb);
    let c = core(&w);
    assert_eq!(c.grip, 1, "haftet rechts");
    assert!(c.vel.y <= Tuning::default().grip_slide_speed + 0.01);
    // nach der Haftzeit (1 s) fällt die Figur normal
    run(&mut w, input(1, false, false), 50);
    assert_eq!(core(&w).grip, 0);
}

#[test]
fn grip_only_on_climbing_wall() {
    let w = climb_world(with(Ability::Grip), Tile::Solid);
    assert_eq!(core(&w).grip, 0);
    let w = climb_world(Abilities::NONE, Tile::Climb);
    assert_eq!(core(&w).grip, 0);
}

#[test]
fn wall_jump_pushes_away_and_keeps_air_jump() {
    let mut w = climb_world(with(Ability::Grip), Tile::Climb);
    run(&mut w, input(1, true, false), 1);
    let c = core(&w);
    assert!(c.triggered_events & events::WALL_JUMP != 0);
    assert!(
        c.vel.x < 0.0 && c.vel.y < 0.0,
        "weg von der Wand: {:?}",
        c.vel
    );
    // Doppelsprung danach noch möglich
    run(&mut w, input(0, false, false), 2);
    run(&mut w, input(0, true, false), 1);
    assert!(core(&w).triggered_events & events::AIR_JUMP != 0);
}

#[test]
fn climbing_wall_is_not_hookable() {
    assert!(!Tile::Climb.is_hookable() && Tile::Climb.is_solid());
    assert!(Tile::Crumble.is_hookable());
    let mut w = world(|t| (5..55).for_each(|x| set(t, x, 18, Tile::Climb)));
    spawn(&mut w, standing(30, FLOOR), Abilities::ALL);
    let up = PlayerInput {
        hook: true,
        target_x: 0,
        target_y: -100,
        ..PlayerInput::default()
    };
    let mut grabbed = false;
    for _ in 0..30 {
        w.step(&[up]);
        grabbed |= core(&w).hook_state == HookState::Grabbed;
    }
    assert!(!grabbed);
}

// ---------------------------------------------------------------- Gleiten

/// Sprung, Doppelsprung, dann Springen halten.
fn glide_fall(abilities: Abilities) -> f32 {
    let mut w = world(|_| {});
    spawn(&mut w, standing(30, FLOOR), abilities);
    run(&mut w, PlayerInput::default(), 5);
    run(&mut w, input(0, true, false), 1);
    run(&mut w, input(0, false, false), 5);
    run(&mut w, input(0, true, false), 40);
    core(&w).vel.y
}

#[test]
fn glide_caps_fall_speed_after_air_jump() {
    let t = Tuning::default();
    let gliding = glide_fall(with(Ability::Glide));
    assert!(
        gliding > 0.0 && gliding <= t.glide_fall_speed + 0.01,
        "{gliding}"
    );
    assert!(glide_fall(Abilities::NONE) > t.glide_fall_speed * 2.0);
}

#[test]
fn no_glide_while_air_jump_unused() {
    let mut w = world(|_| {});
    spawn(&mut w, standing(30, FLOOR), with(Ability::Glide));
    run(&mut w, PlayerInput::default(), 5);
    // Bodensprung gehalten bis über den Scheitel: kein Gleiten
    run(&mut w, input(0, true, false), 45);
    assert!(!core(&w).gliding);
}

#[test]
fn down_on_platform_still_drops_through_with_stomp() {
    let mut w = world(|t| (20..26).for_each(|x| set(t, x, 20, Tile::Platform)));
    spawn(&mut w, standing(22, 20), with(Ability::Stomp));
    run(&mut w, PlayerInput::default(), 10);
    assert!(core(&w).is_grounded(&w.collision));
    run(&mut w, input(0, false, true), 30);
    assert!(!core(&w).stomping);
    assert!(
        core(&w).pos.y > 21.0 * 32.0,
        "durch die Plattform gefallen: {:?}",
        core(&w).pos
    );
}

#[test]
fn hook_grabs_hook_point_and_passes_through_it_otherwise() {
    // Hook-Blüte zwei Tiles über Elora, Decke weit darüber
    let mut w = world(|t| set(t, 10, 24, Tile::HookPoint));
    let i = w.join();
    w.spawn_character(i, standing(10, FLOOR));
    run(&mut w, PlayerInput::default(), 5);
    let hook = PlayerInput {
        hook: true,
        target_x: 0,
        target_y: -100,
        ..PlayerInput::default()
    };
    run(&mut w, hook, 12);
    let core = &w.character(i).unwrap().core;
    assert_eq!(core.hook_state, HookState::Grabbed);
    assert_eq!(
        core.hook_pos,
        Vec2::new(10.0 * 32.0 + 16.0, 24.0 * 32.0 + 16.0)
    );
    // Figuren laufen hindurch, sie ist nicht fest
    assert!(!Tile::HookPoint.is_solid());
    // welk (nur beim Hüter, siehe creatures.rs): Spalten wechseln sich ab
    let mut c = w.collision.clone();
    c.hook_wilt = Some(true);
    assert!(!c.hook_point_active(10) && c.hook_point_active(11));
    c.hook_wilt = Some(false);
    assert!(c.hook_point_active(10) && !c.hook_point_active(11));
}
