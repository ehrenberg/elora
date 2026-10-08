//! What is drawn – regardless of whether the data comes from the local sandbox
//! or from snapshots and prediction (online).

use elora_sim::{Character, CharacterCore, PickupKind, TICKS_PER_SECOND, Team, Vec2, World};

/// A character with state before and after the current tick.
#[derive(Debug, Clone)]
pub struct SceneChar {
    pub slot: usize,
    /// Current state (health, weapons, core).
    pub ch: Character,
    /// Core before the current tick.
    pub prev: CharacterCore,
    /// Fraction between `prev` and `ch.core` (0..1).
    pub alpha: f32,
    pub dummy: bool,
    pub local: bool,
    pub team: Team,
}

impl SceneChar {
    pub fn pos(&self) -> Vec2 {
        self.prev.pos.lerp(self.ch.core.pos, self.alpha)
    }

    pub fn hook_pos(&self) -> Vec2 {
        self.prev.hook_pos.lerp(self.ch.core.hook_pos, self.alpha)
    }
}

/// A visible laser segment.
#[derive(Debug, Clone, Copy)]
pub struct SceneLaser {
    pub from: Vec2,
    pub to: Vec2,
    /// 1 = fresh, 0 = faded.
    pub fade: f32,
}

/// A flag (CTF).
#[derive(Debug, Clone, Copy)]
pub struct SceneFlag {
    pub team: Team,
    pub pos: Vec2,
    pub at_stand: bool,
    pub stand: Vec2,
}

/// Appearance of an adventure object (A1.6).
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectLook {
    Npc {
        character: String,
        facing: i8,
    },
    Chest {
        open: bool,
    },
    Switch {
        on: bool,
    },
    /// `active`: the last rest happened here.
    SavePoint {
        active: bool,
    },
    HealPlant {
        used: bool,
    },
    Collectible {
        item: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneObject {
    pub look: ObjectLook,
    pub pos: Vec2,
}

/// An enemy (A1.2).
#[derive(Debug, Clone)]
pub struct SceneCreature {
    pub id: u32,
    /// Name of the kind (graphics).
    pub kind: String,
    pub pos: Vec2,
    pub facing: i8,
    pub health: i32,
    pub max_health: i32,
    /// Ticks since the last hit (health bar, E-238).
    pub since_hit: Option<u64>,
    pub stunned: bool,
    pub airborne: bool,
    pub boss: bool,
    /// State of multi-stage behaviours (guardian: dive, stunned; `elora_sim::creature::diver`).
    pub mode: u8,
    /// How far a root snake has grown out of the ground (0..1, otherwise 1).
    pub grow: f32,
    /// Velocity (flight poses rotate along, gait bobs).
    pub vel: Vec2,
    /// Collision box (width, height; shadow).
    pub size: Vec2,
    /// Target of the behaviour (Kristella: frost wave x = front, y = floor of the hall).
    pub goal: Vec2,
    /// Hall of the guardian of the Frostspitzen: left and right edge, length of the fresh frost.
    pub hall: Option<(f32, f32, f32)>,
}

#[derive(Debug, Clone, Default)]
pub struct Scene {
    /// Simulation tick (blinking, displays).
    pub tick: u64,
    pub creatures: Vec<SceneCreature>,
    /// Enemy projectiles: position and whether spark (`Some(true)` = glows on the ground).
    pub creature_shots: Vec<(Vec2, Option<bool>)>,
    /// Loot: item and position.
    pub loot: Vec<(String, Vec2)>,
    /// Adventure objects (NPCs, chests, switches …).
    pub objects: Vec<SceneObject>,
    pub flags: Vec<SceneFlag>,
    pub chars: Vec<SceneChar>,
    pub projectiles: Vec<Vec2>,
    pub lasers: Vec<SceneLaser>,
    pub pickups: Vec<(PickupKind, Vec2)>,
    pub spawns: Vec<Vec2>,
    /// Camera centre.
    pub camera: Vec2,
}

impl Scene {
    pub fn local(&self) -> Option<&SceneChar> {
        self.chars.iter().find(|c| c.local)
    }

    /// Add projectiles and lasers of a world at time `tick − span·(1 − alpha)`:
    /// `span` ticks between the previous and this state (online every 2nd tick: 2).
    /// `filter` selects by shooter.
    pub fn add_shots(
        &mut self,
        world: &World,
        alpha: f32,
        span: f32,
        filter: impl Fn(usize) -> bool,
    ) {
        let tps = TICKS_PER_SECOND as f32;
        let back = span * (1.0 - alpha);
        for pr in world.projectiles.iter().filter(|p| filter(p.owner)) {
            let age = (world.tick - pr.start_tick) as f32;
            self.projectiles
                .push(pr.pos_at((age - back).max(0.0) / tps, &world.tuning));
        }
        let fade_ticks = (TICKS_PER_SECOND * world.tuning.laser_bounce_delay) as f32 / 1000.0 + 1.0;
        for l in world.lasers.iter().filter(|l| filter(l.owner)) {
            let age = (world.tick - l.eval_tick) as f32 + 1.0 - back;
            let fade = (1.0 - age / fade_ticks).clamp(0.2, 1.0);
            self.lasers.push(SceneLaser {
                from: l.from,
                to: l.pos,
                fade,
            });
        }
    }

    /// Flags; carried flags hang on the (drawn) carrier.
    pub fn add_flags(&mut self, world: &World) {
        for f in &world.flags {
            let pos = f
                .carrier
                .and_then(|c| self.chars.iter().find(|ch| ch.slot == c))
                .map_or(f.pos, |c| c.pos() + Vec2::new(-10.0, -24.0));
            self.flags.push(SceneFlag {
                team: f.team,
                pos,
                at_stand: f.at_stand,
                stand: f.stand,
            });
        }
    }

    /// Enemies, their projectiles and loot; `prev` are the positions before the tick (id → pos).
    pub fn add_creatures(
        &mut self,
        world: &World,
        prev: &std::collections::HashMap<u32, Vec2>,
        alpha: f32,
    ) {
        self.tick = world.tick;
        for c in &world.creatures {
            let kind = &world.creature_kinds[c.kind];
            let from = prev.get(&c.id).copied().unwrap_or(c.pos);
            self.creatures.push(SceneCreature {
                id: c.id,
                kind: kind.name.clone(),
                pos: from.lerp(c.pos, alpha),
                facing: c.facing,
                health: c.health,
                max_health: kind.health,
                since_hit: c.hit_tick.map(|t| world.tick - t),
                stunned: c.stun > 0,
                airborne: !c.grounded,
                boss: kind.boss,
                mode: c.mode,
                grow: grow(kind, c),
                vel: c.vel,
                size: kind.size(),
                goal: c.goal,
                hall: match &kind.behavior {
                    elora_sim::Behavior::Queen(d) => Some((
                        c.home.x - d.width / 2.0,
                        c.home.x + d.width / 2.0,
                        d.fresh_len,
                    )),
                    _ => None,
                },
            });
        }
        self.creature_shots = world
            .creature_shots
            .iter()
            .map(|s| {
                let spark = (s.glow > 0 || s.landed).then_some(s.landed);
                (s.pos + s.vel * (alpha - 1.0), spark)
            })
            .collect();
        self.loot = world
            .loot
            .iter()
            .map(|l| {
                (
                    l.item.clone(),
                    prev.get(&l.id).copied().unwrap_or(l.pos).lerp(l.pos, alpha),
                )
            })
            .collect();
    }

    pub fn add_pickups(&mut self, world: &World) {
        self.pickups = world
            .pickups
            .iter()
            .filter(|p| p.available())
            .map(|p| (p.kind, p.pos))
            .collect();
        self.spawns.clone_from(&world.spawn_points);
    }
}

/// Growth of an emerging root snake (R2-M2.2): 0 = in the ground, 1 = fully out.
fn grow(kind: &elora_sim::CreatureKind, c: &elora_sim::Creature) -> f32 {
    use elora_sim::creature::{Behavior, burrow};
    match kind.behavior {
        Behavior::Burrower { rise_ms, .. } if c.mode == burrow::RISING => {
            #[allow(clippy::cast_precision_loss)]
            let total = elora_sim::tuning::ms_to_ticks(rise_ms).max(1) as f32;
            #[allow(clippy::cast_precision_loss)]
            let p = c.timer as f32 / total;
            p.clamp(0.0, 1.0)
        }
        _ => 1.0,
    }
}
