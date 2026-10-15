//! Star islands and updrafts from the zones of a map (R2-M2.5, M2.5.2, E-364).
//!
//! - A zone `island…` is a floating island: the zone is its body, the top edge the walkable
//!   surface. With a second zone `<id>-to` it drifts back and forth between its place and the
//!   top-left corner of that zone; with `sink` in the id it sinks while someone stands on it.
//! - A zone `updraft…` lifts gliding figures.

use elora_sim::{Island, Vec2};

use crate::Map;
use crate::adventure::ObjectKind;

/// Start of the id of an island zone.
pub const ISLAND: &str = "island";
/// End of the id of the zone that marks where an island drifts to.
pub const TARGET: &str = "-to";
/// Start of the id of an updraft zone.
pub const UPDRAFT: &str = "updraft";

fn zones(map: &Map) -> impl Iterator<Item = (&str, Vec2, Vec2)> {
    map.adventure.objects.iter().filter_map(|o| match o.kind {
        ObjectKind::Zone { size } => Some((o.id.as_str(), o.pos, size)),
        _ => None,
    })
}

/// The islands of a map, at the start of their paths.
pub fn map_islands(map: &Map) -> Vec<Island> {
    zones(map)
        .filter(|(id, ..)| id.starts_with(ISLAND) && !id.ends_with(TARGET))
        .map(|(id, pos, size)| {
            let end = zones(map)
                .find(|(other, ..)| other.strip_suffix(TARGET) == Some(id))
                .map_or(pos, |(_, to, _)| to);
            Island::new(pos, end, size, id.contains("sink"))
        })
        .collect()
}

/// The updraft areas of a map: top-left corner and size.
pub fn map_updrafts(map: &Map) -> Vec<(Vec2, Vec2)> {
    zones(map)
        .filter(|(id, ..)| id.starts_with(UPDRAFT))
        .map(|(_, pos, size)| (pos, size))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::Object;

    fn zone(id: &str, x: f32, y: f32) -> Object {
        Object {
            id: id.into(),
            pos: Vec2::new(x, y),
            kind: ObjectKind::Zone {
                size: Vec2::new(128.0, 32.0),
            },
        }
    }

    #[test]
    fn zones_become_islands_and_updrafts() {
        let mut map = Map::new("t", 40, 20);
        map.adventure.objects = vec![
            zone("island-a", 64.0, 320.0),
            zone("island-a-to", 512.0, 320.0),
            zone("island-sink-1", 64.0, 480.0),
            zone("updraft-1", 600.0, 100.0),
            zone("shade", 0.0, 0.0),
        ];
        let islands = map_islands(&map);
        assert_eq!(islands.len(), 2);
        assert_eq!(islands[0].end, Vec2::new(512.0, 320.0));
        assert!(!islands[0].sinks);
        assert_eq!(islands[1].start, islands[1].end);
        assert!(islands[1].sinks);
        assert_eq!(
            map_updrafts(&map),
            vec![(Vec2::new(600.0, 100.0), Vec2::new(128.0, 32.0))]
        );
    }
}
