//! Enemy projectiles, contact damage and loot.

use super::{LOOT_DELAY, LOOT_MAGNET_SPEED, LOOT_SIZE, SHOT_LIFETIME, SHOT_RADIUS, grounded, sign};
use crate::character::PHYS_SIZE;
use crate::creature::Creature;
use crate::event::{DeathCause, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;
use crate::world::World;

impl World {
    pub(super) fn tick_shots(&mut self, chars: &[(usize, Vec2)]) {
        let kb = self.tuning.hit_knockback;
        let mut i = 0;
        while i < self.creature_shots.len() {
            let s = &mut self.creature_shots[i];
            s.ticks += 1;
            if s.landed {
                s.glow = s.glow.saturating_sub(1);
            } else {
                s.vel.y += s.gravity;
                s.pos += s.vel;
            }
            let (pos, vel, damage) = (s.pos, s.vel, s.damage);
            let hit = chars
                .iter()
                .find(|&&(_, p)| p.distance(pos) < PHYS_SIZE / 2.0 + SHOT_RADIUS)
                .map(|&(j, _)| j);
            if let Some(j) = hit {
                self.creature_shots.remove(i);
                let force = vel.normalize() * (kb * 0.5);
                self.take_damage(j, force, damage, None, DeathCause::Creature);
            } else if self.creature_shots[i].landed {
                // spark burns out on the ground
                if self.creature_shots[i].glow == 0 {
                    self.creature_shots.remove(i);
                } else {
                    i += 1;
                }
            } else if self.collision.is_solid(pos) && self.creature_shots[i].glow > 0 {
                let s = &mut self.creature_shots[i];
                s.pos -= s.vel;
                s.vel = Vec2::ZERO;
                s.landed = true;
                i += 1;
            } else if self.collision.is_solid(pos) || self.creature_shots[i].ticks > SHOT_LIFETIME {
                self.creature_shots.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Contact damages Elora (E-232) with knockback (A-12); the protection afterwards lives in
    /// `take_damage` (E-234).
    pub(super) fn tick_contact(&mut self, chars: &[(usize, Vec2)]) {
        let kb = self.tuning.hit_knockback;
        let mut hits = Vec::new();
        let mut dazes = Vec::new();
        let mut freezes = Vec::new();
        let mut fled = Vec::new();
        let tick = self.tick;
        let open: Vec<usize> = chars
            .iter()
            .filter(|&&(j, _)| {
                self.players[j]
                    .as_ref()
                    .and_then(|p| p.character.as_ref())
                    .is_some_and(|ch| ch.invulnerable_until <= tick)
            })
            .map(|&(j, _)| j)
            .collect();
        for c in &self.creatures {
            let k = &self.creature_kinds[c.kind];
            // dazed guardians, hidden serpents and companions do no damage
            if (k.touch_damage <= 0 && k.daze_ms == 0 && k.freeze_ms == 0) || !c.harmful(k) {
                continue;
            }
            for &(j, p) in chars {
                let d = p - c.pos;
                if d.x.abs() < f32::midpoint(PHYS_SIZE, k.size[0])
                    && d.y.abs() < f32::midpoint(PHYS_SIZE, k.size[1])
                {
                    let force = Vec2::new(f32::from(sign(d.x)) * kb, -kb * 0.6);
                    hits.push((j, force, k.touch_damage));
                    if k.daze_ms > 0 {
                        dazes.push((j, ms_to_ticks(k.daze_ms)));
                    }
                    if k.freeze_ms > 0 && open.contains(&j) {
                        freezes.push((j, ms_to_ticks(k.freeze_ms)));
                        fled.push(c.id);
                    }
                }
            }
        }
        for (j, force, damage) in hits {
            if damage > 0 {
                self.take_damage(j, force, damage, None, DeathCause::Creature);
            }
        }
        // icicles and snow chunks shatter on Elora
        let shattered: Vec<u32> = self
            .creatures
            .iter()
            .filter(|c| {
                let k = &self.creature_kinds[c.kind];
                Creature::shatters(k)
                    && c.harmful(k)
                    && chars.iter().any(|&(_, p)| {
                        let d = p - c.pos;
                        d.x.abs() < f32::midpoint(PHYS_SIZE, k.size[0])
                            && d.y.abs() < f32::midpoint(PHYS_SIZE, k.size[1])
                    })
            })
            .map(|c| c.id)
            .collect();
        for id in shattered {
            if let Some(i) = self.creature_index(id) {
                self.kill_creature(i, None, false);
            }
        }
        // Frost ghost: Elora freezes, it backs off (D-M24-07)
        for (j, ticks) in freezes {
            if let Some(ch) = self.character_mut(j)
                && ch.core.frozen == 0
            {
                ch.core.frozen = ticks;
            }
        }
        for id in fled {
            if let Some(c) = self.creatures.iter_mut().find(|c| c.id == id) {
                c.mode = crate::creature::ghost::FLEE;
                c.timer = 0;
            }
        }
        // colourful rush (E-311): do not refresh while it is still active
        for (j, ticks) in dazes {
            if let Some(ch) = self.character_mut(j)
                && ch.core.dazed == 0
            {
                ch.core.dazed = ticks;
            }
        }
    }

    pub(super) fn tick_loot(&mut self, chars: &[(usize, Vec2)]) {
        let (gravity, magnet) = (self.tuning.gravity, self.tuning.loot_magnet);
        let size = Vec2::new(LOOT_SIZE, LOOT_SIZE);
        let mut collected = Vec::new();
        for l in &mut self.loot {
            l.age = l.age.saturating_add(1);
            let near = chars
                .iter()
                .map(|&(j, p)| (j, p, p.distance(l.pos)))
                .filter(|x| x.2 < magnet)
                .min_by(|a, b| a.2.total_cmp(&b.2));
            if l.age >= LOOT_DELAY
                && let Some((j, p, d)) = near
            {
                l.vel = Vec2::ZERO;
                if d < PHYS_SIZE / 2.0 + 4.0 {
                    collected.push((l.id, j));
                } else {
                    l.pos += (p - l.pos).normalize() * LOOT_MAGNET_SPEED.min(d);
                }
                continue;
            }
            l.vel.y += gravity;
            let (mut pos, mut vel) = (l.pos, l.vel);
            self.collision
                .move_box_platforms(&mut pos, &mut vel, size, 0.5, true);
            if grounded(&self.collision, pos, size) {
                vel.x *= 0.9;
            }
            l.pos = pos;
            l.vel = vel;
        }
        for (id, player) in collected {
            if let Some(i) = self.loot.iter().position(|l| l.id == id) {
                let l = self.loot.remove(i);
                self.events.push(Event::LootCollect {
                    player,
                    item: l.item,
                    count: l.count,
                    pos: l.pos,
                });
            }
        }
    }
}
