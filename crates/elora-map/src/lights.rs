//! Light sources of the night (R2-M2.5, M2.5.1) from `assets/map/lights.toml`: which
//! decoration glows and how far, and Elora's own light.

use std::collections::HashMap;
use std::sync::OnceLock;

use elora_sim::Vec2;

use crate::Map;
use crate::look::Art;

const SOURCE: &str = include_str!("../../../assets/map/lights.toml");

#[derive(serde::Deserialize)]
struct File {
    elora: f32,
    #[serde(default)]
    decor: HashMap<String, f32>,
}

fn file() -> &'static File {
    static FILE: OnceLock<File> = OnceLock::new();
    FILE.get_or_init(|| toml::from_str(SOURCE).expect("assets/map/lights.toml is valid"))
}

/// Radius of Elora's own light in world units.
pub fn elora_radius() -> f32 {
    file().elora
}

/// Light radius of a built-in decoration (`None`: it does not glow).
pub fn decor_radius(name: &str) -> Option<f32> {
    file().decor.get(name).copied()
}

/// All glowing decoration of a map: centre and radius in world units.
pub fn map_lights(map: &Map) -> Vec<(Vec2, f32)> {
    map.decor_back
        .iter()
        .chain(&map.decor_front)
        .filter_map(|d| match &d.art {
            Art::Builtin(name) => decor_radius(name).map(|r| (d.pos, r * d.scale.max(0.1))),
            Art::Image(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lights_are_read_and_name_existing_decoration() {
        assert!(elora_radius() >= 128.0);
        assert!(decor_radius("glow_mushrooms").is_some());
        assert!(decor_radius("rock-1").is_none());
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/map/decor");
        for name in file().decor.keys() {
            let svg = std::path::Path::new(dir).join(format!("{name}.svg"));
            assert!(svg.is_file(), "light for unknown decoration `{name}`");
        }
    }

    #[test]
    fn glowing_decoration_of_a_map() {
        let mut map = Map::new("t", 8, 8);
        map.decor_back.push(crate::look::Decor::new(
            Art::Builtin("lantern".into()),
            Vec2::new(40.0, 50.0),
        ));
        map.decor_front.push(crate::look::Decor::new(
            Art::Builtin("rock-1".into()),
            Vec2::ZERO,
        ));
        assert_eq!(map_lights(&map), vec![(Vec2::new(40.0, 50.0), 170.0)]);
    }
}
