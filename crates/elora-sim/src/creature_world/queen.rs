//! Guardian of Frostspitzen ([`Behavior::Queen`](crate::creature::Behavior::Queen)).

use super::behavior::{Ctx, Step};
use super::{paced_ticks, sign};
use crate::collision::Collision;
use crate::creature::{Creature, QueenDef};
use crate::event::{CreatureAct, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// One tick of the behaviour: delegates to the state machine and collects its effects.
pub(super) fn tick(cx: &mut Ctx, d: &QueenDef) -> Step {
    let c = &mut *cx.creature;
    let out = tick_queen(
        c,
        d,
        cx.kind.health,
        cx.size,
        cx.target,
        cx.collision,
        cx.events,
    );
    cx.fx.spikes.extend(out.frost);
    if out.icicles > 0
        && let Some((p, _)) = cx.target
    {
        cx.fx.drops.push((p, out.icicles));
    }
    Step::Hold
}

/// Result of one tick of the guardian: fresh frost (like root thrusts) and icicles.
struct QueenOut {
    frost: Option<(Vec2, f32, f32, i32)>,
    icicles: u32,
}

/// One tick of the guardian of the Frostspitzen (E-341).
fn tick_queen(
    c: &mut Creature,
    d: &QueenDef,
    max_health: i32,
    size: Vec2,
    target: Option<(Vec2, f32)>,
    collision: &Collision,
    events: &mut Vec<Event>,
) -> QueenOut {
    use crate::creature::queen::{HOVER, SLEEP, TIRED, WAVE};
    let life = c.health as f32 / max_health.max(1) as f32;
    let angry = d.enrage_at > 0.0 && life <= d.enrage_at;
    let storm = d.storm_at > 0.0 && life <= d.storm_at;
    let pace = if storm {
        1.3
    } else if angry {
        1.15
    } else {
        1.0
    };
    let ticks = |ms: u32| paced_ticks(ms, pace);
    let id = c.id;
    let act = |events: &mut Vec<Event>, pos: Vec2, act: CreatureAct| {
        events.push(Event::CreatureAct { id, pos, act });
    };
    let mut out = QueenOut {
        frost: None,
        icicles: 0,
    };
    c.stun = 0;
    c.vel = Vec2::ZERO;
    c.timer = c.timer.saturating_add(1);
    let half = d.width / 2.0;
    let floor = c.goal.y;
    // announce the blizzard once (the session handles the weather)
    if storm && c.wall_timer == 0 && c.mode != SLEEP {
        c.wall_timer = 1;
        act(events, c.pos, CreatureAct::Storm);
    }
    match c.mode {
        SLEEP => {
            if target.is_some_and(|(_, dist)| dist <= d.sight) {
                // floor of the hall below the start point
                let mut y = c.home.y;
                while y < c.home.y + 1500.0 && !collision.tile_at(Vec2::new(c.home.x, y)).is_solid()
                {
                    y += 4.0;
                }
                let ts = crate::TILE_SIZE as f32;
                c.goal = Vec2::new(c.home.x, (y / ts).floor() * ts);
                c.mode = HOVER;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Wake);
            }
        }
        HOVER => {
            let t = c.timer as f32;
            c.pos = c.home + Vec2::new((t * 0.02).sin() * half * 0.45, (t * 0.05).sin() * 6.0);
            if let Some((p, _)) = target {
                c.facing = sign(p.x - c.pos.x);
            }
            if c.timer >= ticks(d.hover_ms) {
                // calm: the wave comes from the side Elora is not on;
                // angry: alternately from left and right
                let from_left = if angry {
                    c.count.is_multiple_of(2)
                } else {
                    target.is_none_or(|(p, _)| p.x >= c.home.x)
                };
                c.facing = if from_left { 1 } else { -1 };
                c.goal.x = c.home.x - f32::from(c.facing) * half;
                c.mode = WAVE;
                c.timer = 0;
                act(events, Vec2::new(c.goal.x, floor), CreatureAct::Warn);
            }
        }
        WAVE => {
            queen_wave(c, d, pace, angry, &mut out, events);
        }
        TIRED => {
            queen_tired(c, d, size, events);
        }
        _ => {
            let to = c.home - c.pos;
            if to.length() <= 4.0 {
                c.pos = c.home;
                c.mode = HOVER;
                c.timer = 0;
            } else {
                c.pos += to.normalize() * 4.0;
            }
        }
    }
    out
}

/// The frost wave runs through the hall; afterwards the guardian hovers again or tires.
fn queen_wave(
    c: &mut Creature,
    d: &QueenDef,
    pace: f32,
    angry: bool,
    out: &mut QueenOut,
    events: &mut Vec<Event>,
) {
    use crate::creature::queen::{HOVER, TIRED};
    let ticks = |ms: u32| paced_ticks(ms, pace);
    let half = d.width / 2.0;
    let floor = c.goal.y;
    let dir = f32::from(c.facing);
    c.goal.x += dir * d.wave_speed * pace;
    // fresh frost behind the front, inside the hall
    let (a, b) = (c.goal.x - dir * d.fresh_len, c.goal.x);
    let (lo, hi) = (a.min(b).max(c.home.x - half), a.max(b).min(c.home.x + half));
    if hi > lo {
        out.frost = Some((
            Vec2::new(f32::midpoint(lo, hi), floor),
            hi - lo,
            22.0,
            d.wave_damage,
        ));
    }
    if (c.goal.x - c.home.x) * dir > half + d.fresh_len {
        c.count += 1;
        c.timer = 0;
        if c.count.is_multiple_of(d.waves.max(1)) {
            c.mode = TIRED;
            c.hits = 0;
            events.push(Event::CreatureAct {
                id: c.id,
                pos: c.pos,
                act: CreatureAct::Land,
            });
        } else {
            c.mode = HOVER;
            // short pause between two waves
            c.timer = ticks(d.hover_ms) * 2 / 3;
            if angry {
                out.icicles = d.icicles;
            }
        }
    }
}

/// Sinks down exhausted and lies there until rested or hit often enough.
fn queen_tired(c: &mut Creature, d: &QueenDef, size: Vec2, events: &mut Vec<Event>) {
    use crate::creature::queen::RISE;
    let floor = c.goal.y;
    let rest = Vec2::new(c.pos.x, floor - size.y / 2.0);
    let to = rest - c.pos;
    if to.length() > 9.0 {
        c.pos += to.normalize() * 9.0;
        // exhaustion only counts from the landing
        c.timer = 0;
    } else {
        c.pos = rest;
    }
    // took enough hits: straight back up (playtest: too easy)
    let enough = d.open_hits > 0 && c.hits >= d.open_hits;
    if c.timer >= ms_to_ticks(d.stun_ms) || enough {
        c.mode = RISE;
        c.timer = 0;
        events.push(Event::CreatureAct {
            id: c.id,
            pos: c.pos,
            act: CreatureAct::Wake,
        });
    }
}
