//! Heat bar in hot areas (E-320) and cold bar in cold areas (E-342): one [`TemperatureBar`]
//! implementation, its parameters come from `[temperature]` in `worldmap.toml`.

use elora_map::{Map, ObjectKind, WeatherKind};
use elora_sim::{TILE_SIZE, Tile, Vec2, World};

use super::{Session, inside};
use crate::data::TemperatureBar;

/// A roof (solid tile or platform) this many tiles above Elora gives shelter.
const ROOF_TILES: i32 = 10;

/// Where Elora stands, as far as a temperature bar cares.
struct Surroundings<'a> {
    map: &'a Map,
    pos: Vec2,
    roof: bool,
    arena: bool,
}

impl Surroundings<'_> {
    /// Is Elora in a zone whose id starts with one of `prefixes`?
    fn in_zone(&self, prefixes: &[String]) -> bool {
        self.map.adventure.objects.iter().any(|o| {
            prefixes.iter().any(|p| o.id.starts_with(p.as_str()))
                && matches!(o.kind, ObjectKind::Zone { size } if inside(self.pos, o.pos, size))
        })
    }

    fn weather_in(&self, keys: &[String]) -> bool {
        keys.iter()
            .any(|k| WeatherKind::from_key(k) == Some(self.map.weather.kind))
    }
}

impl TemperatureBar {
    /// One tick for the bar `level` (0..1) with the full state `full`; `pct` is the equipment
    /// bonus in percent (negative = fills more slowly). Returns whether Elora is in the open
    /// (the bar fills).
    fn tick(&self, level: &mut f32, full: &mut bool, at: &Surroundings<'_>, pct: f32) -> bool {
        #[allow(clippy::cast_precision_loss)]
        let step = |ms: u32| 1.0 / elora_sim::tuning::ms_to_ticks(ms).max(1) as f32;
        let open = if at.in_zone(&self.relief_zones) {
            *level -= step(self.relief_ms);
            false
        } else if at.roof
            || at.in_zone(&self.shelter_zones)
            || at.weather_in(&self.shelter_weather)
            || (self.shelter_arenas && at.arena)
        {
            *level -= step(self.shelter_ms);
            false
        } else {
            let faster = if at.weather_in(&self.faster_weather) {
                self.faster
            } else {
                1.0
            };
            *level += step(self.fill_ms) * faster * (1.0 + pct / 100.0).max(0.1);
            true
        };
        self.settle(level, full);
        open
    }

    /// Clamp the bar; full makes Elora slow until it is below `recover` again.
    fn settle(&self, level: &mut f32, full: &mut bool) {
        *level = level.clamp(0.0, 1.0);
        if *level >= 1.0 {
            *full = true;
        } else if *level < self.recover {
            *full = false;
        }
    }
}

impl Session {
    /// Is the map a hot area (desert, E-320)?
    pub fn hot(&self) -> bool {
        self.content.area_of(&self.map_name).is_some_and(|a| a.hot)
    }

    /// Is the map a cold area (Frostspitzen, E-342)?
    pub fn chilly(&self) -> bool {
        self.content.area_of(&self.map_name).is_some_and(|a| a.cold)
    }

    /// Heat bar (E-320) and cold bar (E-342) for one tick. Outside their areas the bars are
    /// empty. Heat and cold slow Elora down equally (A-27).
    pub(super) fn temperature(&mut self, world: &mut World, pos: Vec2) {
        let roof = (1..=ROOF_TILES).any(|k| {
            #[allow(clippy::cast_precision_loss)]
            let p = pos - Vec2::new(0.0, (k * TILE_SIZE) as f32);
            let t = world.collision.tile_at(p);
            t.is_solid() || t == Tile::Platform
        });
        let at = Surroundings {
            map: &self.map,
            pos,
            roof,
            arena: self.map_name.ends_with("-arena"),
        };
        let bars = &self.content.temperature;
        let stats = self.save.stats(&self.content);
        let (hot, chilly) = (self.hot(), self.chilly());
        self.in_sun = false;
        if hot {
            self.in_sun = bars
                .heat
                .tick(&mut self.heat, &mut self.overheated, &at, stats.heat_pct);
        } else {
            self.heat = 0.0;
            bars.heat.settle(&mut self.heat, &mut self.overheated);
        }
        if chilly {
            bars.cold
                .tick(&mut self.cold, &mut self.frozen, &at, stats.cold_pct);
        } else {
            self.cold = 0.0;
            bars.cold.settle(&mut self.cold, &mut self.frozen);
        }
        if let Some(ch) = world.character_mut(self.player) {
            ch.core.overheated = self.overheated || self.frozen;
        }
    }
}
