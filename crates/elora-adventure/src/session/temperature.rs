//! Heat bar in hot areas (E-320) and cold bar in cold areas (E-342).

use elora_map::ObjectKind;
use elora_sim::{TILE_SIZE, Tile, Vec2, World};

use super::{Session, inside};

/// Heat bar (E-320): this long in the sun until full, in the shade and at the oasis until empty.
const HEAT_FILL_MS: u32 = 20_000;
const HEAT_SHADE_MS: u32 = 8_000;
const HEAT_OASIS_MS: u32 = 2_500;
/// When it is full, Elora stays slower until the bar drops below this fraction again.
const HEAT_RECOVER: f32 = 0.5;
/// Cold bar (E-342, D-M24-04): this long outside until full (half as long in a blizzard),
/// under a roof and at the fire until empty.
const COLD_FILL_MS: u32 = 60_000;
const COLD_ROOF_MS: u32 = 10_000;
const COLD_FIRE_MS: u32 = 3_000;
/// A roof (solid tile or platform) this many tiles above Elora gives shade.
const SHADE_TILES: i32 = 10;

impl Session {
    /// Is the map a hot area (desert, E-320)?
    pub fn hot(&self) -> bool {
        self.content.area_of(&self.map_name).is_some_and(|a| a.hot)
    }

    /// Is the map a cold area (Frostspitzen, E-342)?
    pub fn chilly(&self) -> bool {
        self.content.area_of(&self.map_name).is_some_and(|a| a.cold)
    }

    /// Cold bar (E-342): outside it fills up (twice as fast in a blizzard, equipment
    /// `cold_pct` slows it down), under a roof and at the fire (zones `feuer…`) Elora warms up;
    /// in the guardians' arenas it rests (warms slowly). Full = slower until it has dropped below
    /// half.
    pub(super) fn chill(&mut self, world: &mut World, pos: Vec2) {
        let fire = self.map.adventure.objects.iter().any(|o| {
            o.id.starts_with("fire")
                && matches!(o.kind, ObjectKind::Zone { size } if inside(pos, o.pos, size))
        });
        let roof = (1..=SHADE_TILES).any(|k| {
            #[allow(clippy::cast_precision_loss)]
            let p = pos - Vec2::new(0.0, (k * TILE_SIZE) as f32);
            let t = world.collision.tile_at(p);
            t.is_solid() || t == Tile::Platform
        });
        #[allow(clippy::cast_precision_loss)]
        let step = |ms: u32| 1.0 / elora_sim::tuning::ms_to_ticks(ms).max(1) as f32;
        if !self.chilly() {
            self.cold = 0.0;
        } else if fire {
            self.cold -= step(COLD_FIRE_MS);
        } else if roof || self.map_name.ends_with("-arena") {
            self.cold -= step(COLD_ROOF_MS);
        } else {
            let storm = if self.map.weather.kind == elora_map::WeatherKind::Blizzard {
                2.0
            } else {
                1.0
            };
            let pct = self.save.stats(&self.content).cold_pct;
            self.cold += step(COLD_FILL_MS) * storm * (1.0 + pct / 100.0).max(0.1);
        }
        self.cold = self.cold.clamp(0.0, 1.0);
        if self.cold >= 1.0 {
            self.frozen = true;
        } else if self.cold < HEAT_RECOVER {
            self.frozen = false;
        }
        // heat and cold slow down equally (A-27)
        if let Some(ch) = world.character_mut(self.player) {
            ch.core.overheated = self.overheated || self.frozen;
        }
    }

    /// Heat bar (E-320): sun fills it (equipment `heat_pct` slows it down); shade (roof above
    /// Elora, zones `schatten…`) and oasis (zones `oase…`) cool. Full = Elora gets slower until
    /// she has cooled down.
    pub(super) fn heat(&mut self, world: &mut World, pos: Vec2) {
        let zone = |prefix: &str| {
            self.map.adventure.objects.iter().any(|o| {
                o.id.starts_with(prefix)
                    && matches!(o.kind, ObjectKind::Zone { size } if inside(pos, o.pos, size))
            })
        };
        let roof = (1..=SHADE_TILES).any(|k| {
            #[allow(clippy::cast_precision_loss)]
            let p = pos - Vec2::new(0.0, (k * TILE_SIZE) as f32);
            let t = world.collision.tile_at(p);
            t.is_solid() || t == Tile::Platform
        });
        #[allow(clippy::cast_precision_loss)]
        let step = |ms: u32| 1.0 / elora_sim::tuning::ms_to_ticks(ms).max(1) as f32;
        self.in_sun = false;
        if !self.hot() {
            self.heat = 0.0;
        } else if zone("oasis") {
            self.heat -= step(HEAT_OASIS_MS);
        } else if roof
            || zone("shade")
            // sand hides the sun (R2-W1)
            || self.map.weather.kind == elora_map::WeatherKind::Sandstorm
        {
            self.heat -= step(HEAT_SHADE_MS);
        } else {
            self.in_sun = true;
            // sun veil: fills more slowly
            let pct = self.save.stats(&self.content).heat_pct;
            self.heat += step(HEAT_FILL_MS) * (1.0 + pct / 100.0).max(0.1);
        }
        self.heat = self.heat.clamp(0.0, 1.0);
        if self.heat >= 1.0 {
            self.overheated = true;
        } else if self.heat < HEAT_RECOVER {
            self.overheated = false;
        }
        if let Some(ch) = world.character_mut(self.player) {
            ch.core.overheated = self.overheated;
        }
    }
}
