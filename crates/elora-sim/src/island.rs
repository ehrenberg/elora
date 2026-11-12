//! Floating star islands and updrafts (R2-M2.5, M2.5.2, E-364).
//!
//! An island is a moving one-way platform: walkable from above, passable from below and the
//! sides, like [`crate::Tile::Platform`]. It drifts back and forth between two points (its
//! position comes from the tick count, so it never drifts apart) and can sink slowly while
//! someone stands on it, rising again afterwards. Characters and creatures standing on it are
//! carried along. Updrafts are areas that lift gliding characters. Both are adventure-only and
//! not networked; the session creates them from the zones of the map.

use crate::math::{Vec2, round_to_int};

/// A floating island.
#[derive(Debug, Clone, PartialEq)]
pub struct Island {
    /// Top-left corner at the start of the path (units).
    pub start: Vec2,
    /// Top-left corner at the other end of the path; equal to `start` for a fixed island.
    pub end: Vec2,
    /// Width and height (units); the top edge is the walkable surface.
    pub size: Vec2,
    /// Sinks while someone stands on it (A-44 to A-46).
    pub sinks: bool,
    /// How deep it has sunk (units).
    pub sunk: f32,
    /// Current top-left corner (whole units).
    pub pos: Vec2,
    /// Top-left corner in the previous tick (for drawing in between ticks).
    pub prev: Vec2,
    /// Someone stood on it in the last tick.
    pub loaded: bool,
}

impl Island {
    /// Island at rest at `start`, moving to `end` and back (`start == end`: fixed).
    pub fn new(start: Vec2, end: Vec2, size: Vec2, sinks: bool) -> Self {
        Self {
            start,
            end,
            size,
            sinks,
            sunk: 0.0,
            pos: start,
            prev: start,
            loaded: false,
        }
    }

    /// Position on the path at `tick` without sinking: back and forth at `speed` units per
    /// tick, slowing down at both ends.
    pub fn path_pos(&self, tick: u64, speed: f32) -> Vec2 {
        let length = self.start.distance(self.end);
        if length < 1.0 || speed <= 0.0 {
            return self.start;
        }
        let one_way = crate::math::to_u32(length / speed).max(1);
        let phase = tick % (2 * u64::from(one_way));
        let s = phase as f32 / one_way as f32;
        let u = if s <= 1.0 { s } else { 2.0 - s };
        let eased = u * u * (3.0 - 2.0 * u);
        self.start.lerp(self.end, eased)
    }

    /// Is a box (centre `pos`, width `w`, bottom edge `bottom`) standing on this island?
    fn carries(&self, x: f32, half_w: f32, bottom: f32) -> bool {
        overlaps_x(self.pos, self.size, x, half_w)
            && bottom >= self.pos.y - STAND_ABOVE
            && bottom <= self.pos.y + STAND_BELOW
    }
}

/// A figure counts as standing this far above the surface (units) ...
pub(crate) const STAND_ABOVE: f32 = 2.5;
/// ... and this far below it.
pub(crate) const STAND_BELOW: f32 = 0.5;

fn overlaps_x(at: Vec2, size: Vec2, x: f32, half_w: f32) -> bool {
    x + half_w > at.x && x - half_w < at.x + size.x
}

/// Moving parts of the collision (islands and updrafts), as boxes: top-left and size.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Movers {
    /// Current islands (one-way platforms).
    pub islands: Vec<(Vec2, Vec2)>,
    /// Updraft areas.
    pub updrafts: Vec<(Vec2, Vec2)>,
}

impl Movers {
    /// Index of the island a box (centre `x`, half width, bottom edge) stands on.
    pub fn island_under(&self, x: f32, half_w: f32, bottom: f32) -> Option<usize> {
        self.islands.iter().position(|&(at, size)| {
            overlaps_x(at, size, x, half_w)
                && bottom >= at.y - STAND_ABOVE
                && bottom <= at.y + STAND_BELOW
        })
    }

    /// Does a box moving down with its bottom edge from `prev_bottom` to `new_bottom` land on
    /// an island?
    pub fn lands(&self, x: f32, half_w: f32, prev_bottom: f32, new_bottom: f32) -> bool {
        new_bottom > prev_bottom
            && self.islands.iter().any(|&(at, size)| {
                overlaps_x(at, size, x, half_w) && prev_bottom <= at.y && new_bottom > at.y
            })
    }

    /// Is the point inside an island (ground for creatures looking ahead)?
    pub fn island_at(&self, p: Vec2) -> bool {
        self.islands.iter().any(|&(at, size)| inside(p, at, size))
    }

    /// Is the point inside an updraft?
    pub fn in_updraft(&self, p: Vec2) -> bool {
        self.updrafts.iter().any(|&(at, size)| inside(p, at, size))
    }
}

fn inside(p: Vec2, at: Vec2, size: Vec2) -> bool {
    p.x >= at.x && p.y >= at.y && p.x <= at.x + size.x && p.y <= at.y + size.y
}

impl crate::World {
    /// Islands before the characters (M2.5.2): move along the path, sink or rise, carry
    /// whoever stands on them.
    pub(crate) fn tick_islands(&mut self) {
        if self.islands.is_empty() && self.bridges.is_empty() {
            return;
        }
        let t = &self.tuning;
        let (speed, sink, rise, depth) = (
            t.island_speed,
            t.island_sink_speed,
            t.island_rise_speed,
            t.island_sink_depth,
        );
        let half = crate::PHYS_SIZE / 2.0;
        for i in 0..self.islands.len() {
            let old = self.islands[i].pos;
            // who stands on it (before it moves)
            let riders: Vec<usize> = self
                .players
                .iter()
                .enumerate()
                .filter_map(|(slot, p)| {
                    let c = &p.as_ref()?.character.as_ref()?.core;
                    (!c.drop_through
                        && c.vel.y >= 0.0
                        && self.islands[i].carries(c.pos.x, half, c.pos.y + half))
                    .then_some(slot)
                })
                .collect();
            let creatures: Vec<usize> = self
                .creatures
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    let size = self.creature_kinds[c.kind].size();
                    c.grounded
                        && c.vel.y >= 0.0
                        && self.islands[i].carries(c.pos.x, size.x / 2.0, c.pos.y + size.y / 2.0)
                })
                .map(|(n, _)| n)
                .collect();
            let island = &mut self.islands[i];
            island.loaded = !riders.is_empty() || !creatures.is_empty();
            if island.sinks {
                island.sunk = if island.loaded {
                    (island.sunk + sink).min(depth)
                } else {
                    (island.sunk - rise).max(0.0)
                };
            }
            let p = island.path_pos(self.tick, speed) + Vec2::new(0.0, island.sunk);
            let new = Vec2::new(round_to_int(p.x) as f32, round_to_int(p.y) as f32);
            island.prev = old;
            island.pos = new;
            let delta = new - old;
            let top = new.y;
            for slot in riders {
                let Some(c) = self.players[slot]
                    .as_mut()
                    .and_then(|p| p.character.as_mut())
                else {
                    continue;
                };
                let to = Vec2::new(c.core.pos.x + delta.x, top - half);
                if !self
                    .collision
                    .test_box(to, Vec2::new(crate::PHYS_SIZE, crate::PHYS_SIZE))
                {
                    c.core.pos = to;
                }
            }
            for n in creatures {
                let size = self.creature_kinds[self.creatures[n].kind].size();
                let c = &mut self.creatures[n];
                let to = Vec2::new(c.pos.x + delta.x, top - size.y / 2.0);
                if !self.collision.test_box(to, size) {
                    c.pos = to;
                }
            }
        }
        self.update_movers();
    }

    /// Collision boxes of the islands and the lit star bridges.
    fn update_movers(&mut self) {
        // lit star bridges carry like fixed islands (M2.5.3)
        let tick = self.tick;
        self.collision.movers.islands = self
            .islands
            .iter()
            .map(|i| (i.pos, i.size))
            .chain(
                self.bridges
                    .iter()
                    .filter(|b| b.active(tick))
                    .map(|b| (b.at, b.size)),
            )
            .collect();
    }

    /// Put islands and updrafts into the world (the session, on entering a map).
    pub fn set_islands(&mut self, islands: Vec<Island>, updrafts: Vec<(Vec2, Vec2)>) {
        self.collision.movers = Movers {
            islands: islands.iter().map(|i| (i.pos, i.size)).collect(),
            updrafts,
        };
        self.islands = islands;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_goes_back_and_forth_and_repeats() {
        let i = Island::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(64.0, 16.0),
            false,
        );
        // 100 units at 1 unit/tick: there after 100 ticks, back after 200
        assert_eq!(i.path_pos(0, 1.0), Vec2::new(0.0, 0.0));
        assert!((i.path_pos(100, 1.0).x - 100.0).abs() < 1e-3);
        assert!((i.path_pos(50, 1.0).x - 50.0).abs() < 1e-3);
        assert_eq!(i.path_pos(200, 1.0), i.path_pos(0, 1.0));
        assert_eq!(i.path_pos(37, 1.0), i.path_pos(237, 1.0));
    }

    #[test]
    fn fixed_island_stays() {
        let p = Vec2::new(64.0, 96.0);
        let i = Island::new(p, p, Vec2::new(64.0, 16.0), false);
        assert_eq!(i.path_pos(12345, 1.0), p);
    }

    #[test]
    fn landing_only_from_above() {
        let m = Movers {
            islands: vec![(Vec2::new(0.0, 100.0), Vec2::new(64.0, 16.0))],
            updrafts: Vec::new(),
        };
        assert!(m.lands(32.0, 14.0, 99.0, 101.0));
        assert!(!m.lands(32.0, 14.0, 101.0, 103.0), "from below");
        assert!(!m.lands(32.0, 14.0, 101.0, 99.0), "upwards");
        assert!(!m.lands(100.0, 14.0, 99.0, 101.0), "beside it");
        assert_eq!(m.island_under(32.0, 14.0, 100.0), Some(0));
        assert_eq!(m.island_under(32.0, 14.0, 110.0), None);
    }
}
