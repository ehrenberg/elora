//! Guardian on the ground ([`Behavior::Warden`](crate::creature::Behavior::Warden)).

use super::behavior::{Ctx, Step};
use super::sign;
use crate::creature::{Creature, WardenDef};
use crate::event::{CreatureAct, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;

/// One tick of the behaviour: delegates to the state machine and collects its effects.
pub(super) fn tick(cx: &mut Ctx, d: &WardenDef) -> Step {
    let c = &mut *cx.creature;
    c.vel = Vec2::ZERO;
    let feet = c.pos.y + cx.size.y / 2.0;
    let out = tick_warden(
        c,
        d,
        cx.kind.health,
        cx.target,
        feet,
        cx.tuggers.contains(&c.id),
        cx.events,
    );
    cx.fx.spikes.extend(
        out.0
            .into_iter()
            .map(|p| (p, d.spike_width, d.spike_height, d.spike_damage)),
    );
    if let Some(p) = out.1 {
        cx.fx.walls.push((p, d.wall_height, d.wall_ms));
    }
    Step::Hold
}

/// One tick of the ground guardian; returns root thrusts (centre on the ground) and a root wall.
fn tick_warden(
    c: &mut Creature,
    d: &WardenDef,
    max_health: i32,
    target: Option<(Vec2, f32)>,
    feet: f32,
    tugged: bool,
    events: &mut Vec<Event>,
) -> (Vec<Vec2>, Option<Vec2>) {
    use crate::creature::warden::{IDLE, OPEN, SLEEP, WARN};
    #[allow(clippy::cast_precision_loss)]
    let life = c.health as f32 / max_health.max(1) as f32;
    let angry = d.enrage_at > 0.0 && life <= d.enrage_at;
    let pace = if angry { 1.4 } else { 1.0 };
    let ticks = |ms: u32| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let t = (ms_to_ticks(ms) as f32 / pace) as u32;
        t.max(1)
    };
    let mut spikes = Vec::new();
    let mut wall = None;
    c.timer = c.timer.saturating_add(1);
    if let Some((p, _)) = target {
        c.facing = sign(p.x - c.pos.x);
    }
    let act = |events: &mut Vec<Event>, pos: Vec2, act: CreatureAct| {
        events.push(Event::CreatureAct { id: c.id, pos, act });
    };
    match c.mode {
        SLEEP => {
            if target.is_some_and(|(_, dist)| dist <= d.sight) {
                c.mode = IDLE;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Wake);
            }
            return (spikes, wall);
        }
        OPEN => {
            c.tug = 0;
            if c.timer >= ms_to_ticks(d.open_ms) {
                c.mode = IDLE;
                c.timer = 0;
                if c.count == 0 {
                    c.count = d.cores;
                }
            }
            return (spikes, wall);
        }
        _ => {}
    }
    // tug-of-war: core comes loose
    c.tug = if tugged {
        c.tug + 1
    } else {
        c.tug.saturating_sub(2)
    };
    if c.tug >= ms_to_ticks(d.pull_ms) && c.count > 0 {
        c.count -= 1;
        c.tug = 0;
        c.mode = OPEN;
        c.timer = 0;
        act(events, c.pos, CreatureAct::Core);
        return (spikes, wall);
    }
    match c.mode {
        WARN => {
            if c.timer >= ticks(d.warn_ms) {
                spikes.push(c.goal);
                act(events, c.goal, CreatureAct::Strike);
                // last core: right away one more spot closer to the warden
                if c.count == 1 {
                    let toward = sign(c.pos.x - c.goal.x);
                    let second =
                        c.goal + Vec2::new(f32::from(toward) * 3.0 * crate::TILE_SIZE as f32, 0.0);
                    spikes.push(second);
                    act(events, second, CreatureAct::Strike);
                }
                c.mode = IDLE;
                c.timer = 0;
            }
        }
        _ => {
            if let Some((p, _)) = target
                && c.timer >= ticks(d.attack_ms)
            {
                c.goal = Vec2::new(p.x, feet);
                c.mode = WARN;
                c.timer = 0;
                act(events, c.goal, CreatureAct::Warn);
            }
        }
    }
    // root walls when angry: between Elora and the warden
    if angry && d.wall_every_ms > 0 {
        c.wall_timer += 1;
        if c.wall_timer >= ms_to_ticks(d.wall_every_ms)
            && let Some((p, _)) = target
        {
            c.wall_timer = 0;
            let mid = Vec2::new(f32::midpoint(p.x, c.pos.x), feet);
            if (mid.x - c.pos.x).abs() > 64.0 && (mid.x - p.x).abs() > 48.0 {
                wall = Some(mid);
            }
        }
    }
    (spikes, wall)
}
