//! Guardian from the air ([`Behavior::Diver`](crate::creature::Behavior::Diver)).

use super::behavior::{Ctx, Step};
use super::{SHOT_RADIUS, paced_ticks, sign};
use crate::collision::Collision;
use crate::creature::{Creature, CreatureShot, DiverDef};
use crate::event::{CreatureAct, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// One tick of the behaviour: delegates to the state machine and collects its effects.
pub(super) fn tick(cx: &mut Ctx, d: &DiverDef) -> Step {
    let c = &mut *cx.creature;
    if let Some(s) = tick_diver(
        c,
        d,
        cx.kind.health,
        cx.target,
        cx.collision,
        cx.shots,
        cx.events,
    ) {
        cx.fx
            .summons
            .push((c.id, d.summon.clone(), d.summon_max, s));
    }
    if c.mode == crate::creature::diver::SLEEP {
        return Step::Hold;
    }
    Step::Move
}

/// One tick of the guardian from the air; returns a position when it calls a helper.
fn tick_diver(
    c: &mut Creature,
    d: &DiverDef,
    max_health: i32,
    target: Option<(Vec2, f32)>,
    collision: &Collision,
    shots: &mut Vec<CreatureShot>,
    events: &mut Vec<Event>,
) -> Option<Vec2> {
    use crate::creature::diver::{AIM, CIRCLE, DIVE, RISE, SLEEP, STUNNED};
    let life = c.health as f32 / max_health.max(1) as f32;
    let angry = d.enrage_at > 0.0 && life <= d.enrage_at;
    let pace = if angry { 1.35 } else { 1.0 };
    let ticks = |ms: u32| paced_ticks(ms, pace);
    c.stun = 0;
    c.timer = c.timer.saturating_add(1);
    let mut summon = None;
    // call helpers (in every awake state)
    if d.summon_at > 0.0
        && life <= d.summon_at
        && c.mode != SLEEP
        && d.summon_ms > 0
        && c.timer.is_multiple_of(ms_to_ticks(d.summon_ms).max(1))
    {
        summon = Some(c.pos + Vec2::new(f32::from(c.facing) * 40.0, -30.0));
    }
    match c.mode {
        SLEEP => {
            c.vel = Vec2::ZERO;
            if target.is_some_and(|(_, dist)| dist <= d.sight) {
                c.mode = CIRCLE;
                c.timer = 0;
                events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Wake,
                });
            }
        }
        CIRCLE => {
            diver_circle(c, d, pace, target.is_some(), shots, events);
        }
        AIM => {
            c.vel = Vec2::ZERO;
            if let Some((p, _)) = target {
                c.goal = p;
                c.facing = sign(p.x - c.pos.x);
            }
            if c.timer >= ticks(d.aim_ms) {
                c.mode = DIVE;
                c.timer = 0;
                events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Dive,
                });
            }
        }
        DIVE => {
            diver_dive(c, d, pace, angry, collision, events);
        }
        STUNNED => {
            c.vel = Vec2::new(0.0, 4.0);
            if c.timer >= ms_to_ticks(d.stun_ms) {
                c.mode = RISE;
                c.timer = 0;
            }
        }
        _ => {
            // RISE: back to the circle
            let to = c.home - c.pos;
            c.vel = if to.length() > 8.0 {
                to.normalize() * d.speed * 1.5 * pace
            } else {
                Vec2::ZERO
            };
            if to.length() <= 8.0 || c.timer > 250 {
                c.mode = CIRCLE;
                c.timer = 0;
            }
        }
    }
    summon
}

/// Circles above the start point, drops shots and starts aiming after a while.
fn diver_circle(
    c: &mut Creature,
    d: &DiverDef,
    pace: f32,
    has_target: bool,
    shots: &mut Vec<CreatureShot>,
    events: &mut Vec<Event>,
) {
    use crate::creature::diver::AIM;
    let ticks = |ms: u32| paced_ticks(ms, pace);
    // ellipse above the start point
    c.count = (c.count + 1) % 100_000;
    let a = c.count as f32 * d.speed * pace / d.circle[0].max(1.0);
    let want = c.home + Vec2::new(a.cos() * d.circle[0], a.sin() * d.circle[1]);
    c.vel = (want - c.pos) * 0.2;
    if c.vel.x.abs() > 0.3 {
        c.facing = sign(c.vel.x);
    }
    if d.drop_ms > 0 && c.timer.is_multiple_of(ticks(d.drop_ms)) {
        let pos = c.pos + Vec2::new(0.0, SHOT_RADIUS + 20.0);
        shots.push(CreatureShot {
            owner: c.id,
            pos,
            vel: Vec2::new(0.0, d.drop_speed),
            damage: d.drop_damage,
            ticks: 0,
            gravity: 0.0,
            glow: 0,
            landed: false,
        });
        events.push(Event::CreatureFire { id: c.id, pos });
    }
    if c.timer >= ticks(d.circle_ms) && has_target {
        c.mode = AIM;
        c.timer = 0;
        // from here on `count` counts the dives
        c.count = 0;
    }
}

/// Dives at the aimed point until it hits the ground (angry: once more right away).
fn diver_dive(
    c: &mut Creature,
    d: &DiverDef,
    pace: f32,
    angry: bool,
    collision: &Collision,
    events: &mut Vec<Event>,
) {
    use crate::creature::diver::{AIM, STUNNED};
    let to = c.goal - c.pos;
    let dir = if to.length() > 1.0 {
        to.normalize()
    } else {
        Vec2::new(0.0, 1.0)
    };
    c.vel = dir * d.dive_speed * pace;
    // ground reached or under way too long
    let ahead = c.pos + c.vel;
    if collision.is_solid(ahead) || c.grounded || c.timer > 150 {
        c.vel = Vec2::ZERO;
        events.push(Event::CreatureAct {
            id: c.id,
            pos: c.pos,
            act: CreatureAct::Land,
        });
        if angry && c.count == 0 {
            // once more right away
            c.count = 1;
            c.mode = AIM;
        } else {
            c.count = 0;
            c.mode = STUNNED;
        }
        c.timer = 0;
    }
}
