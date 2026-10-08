//! Guardian in the sand ([`Behavior::Serpent`](crate::creature::Behavior::Serpent)).

use super::behavior::{Ctx, Step};
use super::sign;
use crate::creature::{Creature, SerpentDef};
use crate::event::{CreatureAct, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// One tick of the behaviour: delegates to the state machine and collects its effects.
pub(super) fn tick(cx: &mut Ctx, d: &SerpentDef) -> Step {
    let c = &mut *cx.creature;
    let feet = c.pos.y + cx.size.y / 2.0;
    let (sand, slam) = tick_serpent(c, d, cx.kind.health, cx.gravity, feet, cx.target, cx.events);
    if let Some(p) = sand {
        cx.fx.sands.push((p, d.sand_width, d.sand_ms));
    }
    if let Some(p) = slam {
        cx.fx
            .spikes
            .push((p, d.land_radius * 2.0, 64.0, d.land_damage));
    }
    if c.mode == crate::creature::serpent::SLEEP {
        return Step::Hold;
    }
    Step::Move
}

/// One tick of the guardian in the sand; returns a spot for quicksand (angry) and the spot
/// of a landing that hurls sand.
#[allow(clippy::too_many_lines)]
fn tick_serpent(
    c: &mut Creature,
    d: &SerpentDef,
    max_health: i32,
    gravity: f32,
    feet: f32,
    target: Option<(Vec2, f32)>,
    events: &mut Vec<Event>,
) -> (Option<Vec2>, Option<Vec2>) {
    use crate::creature::serpent::{LEAP, SLEEP, STUNNED, TRAIL, WARN};
    #[allow(clippy::cast_precision_loss)]
    let life = c.health as f32 / max_health.max(1) as f32;
    let angry = d.enrage_at > 0.0 && life <= d.enrage_at;
    let double = d.double_at > 0.0 && life <= d.double_at;
    let pace = if angry { 1.35 } else { 1.0 };
    let ticks = |ms: u32| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let t = (ms_to_ticks(ms) as f32 / pace) as u32;
        t.max(1)
    };
    let id = c.id;
    let act = |events: &mut Vec<Event>, pos: Vec2, act: CreatureAct| {
        events.push(Event::CreatureAct { id, pos, act });
    };
    c.stun = 0;
    c.timer = c.timer.saturating_add(1);
    c.vel.y += gravity;
    let mut sand = None;
    let mut slam = None;
    // from half health she leaps straight at Elora
    let aim = target.map(|(p, _)| p).filter(|_| angry);
    match c.mode {
        SLEEP => {
            c.vel = Vec2::ZERO;
            if target.is_some_and(|(_, dist)| dist <= d.sight) {
                c.mode = TRAIL;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Wake);
            }
            return (None, None);
        }
        TRAIL => {
            let Some((p, _)) = target else {
                c.vel.x = 0.0;
                return (None, None);
            };
            let dx = p.x - c.pos.x;
            c.facing = sign(dx);
            c.vel.x = if dx.abs() < 6.0 {
                0.0
            } else {
                f32::from(c.facing) * (d.speed * pace).min(dx.abs())
            };
            if c.timer >= ticks(d.trail_ms) && dx.abs() < 20.0 && c.grounded {
                c.mode = WARN;
                c.timer = 0;
                c.goal = Vec2::new(c.pos.x, feet);
                c.vel.x = 0.0;
                act(events, c.goal, CreatureAct::Warn);
            }
        }
        WARN => {
            c.vel.x = 0.0;
            if c.timer >= ticks(d.warn_ms) {
                c.hits = 0;
                leap(c, d, gravity, pace, aim);
                act(events, c.pos, CreatureAct::Emerge);
            }
        }
        LEAP => {
            if c.vel.x.abs() > 0.1 {
                c.facing = sign(c.vel.x);
            }
            if c.grounded && c.timer > 4 && c.vel.y >= 0.0 {
                c.vel.x = 0.0;
                act(events, c.pos, CreatureAct::Land);
                if d.land_damage > 0 {
                    slam = Some(Vec2::new(c.pos.x, feet));
                }
                if double && c.count == 0 {
                    // another arc right away, now towards Elora
                    c.count = 1;
                    leap(c, d, gravity, pace, target.map(|(p, _)| p));
                    act(events, c.pos, CreatureAct::Emerge);
                } else {
                    c.count = 0;
                    c.mode = STUNNED;
                    c.timer = 0;
                }
            }
        }
        _ => {
            // STUNNED: dazed, then back under the sand – immediately after too many hits
            c.vel.x = 0.0;
            let too_many = d.open_hits > 0 && c.hits >= d.open_hits;
            if c.timer >= ms_to_ticks(d.stun_ms) || too_many {
                c.hits = 0;
                c.mode = TRAIL;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Burrow);
            }
        }
    }
    // angry: parts of the basin turn into quicksand, alternately left and right of Elora
    if angry && d.sand_every_ms > 0 && matches!(c.mode, TRAIL | WARN) {
        c.wall_timer += 1;
        if c.wall_timer >= ms_to_ticks(d.sand_every_ms)
            && let Some((p, _)) = target
        {
            c.wall_timer = 0;
            c.tug = 1 - c.tug.min(1);
            let side = if c.tug == 0 { -1.0 } else { 1.0 };
            #[allow(clippy::cast_precision_loss)]
            let at = Vec2::new(p.x + side * 5.0 * crate::TILE_SIZE as f32, feet);
            sand = Some(at);
        }
    }
    (sand, slam)
}

/// Leap of the sand serpent: up and to the centre of the basin (start point) – or with `aim`
/// such that she lands on that spot.
fn leap(c: &mut Creature, d: &SerpentDef, gravity: f32, pace: f32, aim: Option<Vec2>) {
    use crate::creature::serpent::LEAP;
    let vx = if let Some(p) = aim {
        // flight time back to the starting height
        let air = 2.0 * d.jump_y / gravity.max(0.01);
        let max = d.jump_x * 1.6;
        ((p.x - c.pos.x) / air).clamp(-max, max)
    } else {
        let toward = if (c.home.x - c.pos.x).abs() > 1.0 {
            sign(c.home.x - c.pos.x)
        } else {
            -c.facing
        };
        f32::from(toward) * d.jump_x * pace.sqrt()
    };
    if vx.abs() > 0.1 {
        c.facing = sign(vx);
    }
    c.vel = Vec2::new(vx, -d.jump_y);
    c.mode = LEAP;
    c.timer = 0;
}
