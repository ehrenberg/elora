//! Testkarte für die Fähigkeiten (R2-M1, A1.1): `maps/faehigkeiten-test.emap`.
//!
//! Neu schreiben: `cargo test -p elora-map --test abilities_map -- --ignored`

use elora_map::Map;

const W: usize = 80;
const H: usize = 32;

fn layout() -> Vec<String> {
    let mut g = vec![vec!['.'; W]; H];
    let mut fill = |x0: usize, x1: usize, y0: usize, y1: usize, c: char| {
        for row in &mut g[y0..=y1] {
            for cell in &mut row[x0..=x1] {
                *cell = c;
            }
        }
    };
    // Rahmen, Boden
    fill(0, W - 1, 0, 0, '#');
    fill(0, W - 1, 29, H - 1, '#');
    fill(0, 0, 0, H - 1, '#');
    fill(W - 1, W - 1, 0, H - 1, '#');

    // 1 Hook-Ruck: hohe Halle mit Decke
    fill(2, 20, 4, 4, '#');

    // 2 Stampfen: Bröckel-Brücke über einer Kammer mit Herz
    fill(17, 20, 25, 25, '=');
    fill(21, 21, 22, 28, '#');
    fill(31, 31, 22, 28, '#');
    fill(22, 30, 22, 23, ':');

    // 3 Eisgriff: Kamin aus Kletterwänden, oben ein Sims; dazu eine einzelne Wand
    fill(36, 36, 9, 28, '|');
    fill(40, 40, 9, 28, '|');
    fill(41, 55, 8, 8, '#');
    fill(46, 46, 14, 28, '|');

    // 4 Gleiten: vom Sims über die Stachelgrube zur Plattform; darüber nicht hookbar
    fill(41, W - 2, 0, 0, '%');
    fill(56, 75, 28, 28, '^');
    fill(70, 77, 20, 20, '#');

    let mut rows: Vec<String> = g.into_iter().map(|r| r.into_iter().collect()).collect();
    let mut put = |x: usize, y: usize, c: char| rows[y].replace_range(x..=x, &c.to_string());
    put(4, 28, 'S');
    put(26, 28, 'h');
    put(44, 7, 'S');
    rows
}

#[test]
fn layout_is_valid() {
    let rows = layout();
    let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
    let map = Map::from_rows("Fähigkeiten-Test", &refs).expect("gültig");
    assert_eq!((map.width, map.height), (W, H));
}

#[test]
#[ignore = "schreibt maps/faehigkeiten-test.emap"]
fn write_map() {
    let rows = layout();
    let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
    let mut map = Map::from_rows("Fähigkeiten-Test", &refs).expect("gültig");
    map.author = Some("Elora".into());
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../maps/faehigkeiten-test.emap"
    );
    std::fs::write(path, elora_map::encode(&map)).expect("schreiben");
}
