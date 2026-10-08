//! Behaviours in the air and on the ceiling: icicle, bat, frost ghost and flyer.

use super::behavior::{Ctx, Step};
use super::{SHOT_RADIUS, below, sign};
use crate::creature::CreatureShot;
use crate::event::{CreatureAct, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// Hangs, trembles and falls ([`Behavior::Icicle`](crate::creature::Behavior::Icicle)).
pub(super) fn icicle(cx: &mut Ctx, sight: f32, reach: f32, warn_ms: u32) -> Step {
    use crate::creature::icicle::{FALL, HANG, SHAKE};
    let c = &mut *cx.creature;
    let mut fixed = false;
    let id = c.id;
    c.vel.x = 0.0;
    match c.mode {
        HANG => {
            fixed = true;
            c.vel = Vec2::ZERO;
            let tip = c.pos + Vec2::new(0.0, cx.size.y / 2.0 + 2.0);
            if cx.active && below(cx.collision, cx.chars, tip, sight, reach) {
                c.mode = SHAKE;
                c.timer = 0;
                cx.events.push(Event::CreatureAct {
                    id,
                    pos: c.pos,
                    act: CreatureAct::Warn,
                });
            }
        }
        SHAKE => {
            fixed = true;
            c.vel = Vec2::ZERO;
            c.timer += 1;
            if c.timer >= ms_to_ticks(warn_ms) {
                c.mode = FALL;
                c.timer = 0;
                cx.events.push(Event::CreatureAct {
                    id,
                    pos: c.pos,
                    act: CreatureAct::Dive,
                });
            }
        }
        _ => {
            c.timer += 1;
            c.vel.y += cx.gravity;
            if c.grounded && c.timer > 2 {
                cx.fx.died.push(c.id);
            }
        }
    }
    if fixed { Step::Hold } else { Step::Move }
}

/// Swoops down from the ceiling ([`Behavior::Bat`](crate::creature::Behavior::Bat)).
pub(super) fn bat(cx: &mut Ctx, sight: f32, reach: f32, speed: f32) -> Step {
    use crate::creature::bat::{DIVE, HANG, RETURN};
    let c = &mut *cx.creature;
    let mut fixed = false;
    c.timer = c.timer.saturating_add(1);
    match c.mode {
        HANG => {
            fixed = true;
            c.vel = Vec2::ZERO;
            c.pos = c.home;
            // brief rest after returning
            if cx.active
                && c.timer > 50
                && let Some((p, _)) = cx.target
                && below(cx.collision, cx.chars, c.pos, sight, reach)
            {
                c.mode = DIVE;
                c.timer = 0;
                c.goal = p;
                cx.events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Dive,
                });
            }
        }
        DIVE => {
            let to = c.goal - c.pos;
            if to.length() < 12.0 || c.timer > 90 || (c.timer > 3 && c.vel.length() < 0.5) {
                c.mode = RETURN;
                c.timer = 0;
            } else {
                c.vel = to.normalize() * speed;
            }
        }
        _ => {
            let to = c.home - c.pos;
            if to.length() < speed {
                c.pos = c.home;
                c.vel = Vec2::ZERO;
                c.mode = HANG;
                c.timer = 0;
                fixed = true;
            } else {
                c.vel = to.normalize() * speed * 0.6;
                fixed = c.timer > 120; // stuck: home through walls
                if fixed {
                    c.pos += c.vel;
                }
            }
        }
    }
    if c.vel.x.abs() > 0.5 {
        c.facing = sign(c.vel.x);
    }
    if fixed { Step::Hold } else { Step::Move }
}

/// Floats through walls ([`Behavior::Ghost`](crate::creature::Behavior::Ghost)).
pub(super) fn ghost(cx: &mut Ctx, sight: f32, speed: f32, flee_ms: u32) -> Step {
    use crate::creature::ghost::{CHASE, FLEE};
    let c = &mut *cx.creature;
    // floats through everything
    c.timer = c.timer.saturating_add(1);
    let seen = cx.target.filter(|&(_, d)| d <= sight && cx.active);
    let want = match (c.mode, seen) {
        (FLEE, Some((p, _))) if c.timer < ms_to_ticks(flee_ms) => {
            let away = c.pos - p;
            (Vec2::new(away.x, away.y.min(0.0) - 40.0)).normalize() * speed * 1.4
        }
        (_, Some((p, _))) => {
            c.mode = CHASE;
            (p - c.pos).normalize() * speed
        }
        (_, None) => {
            c.mode = CHASE;
            let home = c.home - c.pos;
            if home.length() > 4.0 {
                home.normalize() * speed * 0.5
            } else {
                Vec2::ZERO
            }
        }
    };
    c.vel = c.vel * 0.9 + want * 0.1;
    c.pos += c.vel;
    if c.vel.x.abs() > 0.2 {
        c.facing = sign(c.vel.x);
    }
    Step::Hold
}

/// Hovers and chases Elora, drops sparks ([`Behavior::Flyer`](crate::creature::Behavior::Flyer)).
pub(super) fn flyer(
    cx: &mut Ctx,
    speed: f32,
    sight: f32,
    hover: f32,
    drop_ms: u32,
    drop_damage: i32,
    glow_ms: u32,
) -> Step {
    let c = &mut *cx.creature;
    let chase = cx.target.filter(|&(_, d)| d <= sight && cx.active);
    let goal = chase.map_or(c.home, |(p, _)| p - Vec2::new(0.0, hover));
    c.timer = c.timer.saturating_add(1);
    // drop sparks when above Elora
    if drop_ms > 0
        && let Some((p, _)) = chase
        && (p.x - c.pos.x).abs() < 48.0
        && p.y > c.pos.y
        && c.timer >= ms_to_ticks(drop_ms)
    {
        let pos = c.pos + Vec2::new(0.0, cx.size.y / 2.0 + SHOT_RADIUS);
        cx.shots.push(CreatureShot {
            owner: c.id,
            pos,
            vel: Vec2::new(c.vel.x * 0.5, 1.0),
            damage: drop_damage,
            ticks: 0,
            gravity: cx.gravity * 0.4,
            glow: ms_to_ticks(glow_ms),
            landed: false,
        });
        cx.events.push(Event::CreatureFire { id: c.id, pos });
        c.timer = 0;
    }
    let to = goal - c.pos;
    let want = if to.length() > 4.0 {
        to.normalize() * speed
    } else {
        Vec2::ZERO
    };
    c.vel = if cx.active {
        c.vel * 0.85 + want * 0.15
    } else {
        c.vel * 0.9
    };
    if to.x.abs() > 1.0 {
        c.facing = sign(to.x);
    }
    Step::Move
}
