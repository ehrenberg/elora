//! Behaviours on the ground: walker, hopper, burrower, companion, snow chunk and seal.

use super::behavior::{Ctx, Step};
use super::{SHOT_RADIUS, floor_at, lob_velocity, sign};
use crate::creature::CreatureShot;
use crate::event::{CreatureAct, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// Walks back and forth ([`Behavior::Walker`](crate::creature::Behavior::Walker)).
pub(super) fn walker(cx: &mut Ctx, speed: f32, turn_at_edges: bool) -> Step {
    let c = &mut *cx.creature;
    c.vel.y += cx.gravity;
    if cx.active {
        let ahead = Vec2::new(
            c.pos.x + f32::from(c.facing) * (cx.size.x / 2.0 + 2.0),
            c.pos.y + cx.size.y / 2.0 + 4.0,
        );
        if turn_at_edges && c.grounded && !floor_at(cx.collision, ahead) {
            c.facing = -c.facing;
        }
        c.vel.x = f32::from(c.facing) * speed;
    } else {
        c.vel.x *= 0.8;
    }
    Step::Move
}

/// Waits on the ground and leaps at Elora ([`Behavior::Hopper`](crate::creature::Behavior::Hopper)).
pub(super) fn hopper(cx: &mut Ctx, wait_ms: u32, jump_x: f32, jump_y: f32, sight: f32) -> Step {
    let c = &mut *cx.creature;
    c.vel.y += cx.gravity;
    if c.grounded {
        c.vel.x *= 0.5;
        if cx.active {
            c.timer += 1;
            if let Some((p, d)) = cx.target
                && d <= sight
                && c.timer >= ms_to_ticks(wait_ms)
            {
                c.facing = sign(p.x - c.pos.x);
                c.vel = Vec2::new(f32::from(c.facing) * jump_x, -jump_y);
                c.timer = 0;
            }
        }
    }
    Step::Move
}

/// Hides in the ground and shoots up ([`Behavior::Burrower`](crate::creature::Behavior::Burrower)).
pub(super) fn burrower(cx: &mut Ctx, sight: f32, out_ms: u32, hide_ms: u32, rise_ms: u32) -> Step {
    use crate::creature::burrow::{HIDDEN, OUT, RISING};
    let c = &mut *cx.creature;
    c.vel = Vec2::ZERO;
    c.timer = c.timer.saturating_add(1);
    if let Some((p, _)) = cx.target {
        c.facing = sign(p.x - c.pos.x);
    }
    match c.mode {
        HIDDEN => {
            if cx.active
                && c.timer >= ms_to_ticks(hide_ms)
                && cx.target.is_some_and(|(_, d)| d <= sight)
            {
                c.mode = RISING;
                c.timer = 0;
                cx.events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Emerge,
                });
            }
        }
        RISING => {
            if c.timer >= ms_to_ticks(rise_ms) {
                c.mode = OUT;
                c.timer = 0;
            }
        }
        _ => {
            if c.timer >= ms_to_ticks(out_ms) {
                c.mode = HIDDEN;
                c.timer = 0;
                cx.events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Burrow,
                });
            }
        }
    }
    Step::Hold
}

/// Companion that follows Elora ([`Behavior::Follower`](crate::creature::Behavior::Follower)).
pub(super) fn follower(cx: &mut Ctx, speed: f32, jump: f32) -> Step {
    let c = &mut *cx.creature;
    c.vel.y += cx.gravity;
    let Some((p, dist)) = cx.target else {
        c.vel.x *= 0.7;
        return Step::Hold;
    };
    if dist > 64.0 && (p.x - c.pos.x).abs() > 24.0 {
        let dir = sign(p.x - c.pos.x);
        c.facing = dir;
        let front = c.pos.x + f32::from(dir) * (cx.size.x / 2.0 + 4.0);
        let feet = c.pos.y + cx.size.y / 2.0;
        // Gap or hazard ahead of the feet: wait (E-308). Below the
        // tip of the foot the first tile (up to 2 tiles deep) must carry.
        let below = (0..5).find_map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let y = feet + 4.0 + k as f32 * 16.0;
            let t = cx.collision.tile_at(Vec2::new(front, y));
            (t != crate::Tile::Air).then_some(t)
        });
        let danger = below == Some(crate::Tile::Death);
        let ground_ahead = below.is_some() && !danger;
        let wall = cx.collision.is_solid(Vec2::new(front, feet - 8.0));
        if c.grounded && (danger || (!ground_ahead && p.y <= c.pos.y + 32.0)) {
            c.vel.x = 0.0;
        } else {
            c.vel.x = f32::from(dir) * speed;
            if wall && c.grounded {
                c.vel.y = -jump;
            }
        }
    } else {
        c.vel.x *= 0.7;
    }
    Step::Move
}

/// Snow chunk of an avalanche ([`Behavior::Roller`](crate::creature::Behavior::Roller)).
pub(super) fn roller(cx: &mut Ctx, speed: f32, life_ms: u32) -> Step {
    let c = &mut *cx.creature;
    c.timer += 1;
    c.vel.y += cx.gravity;
    c.vel.x = f32::from(c.facing) * speed;
    if c.timer >= ms_to_ticks(life_ms) {
        cx.fx.died.push(c.id);
    }
    Step::Move
}

/// Slides closer and throws snowballs ([`Behavior::Seal`](crate::creature::Behavior::Seal)).
pub(super) fn seal(
    cx: &mut Ctx,
    sight: f32,
    speed: f32,
    range: f32,
    interval_ms: u32,
    shot_speed: f32,
    shot_damage: i32,
) -> Step {
    use crate::creature::seal::{SLIDE, THROW};
    let c = &mut *cx.creature;
    c.vel.y += cx.gravity;
    c.timer = c.timer.saturating_add(1);
    let seen = cx.target.filter(|&(_, d)| d <= sight && cx.active);
    match (c.mode, seen) {
        (THROW, Some((p, d))) if d <= range * 1.3 => {
            c.vel.x *= 0.7;
            c.facing = sign(p.x - c.pos.x);
            if c.timer >= ms_to_ticks(interval_ms) {
                let pos = c.pos + Vec2::new(0.0, -(cx.size.y / 2.0 + SHOT_RADIUS + 8.0));
                cx.shots.push(CreatureShot {
                    owner: c.id,
                    pos,
                    vel: lob_velocity(pos, p, shot_speed, cx.gravity),
                    damage: shot_damage,
                    ticks: 0,
                    gravity: cx.gravity,
                    glow: 0,
                    landed: false,
                });
                cx.events.push(Event::CreatureFire { id: c.id, pos });
                c.timer = 0;
            }
        }
        (_, Some((p, d))) if d <= range => {
            // within throwing range: rise up, aim briefly
            c.mode = THROW;
            c.timer = ms_to_ticks(interval_ms) / 2;
            c.vel.x *= 0.7;
            c.facing = sign(p.x - c.pos.x);
        }
        (_, Some((p, _))) => {
            c.mode = SLIDE;
            c.facing = sign(p.x - c.pos.x);
            let ahead = Vec2::new(
                c.pos.x + f32::from(c.facing) * (cx.size.x / 2.0 + 2.0),
                c.pos.y + cx.size.y / 2.0 + 4.0,
            );
            let edge = c.grounded && !floor_at(cx.collision, ahead);
            c.vel.x = if edge {
                0.0
            } else {
                f32::from(c.facing) * speed
            };
        }
        (_, None) => {
            c.mode = SLIDE;
            c.vel.x *= 0.9;
        }
    }
    Step::Move
}
