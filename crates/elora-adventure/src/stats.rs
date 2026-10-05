//! Werte aus Fähigkeitenbaum, Ausrüstung und Waffen-Ausbau – zusammengezählt und als
//! Tuning der Simulation.

use elora_sim::Tuning;

use crate::data::Bonus;

/// Summe aller Boni.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Stats {
    pub max_health: i32,
    pub armor: i32,
    pub damage_pct: f32,
    pub fire_delay_pct: f32,
    pub knockback_pct: f32,
    pub ammo: i32,
    pub magnet: f32,
    pub drops_pct: f32,
    pub invulnerable_ms: i32,
    pub heal_bonus: i32,
    pub second_chance: i32,
    pub ruck_cooldown_ms: i32,
    pub hook_length_pct: f32,
    pub stomp_radius: f32,
    pub stomp_damage: i32,
    pub grip_ms: i32,
    pub glide_fall: f32,
    pub hammer_stun_ms: i32,
    pub hammer_damage: i32,
    pub hammer_reach_pct: f32,
    pub hammer_shockwave: bool,
    pub explosion_pct: f32,
    pub grenade_shards: i32,
    pub laser_bounces: i32,
    pub laser_pierce: i32,
    pub laser_delay_pct: f32,
    pub heat_pct: f32,
}

impl Stats {
    pub fn add(&mut self, b: &Bonus) {
        match *b {
            Bonus::MaxHealth(v) => self.max_health += v,
            Bonus::Armor(v) => self.armor += v,
            Bonus::DamagePct(v) => self.damage_pct += v,
            Bonus::FireDelayPct(v) => self.fire_delay_pct += v,
            Bonus::KnockbackPct(v) => self.knockback_pct += v,
            Bonus::Ammo(v) => self.ammo += v,
            Bonus::Magnet(v) => self.magnet += v,
            Bonus::DropsPct(v) => self.drops_pct += v,
            Bonus::InvulnerableMs(v) => self.invulnerable_ms += v,
            Bonus::HealBonus(v) => self.heal_bonus += v,
            Bonus::SecondChance(v) => self.second_chance = self.second_chance.max(v),
            Bonus::RuckCooldownMs(v) => self.ruck_cooldown_ms += v,
            Bonus::HookLengthPct(v) => self.hook_length_pct += v,
            Bonus::StompRadius(v) => self.stomp_radius += v,
            Bonus::StompDamage(v) => self.stomp_damage += v,
            Bonus::GripMs(v) => self.grip_ms += v,
            Bonus::GlideFall(v) => self.glide_fall += v,
            Bonus::HammerStunMs(v) => self.hammer_stun_ms += v,
            Bonus::HammerDamage(v) => self.hammer_damage += v,
            Bonus::HammerReachPct(v) => self.hammer_reach_pct += v,
            Bonus::HammerShockwave(v) => self.hammer_shockwave |= v > 0,
            Bonus::ExplosionPct(v) => self.explosion_pct += v,
            Bonus::GrenadeShards(v) => self.grenade_shards += v,
            Bonus::LaserBounces(v) => self.laser_bounces += v,
            Bonus::LaserPierce(v) => self.laser_pierce += v,
            Bonus::LaserDelayPct(v) => self.laser_delay_pct += v,
            Bonus::HeatPct(v) => self.heat_pct += v,
        }
    }

    /// Tuning des Abenteuers aus dem Grund-Tuning und diesen Werten. Lauftempo, Sprung und
    /// Hook-Zug bleiben unberührt (E-212). `max_health` setzt der Aufrufer.
    pub fn apply(&self, base: &Tuning) -> Tuning {
        let mut t = base.clone();
        let pct = |v: f32| 1.0 + v / 100.0;
        #[allow(clippy::cast_possible_truncation)]
        let scale_dmg = |d: i32| ((d as f32 * pct(self.damage_pct)).round() as i32).max(1);
        t.hammer_damage = scale_dmg(base.hammer_damage + self.hammer_damage);
        t.grenade_damage = scale_dmg(base.grenade_damage);
        t.laser_damage = scale_dmg(base.laser_damage);
        let delay = |ms: u32, extra: f32| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let v = (ms as f32 * pct(self.fire_delay_pct) * pct(extra))
                .round()
                .max(20.0) as u32;
            v
        };
        t.hammer_fire_delay = delay(base.hammer_fire_delay, 0.0);
        t.grenade_fire_delay = delay(base.grenade_fire_delay, 0.0);
        t.laser_fire_delay = delay(base.laser_fire_delay, self.laser_delay_pct);
        t.creature_knockback = base.creature_knockback * pct(self.knockback_pct);
        t.max_ammo = base.max_ammo + self.ammo;
        t.loot_magnet = base.loot_magnet + self.magnet;
        t.hit_invulnerable = add_ms(base.hit_invulnerable, self.invulnerable_ms, 0);
        t.ruck_cooldown = add_ms(base.ruck_cooldown, self.ruck_cooldown_ms, 100);
        t.hook_length = base.hook_length * pct(self.hook_length_pct);
        t.stomp_radius = base.stomp_radius + self.stomp_radius;
        t.stomp_damage = base.stomp_damage + self.stomp_damage;
        t.grip_time = add_ms(base.grip_time, self.grip_ms, 0);
        t.glide_fall_speed = (base.glide_fall_speed + self.glide_fall).max(0.3);
        t.hammer_stun = add_ms(base.hammer_stun, self.hammer_stun_ms, 0);
        t.hammer_reach = base.hammer_reach * pct(self.hammer_reach_pct);
        t.hammer_shockwave = base.hammer_shockwave || self.hammer_shockwave;
        t.explosion_radius = base.explosion_radius * pct(self.explosion_pct);
        t.explosion_inner_radius = base.explosion_inner_radius * pct(self.explosion_pct);
        t.grenade_shards = base.grenade_shards + self.grenade_shards.max(0).unsigned_abs();
        t.laser_bounce_num = base.laser_bounce_num + self.laser_bounces.max(0).unsigned_abs();
        t.laser_pierce = base.laser_pierce + self.laser_pierce.max(0).unsigned_abs();
        t
    }
}

fn add_ms(base: u32, delta: i32, min: u32) -> u32 {
    base.saturating_add_signed(delta).max(min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_stats_keep_tuning() {
        let base = Tuning::default();
        assert_eq!(Stats::default().apply(&base), base);
    }

    #[test]
    #[allow(clippy::float_cmp)] // Werte werden unverändert übernommen
    fn bonuses_change_only_combat_values() {
        let base = Tuning::default();
        let mut s = Stats::default();
        for b in [
            Bonus::DamagePct(10.0),
            Bonus::HammerDamage(1),
            Bonus::FireDelayPct(-8.0),
            Bonus::RuckCooldownMs(-2000),
            Bonus::GlideFall(-5.0),
        ] {
            s.add(&b);
        }
        let t = s.apply(&base);
        assert_eq!(t.hammer_damage, 4, "(3 + 1) × 1,1 gerundet");
        assert_eq!(t.grenade_damage, 7);
        assert!(t.hammer_fire_delay < base.hammer_fire_delay);
        assert_eq!(t.ruck_cooldown, 100, "nie unter 100 ms");
        assert!((t.glide_fall_speed - 0.3).abs() < 1e-6);
        // Bewegungsgefühl bleibt (E-212)
        assert_eq!(t.ground_control_speed, base.ground_control_speed);
        assert_eq!(t.ground_jump_impulse, base.ground_jump_impulse);
        assert_eq!(t.hook_drag_accel, base.hook_drag_accel);
    }
}
