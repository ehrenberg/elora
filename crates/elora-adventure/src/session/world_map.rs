//! Where Elora is on the world map (R2-WM, E-370): her map section and her place along it.

use elora_sim::{TILE_SIZE, World};

use super::Session;
use crate::data::Axis;

/// Elora's place on the world map.
#[derive(Debug, Clone, PartialEq)]
pub struct MapSpot {
    /// Area id.
    pub area: String,
    /// Map id.
    pub map: String,
    /// Place along the section (0 = the side towards Tauwinkel, 1 = the far end).
    pub along: f32,
    /// Point on the overview (0..1); the area's point when the map has no section yet.
    pub point: [f32; 2],
}

impl Session {
    /// Where Elora is on the world map; `None` outside the areas of the world map.
    pub fn map_spot(&self, world: &World) -> Option<MapSpot> {
        let area = self.content.area_of(&self.map_name)?;
        let pos = world.character(self.player)?.core.pos;
        let ts = TILE_SIZE as f32;
        let (w, h) = (
            (self.map.width as f32 * ts).max(1.0),
            (self.map.height as f32 * ts).max(1.0),
        );
        let section = area.section(&self.map_name);
        let along = match section.map_or(Axis::X, |s| s.axis) {
            Axis::X => pos.x / w,
            Axis::Y => 1.0 - pos.y / h,
        }
        .clamp(0.0, 1.0);
        Some(MapSpot {
            area: area.id.clone(),
            map: self.map_name.clone(),
            along,
            point: section.map_or(area.pos, |s| s.point(along)),
        })
    }
}
