//! Abilities (R2-M1, A1.1, E-226 to E-230): hook jerk, stomp, ice grip, gliding
//! as well as climbing wall and crumbling floor.

use elora_sim::character::events;
use elora_sim::{
    Abilities, Ability, Collision, Event, HookState, PlayerInput, Tile, Tuning, Vec2, World,
};

const W: usize = 60;
const H: usize = 30;
/// Floor row of the arena.
const FLOOR: usize = 28;

/// Open arena with floor in row 28; `edit` sets further tiles.
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

/// Figure stands on row `row`, column `tx`.
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

/// Jumps off the floor and rises until just before the apex.
fn jump_up(w: &mut World) {
    run(w, input(0, true, false), 1);
    run(w, input(0, false, false), 12);
}

// ---------------------------------------------------------------- Hook jerk

/// Hook on the ceiling (row 18) above the figure.
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

fn jerk(w: &mut World) -> PlayerInput {
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
fn hook_jerk_pulls_hard_with_cooldown() {
    let mut w = hook_ceiling(with(Ability::HookJerk));
    let held = jerk(&mut w);
    let c = core(&w);
    assert!(c.triggered_events & events::HOOK_JERK != 0);
    assert!(
        c.vel.y < -Tuning::default().hook_drag_speed,
        "schneller als normaler Hook-Zug: {:?}",
        c.vel
    );
    // holding the key does not trigger again, a new press only after the cooldown
    w.step(&[held]);
    assert_eq!(core(&w).triggered_events & events::HOOK_JERK, 0);
    let released = PlayerInput {
        ability: false,
        ..held
    };
    w.step(&[released]);
    w.step(&[held]);
    assert_eq!(
        core(&w).triggered_events & events::HOOK_JERK,
        0,
        "Abklingzeit"
    );
}

/// The jerk pulls noticeably faster than the normal hook (playtest 2026-10-06, A-28):
/// same distance to the ceiling in clearly fewer ticks.
#[test]
fn hook_jerk_reaches_the_hook_point_much_faster() {
    let ticks_to_ceiling = |press: bool| {
        let mut w = hook_ceiling(with(Ability::HookJerk));
        let start = core(&w).pos.y;
        let hold = PlayerInput {
            hook: true,
            ability: press,
            target_x: 0,
            target_y: -100,
            ..PlayerInput::default()
        };
        (1..200)
            .find(|_| {
                w.step(&[hold]);
                start - core(&w).pos.y > 160.0
            })
            .expect("kommt oben an")
    };
    let (normal, jerk) = (ticks_to_ceiling(false), ticks_to_ceiling(true));
    assert!(
        jerk * 3 <= normal * 2,
        "Ruck {jerk} Ticks statt {normal} – mindestens ein Drittel schneller"
    );
}

#[test]
fn hook_jerk_needs_ability() {
    let mut w = hook_ceiling(Abilities::NONE);
    let before = hook_ceiling(Abilities::NONE);
    jerk(&mut w);
    let mut plain = before;
    plain.step(&[PlayerInput {
        hook: true,
        target_x: 0,
        target_y: -100,
        ..PlayerInput::default()
    }]);
    assert_eq!(core(&w).vel, core(&plain).vel, "ohne Fähigkeit wirkungslos");
}

// ---------------------------------------------------------------- Stomp

#[test]
fn stomp_breaks_crumble_floor_and_stays_broken() {
    // crumbling floor island in row 20, air below
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
    // Elora falls through the hole onto the floor
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

// ---------------------------------------------------------------- Ice grip

/// Climbing wall in column 40 (rows 5..28); figure jumps against it from the left.
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
    // after the grip time (1 s) the figure falls normally
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
    // double jump still possible afterwards
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

// ---------------------------------------------------------------- Gliding

/// Jump, double jump, then hold jump.
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
    // ground jump held until past the apex: no gliding
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
    // hook blossom two tiles above Elora, ceiling far above
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
    // figures walk through, it is not solid
    assert!(!Tile::HookPoint.is_solid());
    // withered (only at the guardian, see creatures.rs): columns alternate
    let mut c = w.collision.clone();
    c.hook_wilt = Some(true);
    assert!(!c.hook_point_active(10) && c.hook_point_active(11));
    c.hook_wilt = Some(false);
    assert!(c.hook_point_active(10) && !c.hook_point_active(11));
}

/// Jerk spot (R2-M2.1, M2.1.5): stone shaft, a hook blossom 12 tiles above the
/// floor and close to the right wall, on the right a stone tower 9 tiles above the blossom
/// (with a passage at the bottom).
/// Without the hook jerk Elora cannot get up, with it she can. The maps build the spot
/// the same way (`apps/elora-client/src/editor/chapter1.rs`, `jerk_gate`).
fn jerk_gate_world() -> (elora_sim::World, f32) {
    use elora_sim::{Collision, Tile, Tuning, World};
    let (w, h) = (24, 40);
    let floor = h - 2;
    let mut t = vec![Tile::Air; w * h];
    for x in 0..w {
        for y in floor..h {
            t[y * w + x] = Tile::Solid;
        }
    }
    // shaft: walls x = 7 and x = 13 (stone), ledge on the right from x = 13
    let flower_y = floor - 12;
    let ledge_y = flower_y - 9;
    for y in ledge_y..floor - 3 {
        t[y * w + 7] = Tile::Unhookable;
    }
    // tower with a passage at the bottom (3 tiles), as in the maps
    for y in ledge_y..floor - 3 {
        for x in 13..20 {
            t[y * w + x] = Tile::Unhookable;
        }
    }
    t[flower_y * w + 12] = Tile::HookPoint;
    let world = World::new(Tuning::default(), Collision::new(w, h, t));
    (world, ledge_y as f32 * 32.0)
}

/// Does Elora reach the ledge with these timings?
fn jerk_gate_try(jerk: bool, release: u32, jerk_at: u32, jump_at: u32, right_at: u32) -> bool {
    use elora_sim::{Abilities, PlayerInput, Vec2};
    let (mut w, ledge_top) = jerk_gate_world();
    let i = w.join();
    w.spawn_character(i, Vec2::new(10.0 * 32.0 + 16.0, 38.0 * 32.0 - 15.0));
    if jerk {
        w.set_abilities(i, Abilities::ALL);
    }
    for _ in 0..10 {
        w.step(&[PlayerInput::default()]);
    }
    for t in 0..160u32 {
        let c = &w.character(i).expect("lebt").core;
        // stands (feet above the edge) above the ledge
        if c.pos.y < ledge_top - 14.0 && c.pos.x > 13.0 * 32.0 + 8.0 && c.pos.x < 20.0 * 32.0 {
            return true;
        }
        let input = PlayerInput {
            target_x: 60,
            target_y: -300,
            hook: t < release,
            ability: jerk && t == jerk_at,
            jump: t == jump_at || t == 0,
            direction: i8::from(t >= right_at),
            ..PlayerInput::default()
        };
        w.step(&[input]);
    }
    false
}

#[test]
fn jerk_gate_needs_the_hook_jerk() {
    let reach = |jerk: bool| {
        (5..60).any(|release| {
            let jerks: Vec<u32> = if jerk {
                (1..release).step_by(2).collect()
            } else {
                vec![0]
            };
            jerks.into_iter().any(|jerk_at| {
                (10..100).step_by(3).any(|jump_at| {
                    (0..110)
                        .step_by(6)
                        .any(|right_at| jerk_gate_try(jerk, release, jerk_at, jump_at, right_at))
                })
            })
        })
    };
    assert!(!reach(false), "ohne Hook-Ruck erreichbar");
    assert!(reach(true), "mit Hook-Ruck nicht erreichbar");
}

/// Frostspitzen climbing chimney (R2-M2.4): two climbing walls with three tiles of air
/// between them, 26 rows high – alternating ice grip and wall jumps Elora reaches the top,
/// without them she does not.
#[test]
fn tall_chimney_can_be_climbed_with_grip_only() {
    let climb = |abilities: Abilities| {
        let mut w = world(|t| {
            for y in 2..FLOOR {
                set(t, 20, y, Tile::Climb);
                set(t, 24, y, Tile::Climb);
            }
            // top right a ledge behind the right wall
            for x in 25..40 {
                set(t, x, 1, Tile::Solid);
            }
        });
        spawn(&mut w, standing(22, FLOOR), abilities);
        run(&mut w, PlayerInput::default(), 10);
        let mut toward: i8 = 1;
        let mut held = false;
        let mut best = f32::MAX;
        for _ in 0..1500 {
            let c = core(&w);
            best = best.min(c.pos.y);
            // at the wall: jump off and steer to the other wall; otherwise towards the wall
            let jump = if c.grip != 0 && !held {
                toward = -c.grip;
                true
            } else {
                c.vel.y >= 0.0 && c.is_grounded(&w.collision) && !held
            };
            held = jump;
            run(&mut w, input(toward, jump, false), 1);
        }
        best
    };
    let top = 4.0 * 32.0;
    assert!(
        climb(with(Ability::Grip)) < top,
        "mit Eisgriff oben angekommen"
    );
    assert!(climb(Abilities::NONE) > 12.0 * 32.0, "ohne Eisgriff nicht");
}

/// Strengthened ice grip (R2-M2.4, D-M24-03, A-42): whoever presses towards the wall pulls
/// themselves up until the (doubled) grip time is over; without the boost Elora slowly slides down.
#[test]
fn strong_grip_pulls_elora_up_the_wall() {
    let height_after = |climb: f32| {
        let mut w = world(|t| (5..FLOOR).for_each(|y| set(t, 40, y, Tile::Climb)));
        w.tuning.grip_climb = climb;
        w.tuning.grip_time *= 2;
        spawn(&mut w, standing(37, FLOOR), with(Ability::Grip));
        run(&mut w, PlayerInput::default(), 10);
        run(&mut w, input(1, true, false), 1);
        // to the wall and hold on to it
        let mut gripped_at = None;
        let mut highest = f32::MAX;
        for _ in 0..200 {
            run(&mut w, input(1, false, false), 1);
            if core(&w).grip != 0 {
                gripped_at.get_or_insert(core(&w).pos.y);
                highest = highest.min(core(&w).pos.y);
            }
        }
        (gripped_at.expect("haftet"), highest)
    };
    let (start, end) = height_after(1.6);
    assert!(end < start - 100.0, "zieht sich hinauf: {start} → {end}");
    let (start, end) = height_after(0.0);
    assert!(
        end >= start - 1.0,
        "ohne Stärkung geht es nicht hinauf: {start} → {end}"
    );
}
