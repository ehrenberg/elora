//! Creatures in the world (A1.2): behaviour, movement, hits, contact, projectiles, loot.
//!
//! This module holds the lifecycle (spawn, damage, defeat, loot drop) and the tick order;
//! [`behavior`] dispatches each creature to its behaviour module, [`effects`] applies what the
//! guardians leave in the world and [`contact`] handles projectiles, touches and loot.

mod air;
mod behavior;
mod contact;
mod diver;
mod effects;
mod ground;
mod leaper;
mod queen;
mod serpent;
mod turret;
mod warden;

use crate::character::PHYS_SIZE;
use crate::collision::{Collision, Tile};
use crate::creature::{Behavior, Creature, HookTarget, Loot, rng, rng_f32};
use crate::event::Event;
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;
use crate::world::World;

/// Edge length of the loot box.
const LOOT_SIZE: f32 = 12.0;
/// Loot can only be collected after this time (ticks), so it visibly pops out.
const LOOT_DELAY: u32 = 20;
/// Flight speed of loot towards Elora (units/tick).
const LOOT_MAGNET_SPEED: f32 = 12.0;
/// At most this many individual pieces per entry (the rest is merged).
const LOOT_PIECES: u32 = 8;
/// Radius of an enemy projectile.
const SHOT_RADIUS: f32 = 8.0;
/// How long one half of the hook flowers stays wilted (ticks, 2.5 s).
const WILT_TICKS: u64 = 125;
/// Lifetime of an enemy projectile (ticks).
const SHOT_LIFETIME: u32 = 150;

/// Duration `ms` in ticks, shortened by `pace` (faster when angry); at least one tick.
fn paced_ticks(ms: u32, pace: f32) -> u32 {
    crate::math::to_u32(ms_to_ticks(ms) as f32 / pace).max(1)
}

fn sign(v: f32) -> i8 {
    if v < 0.0 { -1 } else { 1 }
}

/// Is there something solid (or a platform) below the point?
fn floor_at(col: &Collision, p: Vec2) -> bool {
    let t = col.tile_at(p);
    t.is_solid() || t == Tile::Platform
}

/// Arcing throw from `from` to `to`: horizontal with `speed`, vertical such that the
/// projectile arrives there (flat throws go at least a little high).
fn lob_velocity(from: Vec2, to: Vec2, speed: f32, gravity: f32) -> Vec2 {
    let d = to - from;
    let t = (d.x.abs() / speed).max(12.0);
    let vy = (d.y - 0.5 * gravity * t * t) / t;
    Vec2::new(d.x / t, vy.min(-3.0))
}

/// Is a figure below `pos` (horizontally up to `sight`, vertically up to `reach`, clear view)?
fn below(col: &Collision, chars: &[(usize, Vec2)], pos: Vec2, sight: f32, reach: f32) -> bool {
    chars.iter().any(|&(_, p)| {
        (p.x - pos.x).abs() <= sight
            && p.y > pos.y
            && p.y - pos.y <= reach
            && col.intersect_line(pos, p).is_none()
    })
}

fn grounded(col: &Collision, pos: Vec2, size: Vec2) -> bool {
    let y = pos.y + size.y / 2.0 + 2.0;
    floor_at(col, Vec2::new(pos.x - size.x / 2.0 + 1.0, y))
        || floor_at(col, Vec2::new(pos.x + size.x / 2.0 - 1.0, y))
}

impl World {
    /// Hook flowers wilt alternately while a guardian from the air is angry (E-298).
    pub(crate) fn update_hook_wilt(&mut self) {
        let angry = self.creatures.iter().any(|c| {
            let k = &self.creature_kinds[c.kind];
            match &k.behavior {
                Behavior::Diver(d) => {
                    let life = c.health as f32 / k.health.max(1) as f32;
                    c.mode != crate::creature::diver::SLEEP
                        && d.enrage_at > 0.0
                        && life <= d.enrage_at
                }
                _ => false,
            }
        });
        self.collision.hook_wilt = angry.then(|| (self.tick / WILT_TICKS).is_multiple_of(2));
    }

    /// Index of an enemy kind by name.
    pub fn creature_kind(&self, name: &str) -> Option<usize> {
        self.creature_kinds.iter().position(|k| k.name == name)
    }

    /// Spawns an enemy of kind `kind` at `pos`; returns its id.
    pub fn add_creature(&mut self, kind: usize, pos: Vec2) -> Option<u32> {
        let k = self.creature_kinds.get(kind)?;
        let id = self.next_id;
        self.next_id += 1;
        self.creatures.push(Creature {
            id,
            kind,
            pos,
            vel: Vec2::ZERO,
            home: pos,
            health: k.health,
            facing: -1,
            timer: 0,
            stun: 0,
            hit_tick: None,
            grounded: false,
            mode: 0,
            goal: pos,
            count: match &k.behavior {
                Behavior::Warden(d) => d.cores,
                _ => 0,
            },
            tug: 0,
            wall_timer: 0,
            hits: 0,
        });
        Some(id)
    }

    pub(crate) fn hook_targets(&self) -> Vec<HookTarget> {
        self.creatures
            .iter()
            .filter(|c| c.hookable(&self.creature_kinds[c.kind]))
            .map(|c| {
                let k = &self.creature_kinds[c.kind];
                HookTarget {
                    id: c.id,
                    pos: c.pos,
                    radius: k.radius(),
                    small: k.small,
                    anchor: matches!(k.behavior, Behavior::Warden(_)),
                }
            })
            .collect()
    }

    fn creature_index(&self, id: u32) -> Option<usize> {
        self.creatures.iter().position(|c| c.id == id)
    }

    /// First enemy on the segment `from`–`to`; returns index and intersection point.
    pub(crate) fn intersect_creature(
        &self,
        from: Vec2,
        to: Vec2,
        radius: f32,
    ) -> Option<(usize, Vec2)> {
        let mut best: Option<(usize, Vec2, f32)> = None;
        for (i, c) in self.creatures.iter().enumerate() {
            let r = self.creature_kinds[c.kind].radius();
            let at = Vec2::closest_point_on_segment(from, to, c.pos);
            if c.pos.distance(at) < r + radius {
                let d = from.distance(at);
                if best.is_none_or(|b| d < b.2) {
                    best = Some((i, at, d));
                }
            }
        }
        best.map(|(i, at, _)| (i, at))
    }

    /// Enemies on the segment `from`–`to`, sorted by distance (id, intersection point).
    pub(crate) fn creatures_on_line(&self, from: Vec2, to: Vec2) -> Vec<(u32, Vec2)> {
        let mut hits: Vec<(u32, Vec2, f32)> = self
            .creatures
            .iter()
            .filter_map(|c| {
                let r = self.creature_kinds[c.kind].radius();
                let at = Vec2::closest_point_on_segment(from, to, c.pos);
                (c.pos.distance(at) < r).then(|| (c.id, at, from.distance(at)))
            })
            .collect();
        hits.sort_by(|a, b| a.2.total_cmp(&b.2));
        hits.into_iter().map(|(id, at, _)| (id, at)).collect()
    }

    /// Shards (A-20): small follow-up explosions around the impact, only against enemies.
    pub(crate) fn grenade_shards(&mut self, pos: Vec2, owner: usize, max_damage: i32) {
        let n = self.tuning.grenade_shards;
        for k in 0..n {
            let a = std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::TAU / n as f32;
            let p = pos + Vec2::new(a.cos(), a.sin()) * 48.0;
            self.events.push(Event::Explosion { owner, pos: p });
            self.explode_creatures(p, pos, owner, (max_damage / 3).max(1));
        }
    }

    /// Hammer blow on enemies; returns the number of hits. With shockwave (A-19)
    /// it hits all enemies around Elora.
    pub(crate) fn hammer_creatures(&mut self, owner: usize, pos: Vec2, start: Vec2) -> usize {
        let (reach, shockwave) = (self.tuning.hammer_reach, self.tuning.hammer_shockwave);
        let hits: Vec<(u32, Vec2)> = self
            .creatures
            .iter()
            .filter(|c| {
                let r = self.creature_kinds[c.kind].radius();
                let near = if shockwave {
                    c.pos.distance(pos) < reach + PHYS_SIZE * 2.0 + r
                } else {
                    c.pos.distance(start) < reach + r
                };
                near && self.collision.intersect_line(pos, c.pos).is_none()
            })
            .map(|c| (c.id, c.pos))
            .collect();
        let (knockback, damage) = (self.tuning.hammer_knockback, self.tuning.hammer_damage);
        for &(id, p) in &hits {
            self.events.push(Event::HammerHit { owner, pos: p });
            let dir = if p.distance(pos) > 0.0 {
                (p - pos).normalize()
            } else {
                Vec2::new(0.0, -1.0)
            };
            let force = (dir + Vec2::new(0.0, -1.1)).normalize() * knockback;
            if let Some(i) = self.creature_index(id) {
                let stun = ms_to_ticks(self.tuning.hammer_stun);
                self.creatures[i].stun = self.creatures[i].stun.max(stun);
                self.damage_creature(i, force, damage, Some(owner), Some(pos));
            }
        }
        hits.len()
    }

    /// Explosion hits enemies (like figures, without armour); `src` is the impact (shell).
    pub(crate) fn explode_creatures(
        &mut self,
        pos: Vec2,
        src: Vec2,
        owner: usize,
        max_damage: i32,
    ) {
        let t = &self.tuning;
        let (radius, inner, max_force) = (
            t.explosion_radius,
            t.explosion_inner_radius,
            t.explosion_max_force,
        );
        let hits: Vec<(u32, Vec2, i32)> = self
            .creatures
            .iter()
            .filter_map(|c| {
                let r = self.creature_kinds[c.kind].radius();
                let diff = c.pos - pos;
                let l = diff.length();
                if l >= radius + r {
                    return None;
                }
                let force = if l > 0.0 {
                    diff.normalize() * max_force
                } else {
                    Vec2::new(0.0, -max_force)
                };
                let factor = 1.0 - ((l - inner) / (radius - inner)).clamp(0.0, 1.0);
                let damage = (factor * max_damage as f32) as i32;
                (damage != 0).then_some((c.id, force * factor, damage))
            })
            .collect();
        for (id, force, damage) in hits {
            if let Some(i) = self.creature_index(id) {
                self.damage_creature(i, force, damage, Some(owner), Some(src));
            }
        }
    }

    /// Shockwave of the stomp: damage and stun (A-13, A-14).
    pub(crate) fn stomp_creatures(&mut self, player: usize, pos: Vec2) {
        let r = self.tuning.stomp_radius;
        let (damage, stun) = (
            self.tuning.stomp_damage,
            ms_to_ticks(self.tuning.stomp_stun),
        );
        let hits: Vec<(u32, Vec2)> = self
            .creatures
            .iter()
            .filter(|c| c.pos.distance(pos) < r + self.creature_kinds[c.kind].radius())
            .map(|c| (c.id, c.pos))
            .collect();
        for (id, p) in hits {
            let force = Vec2::new(f32::from(sign(p.x - pos.x)) * 4.0, -6.0);
            if let Some(i) = self.creature_index(id) {
                self.creatures[i].stun = stun;
                // the sand serpent takes double damage from stomps (E-316)
                let serpent = matches!(
                    self.creature_kinds[self.creatures[i].kind].behavior,
                    Behavior::Serpent(_)
                );
                let damage = if serpent { damage * 2 } else { damage };
                self.damage_creature(i, force, damage, Some(player), None);
            }
        }
    }

    /// Damage to enemy `id` without knockback (tests, tools); bounces off guardians in the air.
    pub fn hurt_creature(&mut self, id: u32, damage: i32) {
        if let Some(i) = self.creature_index(id) {
            self.damage_creature(i, Vec2::ZERO, damage, None, None);
        }
    }

    /// Damage to enemy `i` from direction `src`; defeats it at 0 health.
    pub(crate) fn damage_creature(
        &mut self,
        i: usize,
        force: Vec2,
        damage: i32,
        from: Option<usize>,
        src: Option<Vec2>,
    ) {
        if self.prediction {
            return;
        }
        let tick = self.tick;
        let c = &mut self.creatures[i];
        let kind = &self.creature_kinds[c.kind];
        // Guardian in the air and shell from the side: hits bounce off (E-299, E-317)
        if !c.vulnerable(kind) || c.armor_blocks(kind, src) {
            self.events.push(Event::CreatureHit {
                id: c.id,
                pos: c.pos,
                damage: 0,
                from,
            });
            return;
        }
        if !matches!(
            kind.behavior,
            Behavior::Turret { .. } | Behavior::Diver(_) | Behavior::Serpent(_)
        ) {
            c.vel += force;
        }
        c.health -= damage;
        c.hits = c.hits.saturating_add(1);
        c.hit_tick = Some(tick);
        self.events.push(Event::CreatureHit {
            id: c.id,
            pos: c.pos,
            damage,
            from,
        });
        if c.health <= 0 {
            self.kill_creature(i, from, true);
        }
    }

    fn kill_creature(&mut self, i: usize, killer: Option<usize>, drop_loot: bool) {
        let c = self.creatures.remove(i);
        self.events.push(Event::CreatureDeath {
            id: c.id,
            kind: c.kind,
            pos: c.pos,
            killer,
        });
        if drop_loot {
            self.drop_loot(&c);
        }
    }

    fn drop_loot(&mut self, c: &Creature) {
        let entries = self.creature_kinds[c.kind].loot.clone();
        let seed = u64::from(c.id) << 32 ^ self.tick;
        for (e, entry) in entries.iter().enumerate() {
            let s = rng(seed ^ (e as u64) << 16);
            if rng_f32(s) >= entry.chance {
                continue;
            }
            let span = entry.max.saturating_sub(entry.min) + 1;
            let count = entry.min + (rng(s ^ 1) % u64::from(span)) as u32;
            if count == 0 {
                continue;
            }
            let pieces = count.min(LOOT_PIECES);
            for p in 0..pieces {
                let share = count / pieces + u32::from(p < count % pieces);
                let r = rng(s ^ u64::from(p + 2) << 8);
                let id = self.next_id;
                self.next_id += 1;
                self.loot.push(Loot {
                    id,
                    item: entry.item.clone(),
                    count: share,
                    pos: c.pos,
                    vel: Vec2::new((rng_f32(r) - 0.5) * 8.0, -6.0 - rng_f32(r ^ 7) * 4.0),
                    age: 0,
                });
            }
        }
    }

    /// Living figures (slot, position).
    fn living(&self) -> Vec<(usize, Vec2)> {
        self.players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| Some((i, p.as_ref()?.character.as_ref()?.core.pos)))
            .collect()
    }

    pub(crate) fn tick_creatures(&mut self) {
        if self.creatures.is_empty() && self.creature_shots.is_empty() && self.loot.is_empty() {
            return;
        }
        let chars = self.living();
        self.tick_pull();
        self.tick_behavior(&chars);
        self.tick_shots(&chars);
        self.tick_contact(&chars);
        self.tick_loot(&chars);
    }

    /// Pulling in (E-233): small enemies fly to Elora.
    fn tick_pull(&mut self) {
        let accel = self.tuning.pull_accel;
        let pulls: Vec<(u32, Vec2)> = self
            .players
            .iter()
            .filter_map(|p| p.as_ref()?.character.as_ref())
            .filter(|ch| ch.core.pulling)
            .filter_map(|ch| Some((ch.core.hooked_creature?, ch.core.pos)))
            .collect();
        for (id, to) in pulls {
            if let Some(c) = self.creatures.iter_mut().find(|c| c.id == id)
                && c.pos.distance(to) > PHYS_SIZE * 1.5
            {
                c.vel += (to - c.pos).normalize() * accel;
            }
        }
    }

    /// Players currently pulling on the Root Warden: on the hook and running away from it.
    fn tuggers(&self) -> Vec<u32> {
        self.players
            .iter()
            .filter_map(|p| p.as_ref()?.character.as_ref())
            .filter_map(|ch| {
                let id = ch.core.hooked_creature?;
                let c = self.creatures.iter().find(|c| c.id == id)?;
                let away = ch.core.direction != 0
                    && sign(ch.core.pos.x - c.pos.x) == ch.core.direction.signum();
                away.then_some(id)
            })
            .collect()
    }
}
