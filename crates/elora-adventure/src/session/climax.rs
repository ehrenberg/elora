//! Climax of chapter 5, “Protect Luma” (R2-M2.5, M2.5.5, E-362).
//!
//! Built from map zones:
//! - `climax-start`: walking in starts the climax (unless Luma was already protected);
//! - `climax-luma`: Luma's place, the goal of the grey shadows (centre of the zone);
//! - `shade-spawn…`: where grey shadows come out of the dark;
//! - `withered-1` … `withered-3`: where the Withered One stands in each verse;
//! - `lantern…`: star lanterns (built by the map, see `elora_map::islands`).
//!
//! Three verses: while no shadow touches Luma her song fills; a shadow on Luma dims her light
//! and stops the song for a moment. Between the verses the Withered One moves on; after the
//! third he looks at Elora and flees towards Tauwinkel (no fight). If Luma's light goes out,
//! Elora goes back to the spring stone without any loss.

use elora_map::ObjectKind;
use elora_sim::tuning::ms_to_ticks;
use elora_sim::{Vec2, World};

use super::{Session, SessionEvent};

/// Zone that starts the climax.
pub const START_ZONE: &str = "climax-start";
/// Zone whose centre is Luma's place.
pub const LUMA_ZONE: &str = "climax-luma";
/// Start of the ids of the zones grey shadows come from.
pub const SPAWN_ZONE: &str = "shade-spawn";
/// Start of the ids of the Withered One's places (`withered-1` …).
pub const WITHERED_ZONE: &str = "withered-";
/// Flag: Luma was protected, the climax does not start again.
pub const LUMA_PROTECTED: &str = "luma_protected";
/// Flag: Luma's song is complete (side quest “Luma's song”, M2.5.6) – she sings stronger.
pub const LUMA_SONG: &str = "luma_song";

/// Number of verses.
pub const VERSES: u8 = 3;
/// Length of a verse without disturbance (ms).
const VERSE_MS: u32 = 30_000;
/// With Luma's song complete the bar fills this much faster.
const SONG_BONUS: f32 = 1.5;
/// A new grey shadow every this many ms, per verse.
const SPAWN_MS: [u32; VERSES as usize] = [2400, 1700, 1200];
/// Speed of the shadows in percent of their kind's speed, per verse.
const SHADE_SPEED: [u32; VERSES as usize] = [100, 125, 150];
/// A shadow this close to Luma reaches her (units).
const LUMA_REACH: f32 = 30.0;
/// Light Luma loses per shadow that reaches her (of 1).
const LIGHT_LOSS: f32 = 0.2;
/// The song stops this long after a shadow reached Luma (ms).
const PAUSE_MS: u32 = 1500;
/// The Withered One moves to his next place in this time (ms).
const MOVE_MS: u32 = 2500;
/// After the last verse he looks at Elora this long (ms), then flees this long.
const LOOK_MS: u32 = 2000;
const FLEE_MS: u32 = 3000;

/// Mode of the Withered One (in [`elora_sim::Creature::mode`], for the client).
pub mod withered {
    /// Drains the colour of the spring.
    pub const DRAIN: u8 = 0;
    /// Looks at Elora (after the last verse).
    pub const LOOK: u8 = 1;
    /// Turns away and flees.
    pub const FLEE: u8 = 2;
}

/// What the climax is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// Luma sings a verse, shadows come.
    #[default]
    Song,
    /// Between the verses: the Withered One moves on (ticks so far).
    Moving(u32),
    /// After the last verse: he looks at Elora (ticks so far).
    Looking(u32),
    /// He flees (ticks so far).
    Fleeing(u32),
}

/// Running climax “Protect Luma” (the default: first verse, nothing sung, no light yet).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Climax {
    /// Current verse (0-based).
    pub verse: u8,
    /// Progress of the current verse 0..1.
    pub song: f32,
    /// Luma's light 0..1; at 0 the climax is lost.
    pub light: f32,
    /// What happens right now.
    pub phase: Phase,
    /// Luma's place (goal of the shadows).
    pub luma: Vec2,
    /// The song stands still until this world tick (a shadow reached Luma).
    pub paused_until: u64,
    /// Enemy id of the Withered One in the world.
    withered: Option<u32>,
    /// Where he comes from while moving.
    from: Vec2,
    /// Ticks until the next shadow.
    next_shade: u32,
    /// Shadows spawned so far (picks spawn zone and place).
    spawned: u32,
}

impl Climax {
    /// Is the song standing still at `tick`?
    pub fn paused(&self, tick: u64) -> bool {
        tick < self.paused_until
    }

    /// Progress over all verses 0..1 (for the bar).
    pub fn total(&self) -> f32 {
        let done = match self.phase {
            Phase::Looking(_) | Phase::Fleeing(_) => 1.0,
            _ => (f32::from(self.verse) + self.song) / f32::from(VERSES),
        };
        done.clamp(0.0, 1.0)
    }
}

/// Centre of a zone.
fn centre(o: &elora_map::Object) -> Option<Vec2> {
    match o.kind {
        ObjectKind::Zone { size } => Some(o.pos + size * 0.5),
        _ => None,
    }
}

impl Session {
    fn zone_centre(&self, id: &str) -> Option<Vec2> {
        self.map.adventure.object(id).and_then(centre)
    }

    /// Place of the Withered One in verse `verse` (0-based); the last place given stays.
    fn withered_place(&self, verse: u8) -> Option<Vec2> {
        (1..=verse + 1)
            .rev()
            .find_map(|n| self.zone_centre(&format!("{WITHERED_ZONE}{n}")))
    }

    /// Starts the climax when Elora walks into `climax-start` (only once until it is won).
    pub(super) fn maybe_start_climax(&mut self, world: &mut World) {
        if self.climax.is_some()
            || self.save.flag(LUMA_PROTECTED) != 0
            || !self.inside.contains(START_ZONE)
        {
            return;
        }
        let Some(luma) = self.zone_centre(LUMA_ZONE) else {
            return;
        };
        let place = self
            .withered_place(0)
            .unwrap_or(luma + Vec2::new(0.0, -200.0));
        let withered = world
            .creature_kind("withered")
            .and_then(|k| world.add_creature(k, place));
        self.climax = Some(Climax {
            verse: 0,
            song: 0.0,
            light: 1.0,
            phase: Phase::Song,
            luma,
            paused_until: 0,
            withered,
            from: place,
            next_shade: ms_to_ticks(SPAWN_MS[0]) / 2,
            spawned: 0,
        });
    }

    /// One tick of the running climax.
    pub(super) fn tick_climax(&mut self, world: &mut World, out: &mut Vec<SessionEvent>) {
        let Some(mut c) = self.climax.take() else {
            return;
        };
        let keep = match c.phase {
            Phase::Song => self.climax_song(world, &mut c, out),
            Phase::Moving(t) => {
                let to = self.withered_place(c.verse).unwrap_or(c.from);
                let f = (t as f32 / ms_to_ticks(MOVE_MS) as f32).min(1.0);
                // glides on a small arc
                let lift = Vec2::new(0.0, -60.0 * (f * std::f32::consts::PI).sin());
                place_withered(world, c.withered, c.from + (to - c.from) * f + lift, None);
                c.phase = if f >= 1.0 {
                    c.from = to;
                    Phase::Song
                } else {
                    Phase::Moving(t + 1)
                };
                true
            }
            Phase::Looking(t) => {
                let elora = world.character(self.player).map(|ch| ch.core.pos);
                if let Some(e) = elora {
                    let facing = if e.x < c.from.x { -1 } else { 1 };
                    face_withered(world, c.withered, facing, withered::LOOK);
                }
                c.phase = if t >= ms_to_ticks(LOOK_MS) {
                    Phase::Fleeing(0)
                } else {
                    Phase::Looking(t + 1)
                };
                true
            }
            Phase::Fleeing(t) => {
                // towards Tauwinkel: to the west and up into the dark
                let f = t as f32 / ms_to_ticks(FLEE_MS) as f32;
                let at = c.from + Vec2::new(-900.0 * f * f, -260.0 * f);
                place_withered(world, c.withered, at, Some(withered::FLEE));
                face_withered(world, c.withered, -1, withered::FLEE);
                if f >= 1.0 {
                    if let Some(id) = c.withered {
                        remove(world, id);
                    }
                    false
                } else {
                    c.phase = Phase::Fleeing(t + 1);
                    true
                }
            }
        };
        if keep {
            self.climax = Some(c);
        }
    }

    /// A verse: spawn shadows, let them reach Luma, fill the song.
    fn climax_song(
        &mut self,
        world: &mut World,
        c: &mut Climax,
        out: &mut Vec<SessionEvent>,
    ) -> bool {
        let verse = usize::from(c.verse);
        face_withered(world, c.withered, 0, withered::DRAIN);
        // new shadows
        if c.next_shade == 0 {
            c.next_shade = ms_to_ticks(SPAWN_MS[verse]);
            if let Some(at) = self.shade_place(c.spawned)
                && let Some(k) = world.creature_kind("grey_shade")
                && let Some(id) = world.add_creature(k, at)
                && let Some(s) = world.creatures.iter_mut().find(|s| s.id == id)
            {
                s.goal = c.luma;
                s.count = SHADE_SPEED[verse];
            }
            c.spawned += 1;
        } else {
            c.next_shade -= 1;
        }
        // shadows reaching Luma
        let shade = world.creature_kind("grey_shade");
        let reached: Vec<u32> = world
            .creatures
            .iter()
            .filter(|s| Some(s.kind) == shade && s.pos.distance(c.luma) <= LUMA_REACH)
            .map(|s| s.id)
            .collect();
        for id in &reached {
            remove(world, *id);
            c.light = (c.light - LIGHT_LOSS).max(0.0);
            c.paused_until = world.tick + u64::from(ms_to_ticks(PAUSE_MS));
        }
        if c.light <= 0.0 {
            // lost: back to the spring stone, nothing is lost (E-362)
            clear_shades(world);
            if let Some(id) = c.withered {
                remove(world, id);
            }
            self.dead = true;
            out.push(SessionEvent::Died { lost: 0 });
            return false;
        }
        // the song
        if !c.paused(world.tick) {
            let bonus = if self.save.flag(LUMA_SONG) != 0 {
                SONG_BONUS
            } else {
                1.0
            };
            c.song += bonus / ms_to_ticks(VERSE_MS) as f32;
        }
        if c.song >= 1.0 {
            // verse done: the shadows dissolve, Luma's light recovers a little
            clear_shades(world);
            c.verse += 1;
            c.song = 0.0;
            c.light = (c.light + LIGHT_LOSS).min(1.0);
            if c.verse >= VERSES {
                c.phase = Phase::Looking(0);
                self.save.set_flag(LUMA_PROTECTED, 1);
                let outs = self.save.update_quests(&self.content);
                out.extend(super::quests::outcomes(outs));
            } else {
                c.phase = Phase::Moving(0);
                c.next_shade = ms_to_ticks(SPAWN_MS[usize::from(c.verse)]);
            }
        }
        true
    }

    /// Where shadow number `n` comes out: the spawn zones in turn, a spread-out place inside.
    fn shade_place(&self, n: u32) -> Option<Vec2> {
        let zones: Vec<(Vec2, Vec2)> = self
            .map
            .adventure
            .objects
            .iter()
            .filter(|o| o.id.starts_with(SPAWN_ZONE))
            .filter_map(|o| match o.kind {
                ObjectKind::Zone { size } => Some((o.pos, size)),
                _ => None,
            })
            .collect();
        let (pos, size) = *zones.get(n as usize % zones.len().max(1))?;
        // golden ratio: places spread evenly without a random generator (deterministic)
        let fx = (n as f32 * 0.618_034).fract();
        let fy = (n as f32 * 0.414_214 + 0.5).fract();
        Some(pos + Vec2::new(size.x * fx, size.y * fy))
    }
}

fn remove(world: &mut World, id: u32) {
    world.remove_creature(id);
}

/// All grey shadows dissolve.
fn clear_shades(world: &mut World) {
    let Some(shade) = world.creature_kind("grey_shade") else {
        return;
    };
    let ids: Vec<u32> = world
        .creatures
        .iter()
        .filter(|s| s.kind == shade)
        .map(|s| s.id)
        .collect();
    for id in ids {
        remove(world, id);
    }
}

fn place_withered(world: &mut World, id: Option<u32>, at: Vec2, mode: Option<u8>) {
    if let Some(w) = world.creatures.iter_mut().find(|c| Some(c.id) == id) {
        w.pos = at;
        if let Some(m) = mode {
            w.mode = m;
        }
    }
}

/// Sets the mode and, unless `facing` is 0, the looking direction of the Withered One.
fn face_withered(world: &mut World, id: Option<u32>, facing: i8, mode: u8) {
    if let Some(w) = world.creatures.iter_mut().find(|c| Some(c.id) == id) {
        w.mode = mode;
        if facing != 0 {
            w.facing = facing;
        }
    }
}
