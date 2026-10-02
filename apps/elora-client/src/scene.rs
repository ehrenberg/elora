//! Was gezeichnet wird – unabhängig davon, ob die Daten aus der lokalen Sandbox
//! oder aus Snapshots und Vorhersage (online) stammen.

use elora_sim::{Character, CharacterCore, PickupKind, TICKS_PER_SECOND, Team, Vec2, World};

/// Eine Figur mit Zustand vor und nach dem aktuellen Tick.
#[derive(Debug, Clone)]
pub struct SceneChar {
    pub slot: usize,
    /// Aktueller Zustand (Leben, Waffen, Kern).
    pub ch: Character,
    /// Kern vor dem aktuellen Tick.
    pub prev: CharacterCore,
    /// Anteil zwischen `prev` und `ch.core` (0..1).
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

/// Ein sichtbarer Laserabschnitt.
#[derive(Debug, Clone, Copy)]
pub struct SceneLaser {
    pub from: Vec2,
    pub to: Vec2,
    /// 1 = frisch, 0 = verblasst.
    pub fade: f32,
}

/// Eine Flagge (CTF).
#[derive(Debug, Clone, Copy)]
pub struct SceneFlag {
    pub team: Team,
    pub pos: Vec2,
    pub at_stand: bool,
    pub stand: Vec2,
}

/// Aussehen eines Abenteuer-Objekts (A1.6).
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
    /// `active`: hier wurde zuletzt gerastet.
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

/// Ein Gegner (A1.2).
#[derive(Debug, Clone)]
pub struct SceneCreature {
    pub id: u32,
    /// Name der Art (Grafik).
    pub kind: String,
    pub pos: Vec2,
    pub facing: i8,
    pub health: i32,
    pub max_health: i32,
    /// Ticks seit dem letzten Treffer (Lebensbalken, E-238).
    pub since_hit: Option<u64>,
    pub stunned: bool,
    pub airborne: bool,
    pub boss: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Scene {
    /// Simulations-Tick (Blinken, Anzeigen).
    pub tick: u64,
    pub creatures: Vec<SceneCreature>,
    pub creature_shots: Vec<Vec2>,
    /// Beute: Gegenstand und Position.
    pub loot: Vec<(String, Vec2)>,
    /// Abenteuer-Objekte (NPCs, Truhen, Schalter …).
    pub objects: Vec<SceneObject>,
    pub flags: Vec<SceneFlag>,
    pub chars: Vec<SceneChar>,
    pub projectiles: Vec<Vec2>,
    pub lasers: Vec<SceneLaser>,
    pub pickups: Vec<(PickupKind, Vec2)>,
    pub spawns: Vec<Vec2>,
    /// Kameramitte.
    pub camera: Vec2,
}

impl Scene {
    pub fn local(&self) -> Option<&SceneChar> {
        self.chars.iter().find(|c| c.local)
    }

    /// Projektile und Laser einer Welt zum Zeitpunkt `tick + alpha` hinzufügen.
    /// `filter` wählt nach Schütze aus.
    pub fn add_shots(&mut self, world: &World, alpha: f32, filter: impl Fn(usize) -> bool) {
        let tps = TICKS_PER_SECOND as f32;
        for pr in world.projectiles.iter().filter(|p| filter(p.owner)) {
            let age = (world.tick - pr.start_tick) as f32;
            self.projectiles
                .push(pr.pos_at((age - 1.0 + alpha).max(0.0) / tps, &world.tuning));
        }
        let fade_ticks = (TICKS_PER_SECOND * world.tuning.laser_bounce_delay) as f32 / 1000.0 + 1.0;
        for l in world.lasers.iter().filter(|l| filter(l.owner)) {
            let age = (world.tick - l.eval_tick) as f32 + alpha;
            let fade = (1.0 - age / fade_ticks).clamp(0.2, 1.0);
            self.lasers.push(SceneLaser {
                from: l.from,
                to: l.pos,
                fade,
            });
        }
    }

    /// Flaggen; getragene Flaggen hängen am (gezeichneten) Träger.
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

    /// Gegner, ihre Geschosse und Beute; `prev` sind die Positionen vor dem Tick (Id → Position).
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
            });
        }
        self.creature_shots = world
            .creature_shots
            .iter()
            .map(|s| s.pos + s.vel * (alpha - 1.0))
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
