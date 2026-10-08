//! New tile kinds (M6.1, E-137, E-140, E-141): platform, ice, jump pad, booster.

use elora_sim::{BeltDir, Collision, JumpDir, PlayerInput, Tile, Tuning, Vec2, World};

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

/// Figure stands on row `row` (bottom 1 unit above the top edge), column `tx`.
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

fn pos(w: &World) -> Vec2 {
    w.character(0).expect("lebt").core.pos
}

fn spawn(w: &mut World, at: Vec2) {
    let i = w.join();
    w.spawn_character(i, at);
}

#[test]
fn platform_carries_from_above_and_lets_through_from_below() {
    // platform in row 20, columns 10..20
    let mut w = world(|t| (10..20).for_each(|x| set(t, x, 20, Tile::Platform)));
    spawn(&mut w, standing(14, 20) - Vec2::new(0.0, 100.0));
    run(&mut w, PlayerInput::default(), 80);
    let p = pos(&w);
    assert!(
        (p.y - (20.0 * 32.0 - 14.0)).abs() < 1.5,
        "lands on the platform: {p:?}"
    );
    assert!(w.character(0).unwrap().core.is_grounded(&w.collision));

    // jump through from below
    let mut w = world(|t| (10..20).for_each(|x| set(t, x, 25, Tile::Platform)));
    spawn(&mut w, standing(14, FLOOR));
    run(&mut w, input(0, true, false), 1);
    let mut highest = f32::MAX;
    for _ in 0..40 {
        run(&mut w, input(0, false, false), 1);
        highest = highest.min(pos(&w).y);
    }
    assert!(
        highest < 25.0 * 32.0 - 14.0,
        "jump passes up through the platform: {highest}"
    );
    run(&mut w, PlayerInput::default(), 80);
    assert!(
        (pos(&w).y - (25.0 * 32.0 - 14.0)).abs() < 1.5,
        "then lands on top of it"
    );
}

#[test]
fn down_drops_through_platform() {
    let mut w = world(|t| (10..20).for_each(|x| set(t, x, 20, Tile::Platform)));
    spawn(&mut w, standing(14, 20));
    run(&mut w, PlayerInput::default(), 20);
    let on_top = pos(&w).y;
    run(&mut w, input(0, false, true), 30);
    assert!(pos(&w).y > on_top + 32.0, "falls through with \"down\"");
    run(&mut w, PlayerInput::default(), 80);
    assert!(
        (pos(&w).y - (FLOOR as f32 * 32.0 - 14.0)).abs() < 1.5,
        "lands on the floor below"
    );
}

#[test]
fn hook_passes_through_platform() {
    let mut w = world(|t| {
        // ceiling in range (400), platform in between
        (5..40).for_each(|x| set(t, x, 23, Tile::Platform));
        (5..40).for_each(|x| set(t, x, 19, Tile::Solid));
    });
    spawn(&mut w, standing(14, FLOOR));
    run(&mut w, PlayerInput::default(), 5);
    let hook = PlayerInput {
        hook: true,
        target_x: 0,
        target_y: -100,
        ..PlayerInput::default()
    };
    run(&mut w, hook, 10);
    let core = &w.character(0).unwrap().core;
    assert_eq!(core.hook_state, elora_sim::HookState::Grabbed);
    assert!(
        core.hook_pos.y <= 20.0 * 32.0 + 1.0,
        "hook hangs on the ceiling, not on the platform: {:?}",
        core.hook_pos
    );
}

/// Distance the figure still slides after releasing the walking direction.
fn slide_after_run(surface: Tile) -> f32 {
    let mut w = world(|t| (2..58).for_each(|x| set(t, x, FLOOR, surface)));
    spawn(&mut w, standing(5, FLOOR));
    run(&mut w, input(1, false, false), 60);
    let start = pos(&w).x;
    run(&mut w, PlayerInput::default(), 100);
    pos(&w).x - start
}

#[test]
fn ice_slides_much_further() {
    let normal = slide_after_run(Tile::Solid);
    let ice = slide_after_run(Tile::Ice);
    assert!(normal < 25.0, "normal short: {normal}");
    assert!(ice > 10.0 * normal, "ice slides far: {ice} vs {normal}");
}

#[test]
fn jump_pad_throws_up_and_sideways() {
    let tuning = Tuning::default();
    for (dir, sign) in [
        (JumpDir::Up, 0.0),
        (JumpDir::UpRight, 1.0),
        (JumpDir::UpLeft, -1.0),
    ] {
        let mut w = world(|t| set(t, 30, FLOOR, Tile::JumpPad(dir)));
        spawn(&mut w, standing(30, FLOOR));
        run(&mut w, PlayerInput::default(), 3);
        let core = &w.character(0).unwrap().core;
        let v = core.vel;
        assert!(
            v.y < -tuning.jump_pad_force * 0.6,
            "{dir:?}: thrown upwards: {v:?}"
        );
        assert!(
            v.x * sign >= 0.0 && (sign == 0.0) == (v.x.abs() < 0.5),
            "{dir:?}: direction {v:?}"
        );
        // higher than a normal jump
        let mut highest = f32::MAX;
        for _ in 0..60 {
            run(&mut w, PlayerInput::default(), 1);
            highest = highest.min(pos(&w).y);
        }
        if dir == JumpDir::Up {
            let height = FLOOR as f32 * 32.0 - 14.0 - highest;
            assert!(
                height > 9.0 * 32.0,
                "jump pad throws about 12 tiles high: {height}"
            );
        }
    }
}

#[test]
fn conveyor_carries_standing_figure() {
    let tuning = Tuning::default();
    let mut w = world(|t| (10..50).for_each(|x| set(t, x, FLOOR, Tile::Conveyor(BeltDir::Right))));
    spawn(&mut w, standing(15, FLOOR));
    run(&mut w, PlayerInput::default(), 5);
    let start = pos(&w).x;
    run(&mut w, PlayerInput::default(), 50);
    let moved = pos(&w).x - start;
    assert!(
        (moved - 50.0 * tuning.conveyor_speed).abs() < 8.0,
        "conveyor carries: {moved}"
    );
    // walking against the direction makes slower progress than without a belt
    let mut w = world(|t| (10..50).for_each(|x| set(t, x, FLOOR, Tile::Conveyor(BeltDir::Right))));
    spawn(&mut w, standing(40, FLOOR));
    run(&mut w, input(-1, false, false), 30);
    let start = pos(&w).x;
    run(&mut w, input(-1, false, false), 30);
    let against = start - pos(&w).x;
    assert!(
        against > 0.0 && against < 30.0 * tuning.ground_control_speed - 50.0,
        "against the belt: {against}"
    );
}

#[test]
fn old_tiles_unchanged_and_chars_roundtrip() {
    for c in ['.', '#', '%', '^', '=', '~', '!', '\\', '/', '<', '>'] {
        assert_eq!(Tile::from_char(c).map(Tile::to_char), Some(c));
    }
    assert!(!Tile::Platform.is_solid());
    assert!(Tile::Ice.is_solid() && Tile::JumpPad(JumpDir::Up).is_solid());
}

/// Playtest 2026-10-08: from a platform onto a solid tile at the same height the figure
/// used to stop at the edge (it stood 1 unit lower on platforms).
#[test]
fn walks_from_platform_onto_ground_at_the_same_height() {
    for dir in [1i8, -1] {
        let mut w = world(|t| {
            for x in 10..20 {
                set(t, x, 20, Tile::Platform);
            }
            for x in 20..30 {
                set(t, x, 20, Tile::Solid);
            }
        });
        spawn(
            &mut w,
            if dir > 0 {
                standing(14, 20)
            } else {
                standing(25, 20)
            },
        );
        run(&mut w, input(0, false, false), 20);
        let start = pos(&w);
        run(&mut w, input(dir, false, false), 35);
        let moved = (pos(&w).x - start.x) * f32::from(dir);
        assert!(moved > 250.0, "direction {dir}: only {moved} units");
        assert!(
            (pos(&w).y - start.y).abs() < 0.5,
            "same height: {:?}",
            pos(&w)
        );
    }
}
