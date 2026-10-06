//! Lawinen (R2-M2.4, E-343, D-M24-06): Zonen `lawine…` in der Karte sind Lawinenhänge. Stampfen
//! oder eine Granate im Hang – oder ein Schritt in die Zone `<hang>-tritt` – lässt Schnee
//! abgehen: A-39 Schneebrocken rollen im Abstand A-40 vom oberen Ende hangabwärts; danach
//! ruht der Hang A-41 lang.

use std::collections::BTreeMap;

use elora_map::{Map, ObjectKind};
use elora_sim::tuning::ms_to_ticks;
use elora_sim::{Event, TILE_SIZE, Vec2, World};

/// Name der Gegnerart für die Brocken (`creatures.toml`).
pub const ROCK: &str = "schneebrocken";
/// Anfang der Zonen-Namen; `-tritt` am Ende macht die Zone zur Auslöse-Stelle.
const PREFIX: &str = "lawine";
const STEP: &str = "-tritt";

/// Laufende und ruhende Lawinen der Karte.
#[derive(Debug, Clone, Default)]
pub struct Avalanches {
    /// Hang → laufende Lawine.
    running: BTreeMap<String, Run>,
    /// Hang → Tick, bis zu dem er ruht.
    rest: BTreeMap<String, u64>,
}

#[derive(Debug, Clone)]
struct Run {
    /// Start der Brocken (oberes Ende des Hangs) und Richtung hangabwärts.
    from: Vec2,
    dir: i8,
    left: u32,
    next: u64,
}

fn inside(p: Vec2, at: Vec2, size: Vec2) -> bool {
    p.x >= at.x && p.y >= at.y && p.x <= at.x + size.x && p.y <= at.y + size.y
}

/// Oberkante des Bodens unter `x` innerhalb der Zone (sonst ihre Unterkante).
fn ground(world: &World, x: f32, top: f32, bottom: f32) -> f32 {
    let ts = TILE_SIZE as f32;
    let mut y = top;
    while y < bottom {
        if world.collision.tile_at(Vec2::new(x, y)).is_solid() {
            return (y / ts).floor() * ts;
        }
        y += ts / 2.0;
    }
    bottom
}

impl Avalanches {
    /// Nach dem Tick der Welt: Auslöser prüfen, Brocken losschicken. `me` = Elora.
    pub fn tick(&mut self, map: &Map, world: &mut World, me: Option<Vec2>) {
        let now = world.tick;
        let blasts: Vec<Vec2> = world
            .events
            .iter()
            .filter_map(|e| match *e {
                Event::Stomp { pos, .. } | Event::Explosion { pos, .. } => Some(pos),
                _ => None,
            })
            .collect();
        for o in &map.adventure.objects {
            let ObjectKind::Zone { size } = o.kind else {
                continue;
            };
            if !o.id.starts_with(PREFIX) || o.id.ends_with(STEP) {
                continue;
            }
            let step = map
                .adventure
                .objects
                .iter()
                .find(|s| s.id == format!("{}{STEP}", o.id))
                .and_then(|s| match s.kind {
                    ObjectKind::Zone { size } => Some((s.pos, size)),
                    _ => None,
                });
            let triggered = blasts.iter().any(|&p| inside(p, o.pos, size))
                || step.is_some_and(|(at, sz)| me.is_some_and(|p| inside(p, at, sz)));
            let resting = self.rest.get(&o.id).is_some_and(|&t| now < t);
            if triggered && !resting && !self.running.contains_key(&o.id) {
                let inset = TILE_SIZE as f32;
                let (left, right) = (o.pos.x + inset, o.pos.x + size.x - inset);
                let bottom = o.pos.y + size.y;
                let (gl, gr) = (
                    ground(world, left, o.pos.y, bottom),
                    ground(world, right, o.pos.y, bottom),
                );
                // oben ist, wo der Boden höher liegt
                let (x, dir) = if gl <= gr { (left, 1) } else { (right, -1) };
                self.running.insert(
                    o.id.clone(),
                    Run {
                        from: Vec2::new(x, o.pos.y + inset),
                        dir,
                        left: world.tuning.avalanche_rocks,
                        next: now,
                    },
                );
            }
        }
        let Some(kind) = world.creature_kind(ROCK) else {
            self.running.clear();
            return;
        };
        let gap = u64::from(ms_to_ticks(world.tuning.avalanche_gap).max(1));
        let rest = u64::from(ms_to_ticks(world.tuning.avalanche_rest));
        let mut done = Vec::new();
        for (id, run) in &mut self.running {
            if now < run.next {
                continue;
            }
            // leicht versetzt, damit die Brocken nicht aufeinander liegen
            #[allow(clippy::cast_precision_loss)]
            let jitter = ((run.left * 37) % 5) as f32 * 6.0 - 12.0;
            let pos = run.from + Vec2::new(jitter, 0.0);
            if let Some(cid) = world.add_creature(kind, pos)
                && let Some(c) = world.creatures.iter_mut().find(|c| c.id == cid)
            {
                c.facing = run.dir;
                // der erste Brocken: die Lawine geht ab (Grollen)
                if run.left == world.tuning.avalanche_rocks {
                    world.events.push(Event::CreatureAct {
                        id: cid,
                        pos,
                        act: elora_sim::CreatureAct::Warn,
                    });
                }
            }
            run.left = run.left.saturating_sub(1);
            run.next = now + gap;
            if run.left == 0 {
                done.push(id.clone());
            }
        }
        for id in done {
            self.running.remove(&id);
            self.rest.insert(id, now + rest);
        }
    }
}
