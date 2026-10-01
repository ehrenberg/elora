//! Neue Tile-Arten (M6.1, E-137, E-140, E-141): Plattform, Eis, Sprungfeld, Beschleuniger.

use elora_sim::{BeltDir, Collision, JumpDir, PlayerInput, Tile, Tuning, Vec2, World};

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

/// Figur steht auf Zeile `row` (Unterkante 1 Einheit über der Oberkante), Spalte `tx`.
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
    // Plattform in Zeile 20, Spalten 10..20
    let mut w = world(|t| (10..20).for_each(|x| set(t, x, 20, Tile::Platform)));
    spawn(&mut w, standing(14, 20) - Vec2::new(0.0, 100.0));
    run(&mut w, PlayerInput::default(), 80);
    let p = pos(&w);
    assert!(
        (p.y - (20.0 * 32.0 - 14.0)).abs() < 1.5,
        "landet auf der Plattform: {p:?}"
    );
    assert!(w.character(0).unwrap().core.is_grounded(&w.collision));

    // von unten durchspringen
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
        "Sprung geht durch die Plattform nach oben: {highest}"
    );
    run(&mut w, PlayerInput::default(), 80);
    assert!(
        (pos(&w).y - (25.0 * 32.0 - 14.0)).abs() < 1.5,
        "landet danach oben auf ihr"
    );
}

#[test]
fn down_drops_through_platform() {
    let mut w = world(|t| (10..20).for_each(|x| set(t, x, 20, Tile::Platform)));
    spawn(&mut w, standing(14, 20));
    run(&mut w, PlayerInput::default(), 20);
    let on_top = pos(&w).y;
    run(&mut w, input(0, false, true), 30);
    assert!(pos(&w).y > on_top + 32.0, "fällt mit „Runter“ hindurch");
    run(&mut w, PlayerInput::default(), 80);
    assert!(
        (pos(&w).y - (FLOOR as f32 * 32.0 - 14.0)).abs() < 1.5,
        "landet auf dem Boden darunter"
    );
}

#[test]
fn hook_passes_through_platform() {
    let mut w = world(|t| {
        // Decke in Reichweite (400), Plattform dazwischen
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
        "Hook hängt an der Decke, nicht an der Plattform: {:?}",
        core.hook_pos
    );
}

/// Strecke, die die Figur nach Loslassen der Laufrichtung noch rutscht.
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
    assert!(normal < 25.0, "normal kurz: {normal}");
    assert!(ice > 10.0 * normal, "Eis rutscht weit: {ice} vs {normal}");
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
            "{dir:?}: nach oben geworfen: {v:?}"
        );
        assert!(
            v.x * sign >= 0.0 && (sign == 0.0) == (v.x.abs() < 0.5),
            "{dir:?}: Richtung {v:?}"
        );
        // höher als ein normaler Sprung
        let mut highest = f32::MAX;
        for _ in 0..60 {
            run(&mut w, PlayerInput::default(), 1);
            highest = highest.min(pos(&w).y);
        }
        if dir == JumpDir::Up {
            let height = FLOOR as f32 * 32.0 - 14.0 - highest;
            assert!(
                height > 9.0 * 32.0,
                "Sprungfeld wirft etwa 12 Tiles hoch: {height}"
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
        "Laufband trägt: {moved}"
    );
    // gegen die Richtung laufen kommt langsamer voran als ohne Band
    let mut w = world(|t| (10..50).for_each(|x| set(t, x, FLOOR, Tile::Conveyor(BeltDir::Right))));
    spawn(&mut w, standing(40, FLOOR));
    run(&mut w, input(-1, false, false), 30);
    let start = pos(&w).x;
    run(&mut w, input(-1, false, false), 30);
    let against = start - pos(&w).x;
    assert!(
        against > 0.0 && against < 30.0 * tuning.ground_control_speed - 50.0,
        "gegen das Band: {against}"
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
