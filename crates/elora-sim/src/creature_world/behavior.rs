//! Behaviour dispatch: one tick per creature, then movement and the collected effects.

use super::{air, diver, ground, grounded, leaper, night, queen, serpent, turret, warden};
use crate::collision::Collision;
use crate::creature::{Behavior, Creature, CreatureKind, CreatureShot};
use crate::event::Event;
use crate::math::Vec2;
use crate::world::World;

/// What a behaviour sees and writes during one tick of one creature.
pub(super) struct Ctx<'a> {
    pub(super) creature: &'a mut Creature,
    pub(super) kind: &'a CreatureKind,
    pub(super) size: Vec2,
    /// Nearest living figure (position, distance).
    pub(super) target: Option<(Vec2, f32)>,
    /// Not stunned.
    pub(super) active: bool,
    pub(super) gravity: f32,
    pub(super) collision: &'a Collision,
    pub(super) chars: &'a [(usize, Vec2)],
    /// Creatures currently pulled on by a player (Root Warden tug of war).
    pub(super) tuggers: &'a [u32],
    /// Light spots of the night (R2-M2.5): centre and radius.
    pub(super) lights: &'a [(Vec2, f32)],
    /// Current tick (hits of this tick).
    pub(super) tick: u64,
    pub(super) shots: &'a mut Vec<CreatureShot>,
    pub(super) events: &'a mut Vec<Event>,
    pub(super) fx: &'a mut Effects,
}

/// Effects collected during the behaviour tick, applied once all creatures have moved.
#[derive(Default)]
pub(super) struct Effects {
    /// Creatures that die without loot (shattered, burst, fell into death tiles).
    pub(super) died: Vec<u32>,
    /// Creatures defeated without a hit (echo faded in the light): loot for the nearest figure.
    pub(super) defeated: Vec<u32>,
    /// Echoes sent out by dark mirrors: mirror id, kind name, position.
    pub(super) echoes: Vec<(u32, String, Vec2)>,
    /// Helpers of the guardians: owner, kind name, at most this many, position.
    pub(super) summons: Vec<(u32, String, u32, Vec2)>,
    /// Root thrusts (centre on the ground, width, height, damage).
    pub(super) spikes: Vec<(Vec2, f32, f32, i32)>,
    /// Root walls (foot, height, duration).
    pub(super) walls: Vec<(Vec2, u32, u32)>,
    /// Quicksand of the sand serpent (centre on the ground, width in tiles, duration).
    pub(super) sands: Vec<(Vec2, u32, u32)>,
    /// Icicles of the guardian: above whom, how many.
    pub(super) drops: Vec<(Vec2, u32)>,
}

/// Result of a behaviour tick.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    /// The creature moves with its velocity and collides with the map.
    Move,
    /// The creature stays put (fixed, asleep or placed by the behaviour itself).
    Hold,
}

impl World {
    pub(super) fn tick_behavior(&mut self, chars: &[(usize, Vec2)]) {
        let tuggers = self.tuggers();
        let lights = self.light_spots();
        let tick = self.tick;
        let Self {
            creatures,
            creature_kinds,
            collision,
            tuning,
            creature_shots,
            events,
            ..
        } = self;
        let gravity = tuning.gravity;
        let mut fx = Effects::default();
        for c in creatures.iter_mut() {
            let kind = &creature_kinds[c.kind];
            let size = kind.size();
            let target = chars
                .iter()
                .map(|&(_, p)| (p, p.distance(c.pos)))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            c.stun = c.stun.saturating_sub(1);
            let active = c.stun == 0;
            let cx = &mut Ctx {
                creature: c,
                kind,
                size,
                target,
                active,
                gravity,
                collision,
                chars,
                tuggers: &tuggers,
                lights: &lights,
                tick,
                shots: creature_shots,
                events,
                fx: &mut fx,
            };
            if dispatch(cx) == Step::Hold {
                continue;
            }
            let wanted_x = c.vel.x;
            let (mut pos, mut vel) = (c.pos, c.vel);
            let diving = matches!(kind.behavior, Behavior::Diver(_));
            let follower = matches!(kind.behavior, Behavior::Follower { .. });
            let death = collision.move_box_platforms(&mut pos, &mut vel, size, 0.0, !diving)
                && !diving
                && !follower;
            if wanted_x != 0.0 && vel.x == 0.0 {
                bumped(kind, c, &mut fx, events);
            }
            c.pos = pos;
            c.vel = vel;
            c.grounded = grounded(collision, pos, size);
            if death {
                fx.died.push(c.id);
            }
        }
        for id in fx.died {
            if let Some(i) = self.creature_index(id) {
                self.kill_creature(i, None, false);
            }
        }
        for id in fx.defeated {
            if let Some(i) = self.creature_index(id) {
                let at = self.creatures[i].pos;
                let by = chars
                    .iter()
                    .min_by(|a, b| a.1.distance(at).total_cmp(&b.1.distance(at)))
                    .map(|&(slot, _)| slot);
                self.kill_creature(i, by, true);
            }
        }
        self.mirror_echoes(fx.echoes);
        self.root_spikes(&fx.spikes, chars);
        self.root_walls(&fx.walls, chars);
        self.quicksand_patches(&fx.sands);
        self.drop_icicles(&fx.drops);
        // Helpers of the guardians, at most `max` at a time
        for (owner, name, max, pos) in fx.summons {
            let Some(kind) = self.creature_kind(&name) else {
                continue;
            };
            let alive = self.creatures.iter().filter(|c| c.kind == kind).count();
            if alive < max as usize
                && let Some(id) = self.add_creature(kind, pos)
            {
                self.events.push(Event::CreatureFire { id: owner, pos });
                let _ = id;
            }
        }
    }
}

/// Runs the behaviour of the creature in `cx` for one tick.
fn dispatch(cx: &mut Ctx) -> Step {
    match cx.kind.behavior {
        Behavior::Walker {
            speed,
            turn_at_edges,
        } => ground::walker(cx, speed, turn_at_edges),
        Behavior::Hopper {
            wait_ms,
            jump_x,
            jump_y,
            sight,
        } => ground::hopper(cx, wait_ms, jump_x, jump_y, sight),
        Behavior::Turret {
            interval_ms,
            range,
            shot_speed,
            shot_damage,
            lob,
        } => turret::tick(cx, interval_ms, range, shot_speed, shot_damage, lob),
        Behavior::Queen(ref d) => queen::tick(cx, d),
        Behavior::Diver(ref d) => diver::tick(cx, d),
        Behavior::Warden(ref d) => warden::tick(cx, d),
        Behavior::Burrower {
            sight,
            out_ms,
            hide_ms,
            rise_ms,
        } => ground::burrower(cx, sight, out_ms, hide_ms, rise_ms),
        Behavior::Follower { speed, jump } => ground::follower(cx, speed, jump),
        Behavior::Serpent(ref d) => serpent::tick(cx, d),
        Behavior::Leaper {
            sight,
            speed,
            warn_ms,
            jump_x,
            jump_y,
            rest_ms,
        } => leaper::tick(cx, sight, speed, warn_ms, jump_x, jump_y, rest_ms),
        Behavior::Icicle {
            sight,
            reach,
            warn_ms,
        } => air::icicle(cx, sight, reach, warn_ms),
        Behavior::Roller { speed, life_ms } => ground::roller(cx, speed, life_ms),
        Behavior::Seal {
            sight,
            speed,
            range,
            interval_ms,
            shot_speed,
            shot_damage,
        } => ground::seal(
            cx,
            sight,
            speed,
            range,
            interval_ms,
            shot_speed,
            shot_damage,
        ),
        Behavior::Bat {
            sight,
            reach,
            speed,
        } => air::bat(cx, sight, reach, speed),
        Behavior::Ghost {
            sight,
            speed,
            flee_ms,
        } => air::ghost(cx, sight, speed, flee_ms),
        Behavior::Flyer {
            speed,
            sight,
            hover,
            drop_ms,
            drop_damage,
            glow_ms,
        } => air::flyer(cx, speed, sight, hover, drop_ms, drop_damage, glow_ms),
        Behavior::Moth {
            sight,
            speed,
            flutter,
        } => night::moth(cx, sight, speed, flutter),
        Behavior::Goblin {
            sight,
            wake_ms,
            speed,
            daze_ms,
        } => night::goblin(cx, sight, wake_ms, speed, daze_ms),
        Behavior::Mirror {
            sight,
            ref spawn,
            rest_ms,
        } => night::mirror(cx, sight, spawn, rest_ms),
        Behavior::Echo { delay_ms, fade_ms } => night::echo(cx, delay_ms, fade_ms),
    }
}

impl World {
    /// Dark mirrors (R2-M2.5): send out their echoes, and rest once their echo is gone.
    fn mirror_echoes(&mut self, sent: Vec<(u32, String, Vec2)>) {
        for (mirror, name, pos) in sent {
            let echo = self
                .creature_kind(&name)
                .and_then(|kind| self.add_creature(kind, pos));
            if let Some(m) = self.creatures.iter_mut().find(|c| c.id == mirror) {
                m.count = echo.unwrap_or(0);
                if echo.is_none() {
                    m.mode = crate::creature::mirror::REST;
                    m.timer = 0;
                }
            }
            if let Some(id) = echo {
                self.events.push(Event::CreatureAct {
                    id,
                    pos,
                    act: crate::event::CreatureAct::Wake,
                });
            }
        }
        let alive: Vec<u32> = self.creatures.iter().map(|c| c.id).collect();
        for m in &mut self.creatures {
            if matches!(
                self.creature_kinds[m.kind].behavior,
                Behavior::Mirror { .. }
            ) && m.mode == crate::creature::mirror::OUT
                && !alive.contains(&m.count)
            {
                m.mode = crate::creature::mirror::REST;
                m.timer = 0;
            }
        }
    }
}

/// A creature that wanted to move sideways stopped at a wall: walkers turn, snow chunks burst,
/// rolling stone goblins are dazed.
fn bumped(kind: &CreatureKind, c: &mut Creature, fx: &mut Effects, events: &mut Vec<Event>) {
    match kind.behavior {
        Behavior::Walker { .. } => c.facing = -c.facing,
        Behavior::Roller { .. } => fx.died.push(c.id),
        Behavior::Goblin { daze_ms, .. } if c.mode == crate::creature::goblin::ROLL => {
            night::daze(c, daze_ms, events);
        }
        _ => {}
    }
}
