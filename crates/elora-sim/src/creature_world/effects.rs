//! What the guardians leave in the world: root thrusts, root walls, icicles, quicksand.

use crate::character::PHYS_SIZE;
use crate::collision::Tile;
use crate::event::{DeathCause, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;
use crate::world::World;

impl World {
    /// Root thrusts hit figures in their zone (knockback upwards).
    pub(super) fn root_spikes(
        &mut self,
        spikes: &[(Vec2, f32, f32, i32)],
        chars: &[(usize, Vec2)],
    ) {
        let kb = self.tuning.hit_knockback;
        for &(at, w, h, damage) in spikes {
            for &(j, p) in chars {
                if (p.x - at.x).abs() < w / 2.0 + PHYS_SIZE / 2.0
                    && p.y > at.y - h
                    && p.y < at.y + 4.0
                {
                    self.take_damage(
                        j,
                        Vec2::new(0.0, -kb * 1.2),
                        damage,
                        None,
                        DeathCause::Creature,
                    );
                }
            }
        }
    }

    /// Root walls: a tile column from the ground upwards, only in air and not on figures.
    pub(super) fn root_walls(&mut self, walls: &[(Vec2, u32, u32)], chars: &[(usize, Vec2)]) {
        let ts = crate::TILE_SIZE;
        for &(foot, height, ms) in walls {
            let tx = (foot.x as i32).div_euclid(ts);
            let ground = (foot.y as i32 - 1).div_euclid(ts);
            let until = self.tick + u64::from(ms_to_ticks(ms));
            #[allow(clippy::cast_possible_wrap)] // a few tiles
            for k in 0..height as i32 {
                let ty = ground - k;
                if self.collision.tile(tx, ty) != Tile::Air {
                    break;
                }
                let center =
                    Vec2::new((tx as f32 + 0.5) * ts as f32, (ty as f32 + 0.5) * ts as f32);
                if chars.iter().any(|&(_, p)| p.distance(center) < PHYS_SIZE) {
                    break;
                }
                self.collision.set_tile(tx, ty, Tile::Unhookable);
                self.temp_tiles.push((tx, ty, Tile::Air, until));
                self.events.push(Event::TileSet {
                    tx,
                    ty,
                    tile: Tile::Unhookable,
                });
            }
        }
    }

    /// Icicles of the guardian (angry): on the ceiling above and next to Elora; they tremble
    /// at once.
    pub(super) fn drop_icicles(&mut self, drops: &[(Vec2, u32)]) {
        let Some(kind) = self.creature_kind("icicle") else {
            return;
        };
        let half = self.creature_kinds[kind].size[1] / 2.0;
        let ts = crate::TILE_SIZE as f32;
        for &(at, n) in drops {
            for k in 0..n {
                let x = at.x + (k as f32 - (n as f32 - 1.0) / 2.0) * 110.0;
                // ceiling above this spot
                let mut y = at.y;
                while y > at.y - 900.0 && !self.collision.tile_at(Vec2::new(x, y)).is_solid() {
                    y -= 8.0;
                }
                if y <= at.y - 900.0 {
                    continue;
                }
                let pos = Vec2::new(x, ((y / ts).floor() + 1.0) * ts + half + 1.0);
                if let Some(id) = self.add_creature(kind, pos)
                    && let Some(c) = self.creatures.iter_mut().find(|c| c.id == id)
                {
                    c.mode = crate::creature::icicle::SHAKE;
                }
            }
        }
    }

    /// Quicksand in the basin (sand serpent, angry): the surface of the ground around `at` turns
    /// into quicksand for a while (only solid tiles with air above and solid ground below).
    pub(super) fn quicksand_patches(&mut self, patches: &[(Vec2, u32, u32)]) {
        let ts = crate::TILE_SIZE;
        for &(at, width, ms) in patches {
            let (cx, ty) = (
                (at.x as i32).div_euclid(ts),
                (at.y as i32 + 4).div_euclid(ts),
            );
            let until = self.tick + u64::from(ms_to_ticks(ms));
            #[allow(clippy::cast_possible_wrap)] // a few tiles
            let half = width as i32 / 2;
            for tx in cx - half..=cx + half {
                let t = self.collision.tile(tx, ty);
                if !matches!(t, Tile::Solid | Tile::Unhookable)
                    || self.collision.tile(tx, ty - 1) != Tile::Air
                    || !self.collision.tile(tx, ty + 1).is_solid()
                {
                    continue;
                }
                self.collision.set_tile(tx, ty, Tile::Quicksand);
                self.temp_tiles.push((tx, ty, t, until));
                self.events.push(Event::TileSet {
                    tx,
                    ty,
                    tile: Tile::Quicksand,
                });
            }
        }
    }
}
