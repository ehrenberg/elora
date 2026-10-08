//! Determinism golden tests (M1.6).
//!
//! Every recording in `tests/recordings/*.erec.toml` is played back; the
//! state log must match the `<name>.golden` file next to it bit for bit.
//! In addition there is a hard-coded scenario (`scripted`).
//!
//! Rewrite golden files (after a deliberate physics change):
//! `ELORA_BLESS=1 cargo nextest run -p elora-sim --all-features`

use std::path::{Path, PathBuf};

use elora_sim::replay::Recording;
use elora_sim::{
    Collision, DummyPattern, PickupKind, PlayerInput, Tile, Tuning, Vec2, Weapon, World,
};

fn recordings_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recordings")
}

fn bless() -> bool {
    std::env::var_os("ELORA_BLESS").is_some()
}

/// Compares `actual` with the golden file (or writes it with `ELORA_BLESS`).
fn check_golden(name: &str, actual: &str) {
    let path = recordings_dir().join(format!("{name}.golden"));
    if bless() {
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{} fehlt – mit ELORA_BLESS=1 erzeugen", path.display());
    });
    if expected != actual {
        // For comparison: put the actual log next to the golden file
        let _ = std::fs::write(path.with_extension("actual"), actual);
        let first = expected
            .lines()
            .zip(actual.lines())
            .find(|(e, a)| e != a)
            .map_or_else(String::new, |(e, a)| {
                format!("\nerwartet: {e}\nerhalten: {a}")
            });
        panic!("{name}: Simulation weicht von der Golden-Datei ab{first}");
    }
}

/// Fixed scenario: walking, jumping, double jump, hook on ceiling and wall.
fn scripted() -> Recording {
    let mut rows = vec![format!("#{}#", ".".repeat(58)); 30];
    rows[0] = "#".repeat(60);
    rows[28] = format!("#{}{}#", ".".repeat(40), "^".repeat(18));
    rows[29] = "#".repeat(60);
    rows[10].replace_range(20..30, "%%%%%%%%%%");
    rows[20].replace_range(5..15, "##########");
    let tiles: Vec<Tile> = rows
        .iter()
        .flat_map(|r| {
            r.chars().map(|c| match c {
                '#' => Tile::Solid,
                '%' => Tile::Unhookable,
                '^' => Tile::Death,
                _ => Tile::Air,
            })
        })
        .collect();
    let col = Collision::new(60, 30, tiles);
    let mut world = World::new(Tuning::default(), col);
    world.spawn_points.push(Vec2::new(200.0, 800.0));
    world.spawn(Vec2::new(200.0, 800.0));
    let mut rec = Recording::new(&world);
    for t in 0..1500_i32 {
        rec.push(&PlayerInput {
            direction: [1, 1, 0, -1, -1, 0][usize::try_from(t / 29 % 6).unwrap()],
            target_x: (t * 37 % 400) - 200,
            target_y: -((t * 13) % 300) - 1,
            jump: t % 23 < 5 || t % 71 == 3,
            hook: t % 60 > 15,
            ..PlayerInput::default()
        });
    }
    rec
}

/// Fixed combat scenario (M2.9): collect pickups, switch weapons, shoot at all
/// dummy kinds, rocket jumps, death and respawn.
fn scripted_combat() -> Recording {
    let (w, h) = (50, 20);
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
    for x in 30..40 {
        tiles[12 * w + x] = Tile::Solid;
    }
    let tile = |x: i32, y: i32| Vec2::new(x as f32 * 32.0 + 16.0, y as f32 * 32.0 + 16.0);
    let mut world = World::new(Tuning::default(), Collision::new(w, h, tiles));
    world.spawn_points.push(tile(3, 17));
    world.spawn_points.push(tile(46, 17));
    // Elora first (slot 0), pickups right next to her, dummies further to the right
    world.spawn(tile(3, 17));
    world.add_pickup(PickupKind::Weapon(Weapon::Grenade), tile(5, 17));
    world.add_pickup(PickupKind::Weapon(Weapon::Laser), tile(6, 17));
    world.add_pickup(PickupKind::Armor, tile(7, 17));
    world.add_dummy(tile(18, 17), DummyPattern::Stand);
    world.add_dummy(tile(24, 17), DummyPattern::Walk);
    world.add_dummy(tile(35, 11), DummyPattern::Jump);
    world.add_dummy(tile(42, 17), DummyPattern::WalkJump);
    let mut rec = Recording::new(&world);
    let mut fire = 0u8;
    for t in 0..3000_i32 {
        let phase = t / 250;
        // fire key: press every 20 ticks, hold for 10 ticks
        if t % 10 == 0 {
            fire = fire.wrapping_add(1) & 0x3f;
        }
        // phases: 0 fetch weapons, 1 grenade, 2 laser, 3 rocket jumps, 4 hammer while running,
        // then laser/grenade alternating with hook
        let weapon: u8 = match phase {
            0 | 2 | 5 | 7 | 9 | 11 => 3,
            1 | 3 | 6 | 8 | 10 => 2,
            _ => 1,
        };
        let rocket = phase == 3;
        rec.push(&PlayerInput {
            direction: match phase {
                0 => i8::from(t < 60),
                4 => 1,
                5 | 9 => -1,
                _ => 0,
            },
            target_x: if rocket { 0 } else { 400 - (t * 7 % 80) },
            target_y: if rocket { 100 } else { -((t * 11) % 60) - 10 },
            jump: if rocket {
                t % 60 < 2
            } else {
                phase > 0 && t % 97 < 3
            },
            hook: phase >= 6 && t % 70 > 40,
            fire: if phase == 0 { 0 } else { fire },
            wanted_weapon: if t % 250 == 1 { weapon } else { 0 },
            ..PlayerInput::default()
        });
    }
    rec
}

#[test]
fn scripted_combat_matches_golden() {
    let log = scripted_combat().replay().unwrap();
    assert!(log.contains("tot seit"), "Szenario soll Tode enthalten");
    check_golden("scripted-combat", &log);
}

#[test]
fn scripted_scenario_matches_golden() {
    let log = scripted().replay().unwrap();
    check_golden("scripted", &log);
}

#[test]
fn recordings_match_golden() {
    let Ok(entries) = std::fs::read_dir(recordings_dir()) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().ends_with(".erec.toml"))
        .collect();
    paths.sort();
    for path in paths {
        let name = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .trim_end_matches(".erec.toml")
            .to_string();
        let src = std::fs::read_to_string(&path).unwrap();
        let rec = Recording::from_toml(&src).unwrap_or_else(|e| panic!("{name}: {e}"));
        check_golden(
            &name,
            &rec.replay().unwrap_or_else(|e| panic!("{name}: {e}")),
        );
    }
}

#[test]
fn replay_is_reproducible() {
    let rec = scripted();
    assert_eq!(rec.replay().unwrap(), rec.replay().unwrap());
}
