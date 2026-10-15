//! Note stones and star bridges (R2-M2.5, M2.5.3, E-364).
//!
//! Touching a note stone plays its note and lights it for A-50; while it is lit, its star
//! bridges carry like floating islands (one-way platforms). Lit stones and bridges count as
//! light for the creatures of the night. Adventure only, not networked; the map zones become
//! stones and bridges in `elora-map`.

use crate::event::Event;
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// Light radius of a lit note stone (units).
pub const STONE_LIGHT: f32 = 150.0;
/// Light reaching out from a lit star bridge (units).
pub const BRIDGE_LIGHT: f32 = 80.0;

/// A note stone.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteStone {
    /// Top-left corner of the touch area (units).
    pub at: Vec2,
    /// Size of the touch area.
    pub size: Vec2,
    /// Pitch in semitones above c (0 = c, 12 = the c above).
    pub note: u8,
    /// Indices into [`crate::World::bridges`] that this stone lights.
    pub bridges: Vec<usize>,
    /// Lit until this tick.
    pub lit_until: u64,
    /// Someone touched it in the last tick (a note sounds only when touched anew).
    pub touched: bool,
}

impl NoteStone {
    /// Note stone at `at` with touch area `size`.
    pub fn new(at: Vec2, size: Vec2, note: u8, bridges: Vec<usize>) -> Self {
        Self {
            at,
            size,
            note,
            bridges,
            lit_until: 0,
            touched: false,
        }
    }

    /// Centre of the stone.
    pub fn centre(&self) -> Vec2 {
        self.at + self.size * 0.5
    }

    /// Is it lit at `tick`?
    pub fn lit(&self, tick: u64) -> bool {
        tick < self.lit_until
    }
}

/// A star bridge: carries while its note stone is lit.
#[derive(Debug, Clone, PartialEq)]
pub struct StarBridge {
    /// Top-left corner (units); the top edge is the walkable surface.
    pub at: Vec2,
    /// Size.
    pub size: Vec2,
    /// Carries until this tick.
    pub until: u64,
    /// Tick at which it was last lit (fading in).
    pub since: u64,
}

impl StarBridge {
    /// Unlit star bridge.
    pub fn new(at: Vec2, size: Vec2) -> Self {
        Self {
            at,
            size,
            until: 0,
            since: 0,
        }
    }

    /// Does it carry at `tick`?
    pub fn active(&self, tick: u64) -> bool {
        tick < self.until
    }

    /// Shortest distance from `p` to the walkable surface.
    fn distance(&self, p: Vec2) -> f32 {
        let x = p.x.clamp(self.at.x, self.at.x + self.size.x);
        p.distance(Vec2::new(x, self.at.y))
    }
}

/// Parses the pitch of a note stone id `note-<pitch>…`: `c d e f g a b`, followed by `2` for the
/// octave above (e.g. `note-e-1`, `note-c2`).
pub fn pitch(id: &str) -> Option<u8> {
    let rest = id.strip_prefix("note-")?;
    let mut chars = rest.chars();
    let base = match chars.next()? {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => return None,
    };
    match chars.next() {
        None | Some('-') => Some(base),
        Some('2') if matches!(chars.next(), None | Some('-')) => Some(base + 12),
        _ => None,
    }
}

impl crate::World {
    /// Put note stones and star bridges into the world (from the map).
    pub fn set_notes(&mut self, stones: Vec<NoteStone>, bridges: Vec<StarBridge>) {
        self.note_stones = stones;
        self.bridges = bridges;
    }

    /// Is a lit note stone or an active star bridge near `pos`?
    pub(crate) fn note_light(&self, pos: Vec2) -> bool {
        self.note_stones
            .iter()
            .any(|s| s.lit(self.tick) && s.centre().distance(pos) <= STONE_LIGHT)
            || self
                .bridges
                .iter()
                .any(|b| b.active(self.tick) && b.distance(pos) <= BRIDGE_LIGHT)
    }

    /// All light spots of the night for the creatures (centre, radius): the light sources of
    /// the map, lit note stones and points along the active star bridges.
    pub(crate) fn light_spots(&self) -> Vec<(Vec2, f32)> {
        let mut out = self.lights.clone();
        let tick = self.tick;
        for s in self.note_stones.iter().filter(|s| s.lit(tick)) {
            out.push((s.centre(), STONE_LIGHT));
        }
        for b in self.bridges.iter().filter(|b| b.active(tick)) {
            let mut x = b.at.x;
            while x < b.at.x + b.size.x + BRIDGE_LIGHT {
                out.push((Vec2::new(x.min(b.at.x + b.size.x), b.at.y), BRIDGE_LIGHT));
                x += BRIDGE_LIGHT;
            }
        }
        out
    }

    /// Before the islands: touched note stones sound and light their bridges.
    pub(crate) fn tick_notes(&mut self) {
        if self.note_stones.is_empty() {
            return;
        }
        let half = crate::PHYS_SIZE / 2.0;
        let bodies: Vec<(usize, Vec2)> = self
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| Some((i, p.as_ref()?.character.as_ref()?.core.pos)))
            .collect();
        let duration = u64::from(ms_to_ticks(self.tuning.note_light_time));
        let tick = self.tick;
        for stone in &mut self.note_stones {
            let by = bodies.iter().find(|&&(_, p)| {
                p.x + half > stone.at.x
                    && p.x - half < stone.at.x + stone.size.x
                    && p.y + half > stone.at.y
                    && p.y - half < stone.at.y + stone.size.y
            });
            if let Some(&(player, _)) = by
                && !stone.touched
            {
                stone.lit_until = tick + duration;
                for &b in &stone.bridges {
                    if let Some(bridge) = self.bridges.get_mut(b) {
                        if !bridge.active(tick) {
                            bridge.since = tick;
                        }
                        bridge.until = bridge.until.max(stone.lit_until);
                    }
                }
                self.events.push(Event::Note {
                    player,
                    note: stone.note,
                    pos: stone.centre(),
                });
            }
            stone.touched = by.is_some();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitches_from_ids() {
        assert_eq!(pitch("note-c"), Some(0));
        assert_eq!(pitch("note-e-1"), Some(4));
        assert_eq!(pitch("note-b-high"), Some(11));
        assert_eq!(pitch("note-c2"), Some(12));
        assert_eq!(pitch("note-g2-x"), Some(19));
        assert_eq!(pitch("note-h"), None);
        assert_eq!(pitch("note-cx"), None);
        assert_eq!(pitch("notes"), None);
    }
}
