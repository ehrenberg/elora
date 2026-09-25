//! Determinismus-Golden-Tests (M1.6).
//!
//! Jede Aufzeichnung in `tests/recordings/*.erec.toml` wird abgespielt; das
//! Zustandsprotokoll muss bit-genau der Datei `<name>.golden` daneben entsprechen.
//! Zusätzlich gibt es ein fest programmiertes Szenario (`scripted`).
//!
//! Golden-Dateien neu schreiben (nach bewusster Physik-Änderung):
//! `ELORA_BLESS=1 cargo nextest run -p elora-sim --all-features`

use std::path::{Path, PathBuf};

use elora_sim::replay::Recording;
use elora_sim::{Collision, PlayerInput, Tile, Tuning, Vec2};

fn recordings_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recordings")
}

fn bless() -> bool {
    std::env::var_os("ELORA_BLESS").is_some()
}

/// Vergleicht `actual` mit der Golden-Datei (oder schreibt sie bei `ELORA_BLESS`).
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

/// Festes Szenario: Laufen, Springen, Doppelsprung, Hook an Decke und Wand.
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
    let mut rec = Recording::new(&col, Vec2::new(200.0, 800.0), Tuning::default());
    for t in 0..1500_i32 {
        rec.push(&PlayerInput {
            direction: [1, 1, 0, -1, -1, 0][usize::try_from(t / 29 % 6).unwrap()],
            target_x: (t * 37 % 400) - 200,
            target_y: -((t * 13) % 300) - 1,
            jump: t % 23 < 5 || t % 71 == 3,
            hook: t % 60 > 15,
        });
    }
    rec
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
