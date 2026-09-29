//! Was gezeichnet wird – unabhängig davon, ob die Daten aus der lokalen Sandbox
//! oder aus Snapshots und Vorhersage (online) stammen.

use elora_sim::{Character, CharacterCore, PickupKind, TICKS_PER_SECOND, Vec2, World};

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

#[derive(Debug, Clone, Default)]
pub struct Scene {
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
