//! Terrain with state (R2-M2.4, E-343): temporarily placed tiles (root walls, quicksand of the
//! Sand Serpent, broken thin ice) return; thin ice gets cracks under Elora
//! and breaks.

use crate::character::PHYS_SIZE;
use crate::event::Event;
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;
use crate::world::World;
use crate::{TILE_SIZE, Tile};

impl World {
    /// Before the characters: cracks in thin ice, breaks, returning tiles.
    pub(crate) fn tick_terrain(&mut self) {
        if self.prediction {
            return;
        }
        self.crack_thin_ice();
        self.tick_temp_tiles();
    }

    /// Thin ice under standing characters gets cracks and breaks according to A-36.
    fn crack_thin_ice(&mut self) {
        let ts = TILE_SIZE as f32;
        let feet: Vec<(i32, i32)> = self
            .players
            .iter()
            .filter_map(|p| p.as_ref()?.character.as_ref())
            .flat_map(|ch| {
                let y = ch.core.pos.y + PHYS_SIZE / 2.0 + 5.0;
                [-1.0, 1.0].map(|s| Vec2::new(ch.core.pos.x + s * (PHYS_SIZE / 2.0 - 1.0), y))
            })
            .filter(|&p| self.collision.tile_at(p) == Tile::ThinIce)
            .map(|p| {
                #[allow(clippy::cast_possible_truncation)]
                ((p.x / ts).floor() as i32, (p.y / ts).floor() as i32)
            })
            .collect();
        let until = self.tick + u64::from(ms_to_ticks(self.tuning.thin_ice_break).max(1));
        for (tx, ty) in feet {
            if !self.cracking.iter().any(|c| (c.0, c.1) == (tx, ty)) {
                self.cracking.push((tx, ty, until));
                self.events.push(Event::IceCrack {
                    tx,
                    ty,
                    broken: false,
                });
            }
        }
        let tick = self.tick;
        let due: Vec<(i32, i32)> = self
            .cracking
            .iter()
            .filter(|c| c.2 <= tick)
            .map(|c| (c.0, c.1))
            .collect();
        for (tx, ty) in due {
            self.break_thin_ice(tx, ty);
        }
    }

    /// Thin ice breaks (cracks expired or stomp) and grows back according to A-37.
    pub(crate) fn break_thin_ice(&mut self, tx: i32, ty: i32) {
        self.cracking.retain(|c| (c.0, c.1) != (tx, ty));
        if self.collision.tile(tx, ty) != Tile::ThinIce {
            return;
        }
        self.collision.set_tile(tx, ty, Tile::Air);
        let until = self.tick + u64::from(ms_to_ticks(self.tuning.thin_ice_regrow));
        self.temp_tiles.push((tx, ty, Tile::ThinIce, until));
        self.events.push(Event::IceCrack {
            tx,
            ty,
            broken: true,
        });
        self.events.push(Event::TileSet {
            tx,
            ty,
            tile: Tile::Air,
        });
    }

    /// Temporary tiles return – solid ones only once no character and no enemy is inside.
    fn tick_temp_tiles(&mut self) {
        let tick = self.tick;
        let ts = TILE_SIZE as f32;
        let mut i = 0;
        while i < self.temp_tiles.len() {
            let (tx, ty, old, until) = self.temp_tiles[i];
            #[allow(clippy::cast_precision_loss)]
            let (x0, y0) = (tx as f32 * ts, ty as f32 * ts);
            let overlaps = |p: Vec2, half: Vec2| {
                p.x + half.x > x0
                    && p.x - half.x < x0 + ts
                    && p.y + half.y > y0
                    && p.y - half.y < y0 + ts
            };
            let blocked = old.is_solid()
                && (self
                    .players
                    .iter()
                    .filter_map(|p| p.as_ref()?.character.as_ref())
                    .any(|ch| overlaps(ch.core.pos, Vec2::new(PHYS_SIZE / 2.0, PHYS_SIZE / 2.0)))
                    || self
                        .creatures
                        .iter()
                        .any(|c| overlaps(c.pos, self.creature_kinds[c.kind].size() * 0.5)));
            if tick >= until && !blocked {
                self.collision.set_tile(tx, ty, old);
                self.events.push(Event::TileSet { tx, ty, tile: old });
                self.temp_tiles.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
