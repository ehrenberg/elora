//! Stationary shooter ([`Behavior::Turret`](crate::creature::Behavior::Turret)).

use super::behavior::{Ctx, Step};
use super::{SHOT_RADIUS, lob_velocity, sign};
use crate::creature::CreatureShot;
use crate::event::Event;
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// Stands still and shoots at Elora (straight or in an arc).
pub(super) fn tick(
    cx: &mut Ctx,
    interval_ms: u32,
    range: f32,
    shot_speed: f32,
    shot_damage: i32,
    lob: bool,
) -> Step {
    let c = &mut *cx.creature;
    c.vel = Vec2::ZERO;
    c.timer = c.timer.saturating_add(1);
    if cx.active
        && let Some((p, d)) = cx.target
        && d <= range
    {
        c.facing = sign(p.x - c.pos.x);
        if c.timer >= ms_to_ticks(interval_ms)
            && (lob || cx.collision.intersect_line(c.pos, p).is_none())
        {
            let dir = (p - c.pos).normalize();
            let (pos, vel, g) = if lob {
                let pos = c.pos + Vec2::new(0.0, -(cx.kind.radius() + SHOT_RADIUS));
                (
                    pos,
                    lob_velocity(pos, p, shot_speed, cx.gravity),
                    cx.gravity,
                )
            } else {
                (
                    c.pos + dir * (cx.kind.radius() + SHOT_RADIUS),
                    dir * shot_speed,
                    0.0,
                )
            };
            cx.shots.push(CreatureShot {
                owner: c.id,
                pos,
                vel,
                damage: shot_damage,
                ticks: 0,
                gravity: g,
                glow: 0,
                landed: false,
            });
            cx.events.push(Event::CreatureFire { id: c.id, pos });
            c.timer = 0;
        }
    }
    Step::Hold
}
