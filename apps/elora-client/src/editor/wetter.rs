//! Wetter-Testkarte (R2-W1): Wiese mit Hügeln, einem Dach, einer Grube, Stegen und Bäumen,
//! um alle Wetter zu sehen (im Debug-Panel umschaltbar) – Partikel auf Oberflächen, unter
//! Dächern, vor und hinter der Spielfläche.
//!
//! `cargo test -p elora-client --bin elora write_weather_test_map -- --ignored` schreibt
//! `maps/wetter-test.emap`.

#![allow(clippy::cast_precision_loss)]

use elora_map::{Map, Weather, WeatherKind};

use super::prolog::{Grid, decor, finish};
use super::release;

/// 140 × 50, Start links; Wetter: Regen.
pub fn weather_test() -> Map {
    let (w, h) = (140, 50);
    let mut g = Grid::new(w, h, 40);
    g.ground(0, 30, 38);
    g.ground(31, 50, 36);
    g.ground(51, 70, 40);
    // Dach (Schutz vor Regen, Schatten)
    g.fill((58, 68), (31, 32), '#');
    g.ground(71, 80, 44);
    g.ground(81, 110, 38);
    g.fill((88, 94), (31, 31), '=');
    g.fill((98, 104), (27, 27), '=');
    g.ground(111, w - 1, 34);
    let mut m = g.map("Wetter-Test");
    m.entities.push(elora_map::Entity {
        kind: elora_map::EntityKind::Spawn,
        tx: 6,
        ty: 37,
    });
    m.decor_back = vec![
        decor("tree-round", 12.0, 38),
        decor("tree-pine", 40.0, 36),
        decor("haus-oma", 63.0, 40),
        decor("tree-round", 100.0, 38),
        decor("fence", 120.0, 34),
    ];
    m.decor_front = vec![decor("bush-1", 24.0, 38), decor("grass-1", 85.0, 38)];
    let mut m = finish(m, &release::THEMES[0]);
    m.name = "Wetter-Test".into();
    m.weather = Weather {
        kind: WeatherKind::Rain,
        intensity: 0.7,
        wind: 0.3,
    };
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped() -> String {
        format!(
            "{}/../../maps/wetter-test.{}",
            env!("CARGO_MANIFEST_DIR"),
            elora_map::EXTENSION
        )
    }

    #[test]
    fn weather_test_map_is_current_and_rainy() {
        let file = std::fs::read(shipped()).expect("Karte vorhanden");
        let map = elora_map::decode(&file).expect("gültig");
        assert_eq!(
            map,
            weather_test(),
            "veraltet – write_weather_test_map -- --ignored"
        );
        assert_eq!(map.weather.kind, WeatherKind::Rain);
    }

    /// Varianten zum Ansehen: `… write_weather_variants -- --ignored` → `target/wetter-<art>.emap`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn write_weather_variants() {
        for kind in WeatherKind::ALL {
            let mut m = weather_test();
            m.weather = Weather {
                kind,
                intensity: 0.8,
                wind: 0.4,
            };
            let path = format!(
                "{}/../../target/wetter-{}.{}",
                env!("CARGO_MANIFEST_DIR"),
                kind.key(),
                elora_map::EXTENSION
            );
            m.save(std::path::Path::new(&path)).unwrap();
        }
    }

    #[test]
    #[ignore = "schreibt maps/wetter-test.emap"]
    fn write_weather_test_map() {
        weather_test()
            .save(std::path::Path::new(&shipped()))
            .unwrap();
    }
}
