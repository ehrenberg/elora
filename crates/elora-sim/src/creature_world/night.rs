//! Creatures of the night (R2-M2.5, E-365): shadow moth, stone goblin and echo.

use super::behavior::{Ctx, Step};
use super::sign;
use crate::event::{CreatureAct, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// Is `p` inside one of the light spots?
fn lit(lights: &[(Vec2, f32)], p: Vec2) -> bool {
    lights.iter().any(|&(c, r)| c.distance(p) <= r)
}

/// The light spot closest to `p`.
fn nearest_light(lights: &[(Vec2, f32)], p: Vec2) -> Option<Vec2> {
    lights
        .iter()
        .map(|&(c, _)| c)
        .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)))
}

/// Flutters, attacks in the dark, flees from light
/// ([`Behavior::Moth`](crate::creature::Behavior::Moth)).
pub(super) fn moth(cx: &mut Ctx, sight: f32, speed: f32, flutter: f32) -> Step {
    use crate::creature::moth::{ATTACK, FLEE, FLUTTER};
    let c = &mut *cx.creature;
    c.timer = c.timer.wrapping_add(1);
    // every moth of the swarm flutters on its own figure
    let phase = c.timer as f32 * 0.05 + c.id as f32 * 1.7;
    let wobble = Vec2::new((phase * 1.3).sin(), (phase * 2.1).cos() * 0.6) * flutter;
    let in_light = lit(cx.lights, c.pos);
    let prey = cx
        .target
        .filter(|&(p, d)| cx.active && d <= sight && !lit(cx.lights, p));
    let want = if in_light {
        c.mode = FLEE;
        let from = nearest_light(cx.lights, c.pos).unwrap_or(c.pos + Vec2::new(0.0, 1.0));
        let away = (c.pos - from).normalize();
        let away = if away == Vec2::ZERO {
            Vec2::new(0.0, -1.0)
        } else {
            away
        };
        away * speed * 1.3
    } else if let Some((p, _)) = prey {
        c.mode = ATTACK;
        (p + wobble * 0.3 - c.pos).normalize() * speed
    } else {
        c.mode = FLUTTER;
        let to = c.home + wobble - c.pos;
        to.normalize() * speed * 0.5 * (to.length() / 40.0).min(1.0)
    };
    c.vel = c.vel * 0.85 + want * 0.15;
    // never fly into the light on purpose
    if !in_light && lit(cx.lights, c.pos + c.vel * 6.0) {
        c.vel *= -0.5;
    }
    if c.vel.x.abs() > 0.3 {
        c.facing = sign(c.vel.x);
    }
    Step::Move
}

/// Rock, wakes up, rolls at Elora, dazed when stopped
/// ([`Behavior::Goblin`](crate::creature::Behavior::Goblin)).
pub(super) fn goblin(cx: &mut Ctx, sight: f32, wake_ms: u32, speed: f32, daze_ms: u32) -> Step {
    use crate::creature::goblin::{ROCK, ROLL, WAKE};
    let c = &mut *cx.creature;
    c.vel.y += cx.gravity;
    match c.mode {
        ROCK => {
            c.vel.x = 0.0;
            if cx.active
                && let Some((p, d)) = cx.target
                && d <= sight
            {
                c.mode = WAKE;
                c.timer = 0;
                c.facing = sign(p.x - c.pos.x);
                cx.events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Warn,
                });
            }
        }
        WAKE => {
            c.vel.x = 0.0;
            c.timer += 1;
            if c.timer >= ms_to_ticks(wake_ms) {
                c.mode = ROLL;
                c.timer = 0;
                c.count = 0;
                cx.events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Wake,
                });
            }
        }
        ROLL => {
            // a hit (this or the last tick) or a stomp stops it
            if !cx.active || c.hit_tick.is_some_and(|t| t + 1 >= cx.tick) {
                daze(c, daze_ms, cx.events);
                return Step::Move;
            }
            // turns towards Elora only after a moment (it overshoots)
            if let Some((p, _)) = cx.target {
                let want = sign(p.x - c.pos.x);
                if want == c.facing || (p.x - c.pos.x).abs() < 8.0 {
                    c.count = 0;
                } else {
                    c.count += 1;
                    if c.count > 25 {
                        c.facing = want;
                        c.count = 0;
                    }
                }
            }
            let goal = f32::from(c.facing) * speed;
            c.vel.x += (goal - c.vel.x).clamp(-0.3, 0.3);
        }
        _ => {
            c.vel.x *= 0.8;
            if cx.active {
                c.mode = ROCK;
            }
        }
    }
    Step::Move
}

/// Stops a rolling goblin: dazed for `daze_ms` (also when it hits a wall).
pub(super) fn daze(c: &mut crate::creature::Creature, daze_ms: u32, events: &mut Vec<Event>) {
    c.mode = crate::creature::goblin::DAZED;
    c.vel.x = 0.0;
    c.stun = c.stun.max(ms_to_ticks(daze_ms));
    events.push(Event::CreatureAct {
        id: c.id,
        pos: c.pos,
        act: CreatureAct::Land,
    });
}

/// Waits in its mirror, walks Elora's path with a delay, fades in the light
/// ([`Behavior::Echo`](crate::creature::Behavior::Echo)).
pub(super) fn echo(cx: &mut Ctx, sight: f32, delay_ms: u32, fade_ms: u32) -> Step {
    use crate::creature::echo::{FADE, FOLLOW, MIRROR};
    let c = &mut *cx.creature;
    let delay = ms_to_ticks(delay_ms).max(1) as usize;
    let before = c.pos;
    // the mirror stays where it is (drawn at `goal`)
    c.goal = c.home;
    match c.mode {
        MIRROR => {
            c.pos = c.home;
            if cx.active
                && let Some((_, d)) = cx.target
                && d <= sight
            {
                c.mode = FOLLOW;
                c.timer = 0;
                c.trail.clear();
                cx.events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Wake,
                });
            }
        }
        FOLLOW => {
            if let Some((p, _)) = cx.target {
                c.trail.push_back(p);
            }
            if c.trail.len() > delay {
                if let Some(next) = c.trail.pop_front() {
                    c.pos = next;
                }
            } else if let Some(&first) = c.trail.front() {
                // steps out of the mirror to where Elora was when it saw her
                let left = (delay + 1 - c.trail.len()) as f32;
                c.pos = c.pos + (first - c.pos) / left;
            }
            if lit(cx.lights, c.pos) {
                c.mode = FADE;
                c.timer = 0;
            }
        }
        _ => {
            c.timer += 1;
            if c.timer >= ms_to_ticks(fade_ms) {
                cx.fx.defeated.push(c.id);
            }
        }
    }
    c.vel = c.pos - before;
    if c.vel.x.abs() > 0.3 {
        c.facing = sign(c.vel.x);
    }
    Step::Hold
}
