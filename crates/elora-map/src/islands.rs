//! Star islands and updrafts from the zones of a map (R2-M2.5, M2.5.2, E-364).
//!
//! - A zone `island…` is a floating island: the zone is its body, the top edge the walkable
//!   surface. With a second zone `<id>-to` it drifts back and forth between its place and the
//!   top-left corner of that zone; with `sink` in the id it sinks while someone stands on it.
//! - A zone `updraft…` lifts gliding figures.
//! - A zone `note-<pitch>…` is a note stone (M2.5.3, pitch `c d e f g a b`, `2` for the octave
//!   above, see [`elora_sim::notes::pitch`]); zones `<stone id>-bridge…` are the star bridges it
//!   lights.
//! - A zone `lantern…` is a star lantern (M2.5.5): touching it lights it for a while.

use elora_sim::{Island, Lantern, NoteStone, StarBridge, Vec2};

use crate::Map;
use crate::adventure::ObjectKind;

/// Start of the id of an island zone.
pub const ISLAND: &str = "island";
/// End of the id of the zone that marks where an island drifts to.
pub const TARGET: &str = "-to";
/// Start of the id of an updraft zone.
pub const UPDRAFT: &str = "updraft";
/// Start of the id of a note stone zone.
pub const NOTE: &str = "note-";
/// Marks a star bridge zone: `<stone id>-bridge…`.
pub const BRIDGE: &str = "-bridge";
/// Start of the id of a star lantern zone.
pub const LANTERN: &str = "lantern";

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

/// The star lanterns of a map (M2.5.5), unlit.
pub fn map_lanterns(map: &Map) -> Vec<Lantern> {
    zones(map)
        .filter(|(id, ..)| id.starts_with(LANTERN))
        .map(|(_, pos, size)| Lantern::new(pos, size))
        .collect()
}

/// The updraft areas of a map: top-left corner and size.
pub fn map_updrafts(map: &Map) -> Vec<(Vec2, Vec2)> {
    zones(map)
        .filter(|(id, ..)| id.starts_with(UPDRAFT))
        .map(|(_, pos, size)| (pos, size))
        .collect()
}

/// Note stones and their star bridges of a map. Stones with an unknown pitch are left out
/// (the adventure check reports them).
pub fn map_notes(map: &Map) -> (Vec<NoteStone>, Vec<StarBridge>) {
    let bridge_zones: Vec<(&str, Vec2, Vec2)> = zones(map)
        .filter(|(id, ..)| id.starts_with(NOTE) && id.contains(BRIDGE))
        .collect();
    let stones = zones(map)
        .filter(|(id, ..)| id.starts_with(NOTE) && !id.contains(BRIDGE))
        .filter_map(|(id, pos, size)| {
            let note = elora_sim::notes::pitch(id)?;
            let prefix = format!("{id}{BRIDGE}");
            let bridges = bridge_zones
                .iter()
                .enumerate()
                .filter(|(_, (b, ..))| b.starts_with(&prefix))
                .map(|(i, _)| i)
                .collect();
            Some(NoteStone::new(pos, size, note, bridges))
        })
        .collect();
    let bridges = bridge_zones
        .iter()
        .map(|&(_, pos, size)| StarBridge::new(pos, size))
        .collect();
    (stones, bridges)
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

    #[test]
    fn zones_become_note_stones_and_bridges() {
        let mut map = Map::new("t", 40, 20);
        map.adventure.objects = vec![
            zone("note-e-1-bridge", 300.0, 320.0),
            zone("note-e-1", 64.0, 320.0),
            zone("note-c2", 64.0, 100.0),
            zone("note-e-1-bridge-2", 500.0, 320.0),
            zone("note-x", 0.0, 0.0),
        ];
        let (stones, bridges) = map_notes(&map);
        assert_eq!(stones.len(), 2, "unknown pitch left out");
        assert_eq!((stones[0].note, stones[0].bridges.clone()), (4, vec![0, 1]));
        assert_eq!((stones[1].note, stones[1].bridges.len()), (12, 0));
        assert_eq!(bridges.len(), 2);
        assert_eq!(bridges[1].at, Vec2::new(500.0, 320.0));
    }

    #[test]
    fn zones_become_lanterns() {
        let mut map = Map::new("t", 40, 20);
        map.adventure.objects = vec![
            zone("lantern-1", 64.0, 320.0),
            zone("lantern-high", 512.0, 96.0),
            zone("island-1", 0.0, 0.0),
        ];
        let lanterns = map_lanterns(&map);
        assert_eq!(lanterns.len(), 2);
        assert_eq!(lanterns[1].at, Vec2::new(512.0, 96.0));
        assert!(lanterns.iter().all(|l| l.lit_until == 0));
        assert_eq!(map.world(elora_sim::Tuning::default()).lanterns.len(), 2);
    }
}
