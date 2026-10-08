//! Dune worm ([`Behavior::Leaper`](crate::creature::Behavior::Leaper)).

use super::behavior::{Ctx, Step};
use super::{floor_at, sign};
use crate::collision::Collision;
use crate::creature::Creature;
use crate::event::{CreatureAct, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// One tick of the behaviour (only aims at Elora while not stunned).
pub(super) fn tick(
    cx: &mut Ctx,
    sight: f32,
    speed: f32,
    warn_ms: u32,
    jump_x: f32,
    jump_y: f32,
    rest_ms: u32,
) -> Step {
    let d = LeaperDef {
        sight,
        speed,
        warn_ms,
        jump_x,
        jump_y,
        rest_ms,
    };
    tick_leaper(
        cx.creature,
        &d,
        cx.size,
        cx.gravity,
        if cx.active { cx.target } else { None },
        cx.collision,
        cx.events,
    );
    Step::Move
}

/// Values of the dune worm (from [`Behavior::Leaper`]).
struct LeaperDef {
    sight: f32,
    speed: f32,
    warn_ms: u32,
    jump_x: f32,
    jump_y: f32,
    rest_ms: u32,
}

/// One tick of the dune worm: approach under the sand, warning, arcing leap onto the spot
/// where Elora stood at the warning, landing and submerging.
fn tick_leaper(
    c: &mut Creature,
    d: &LeaperDef,
    size: Vec2,
    gravity: f32,
    target: Option<(Vec2, f32)>,
    collision: &Collision,
    events: &mut Vec<Event>,
) {
    use crate::creature::leaper::{LEAP, UNDER, WARN};
    c.vel.y += gravity;
    c.timer = c.timer.saturating_add(1);
    // flight time of a leap back to the starting height
    let air = 2.0 * d.jump_y / gravity.max(0.01);
    let act = |events: &mut Vec<Event>, pos: Vec2, act: CreatureAct| {
        events.push(Event::CreatureAct { id: c.id, pos, act });
    };
    match c.mode {
        UNDER => {
            let Some((p, _)) = target.filter(|&(_, dist)| dist <= d.sight) else {
                c.vel.x *= 0.8;
                return;
            };
            let dx = p.x - c.pos.x;
            c.facing = sign(dx);
            if c.grounded
                && c.timer >= ms_to_ticks(d.rest_ms)
                && dx.abs() <= d.jump_x * air
                && (p.y - c.pos.y).abs() < 160.0
            {
                c.mode = WARN;
                c.timer = 0;
                c.goal = p;
                c.vel.x = 0.0;
                act(events, c.pos, CreatureAct::Warn);
                return;
            }
            let ahead = Vec2::new(
                c.pos.x + f32::from(c.facing) * (size.x / 2.0 + 2.0),
                c.pos.y + size.y / 2.0 + 4.0,
            );
            let edge = c.grounded && !floor_at(collision, ahead);
            c.vel.x = if edge || dx.abs() < 8.0 {
                0.0
            } else {
                f32::from(c.facing) * d.speed
            };
        }
        WARN => {
            c.vel.x = 0.0;
            if c.timer >= ms_to_ticks(d.warn_ms) {
                let dx = c.goal.x - c.pos.x;
                c.vel = Vec2::new((dx / air).clamp(-d.jump_x, d.jump_x), -d.jump_y);
                c.mode = LEAP;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Emerge);
            }
        }
        _ => {
            if c.grounded && c.timer > 4 && c.vel.y >= 0.0 {
                c.vel.x = 0.0;
                c.mode = UNDER;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Burrow);
            }
        }
    }
}
