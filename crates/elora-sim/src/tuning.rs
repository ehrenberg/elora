//! Adjustable simulation values (docs/handbook/tuning.md, E-023).
//!
//! All values apply per tick at [`crate::TICKS_PER_SECOND`].

/// Movement, hook and weapon tuning. The defaults are the accepted
/// starting values T-02 to T-30 as well as E-052.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Tuning {
    /// T-02
    pub ground_control_speed: f32,
    /// T-03
    pub ground_control_accel: f32,
    /// T-04
    pub ground_friction: f32,
    /// T-05
    pub ground_jump_impulse: f32,
    /// T-06
    pub air_jump_impulse: f32,
    /// T-07
    pub air_control_speed: f32,
    /// T-08
    pub air_control_accel: f32,
    /// T-09
    pub air_friction: f32,
    /// T-10
    pub gravity: f32,
    /// T-11
    pub velramp_start: f32,
    /// T-11
    pub velramp_range: f32,
    /// T-11
    pub velramp_curvature: f32,
    /// T-12
    pub hook_length: f32,
    /// T-13
    pub hook_fire_speed: f32,
    /// T-14
    pub hook_drag_accel: f32,
    /// T-15
    pub hook_drag_speed: f32,
    /// T-16: maximum duration one player can hold another (ticks)
    pub player_hook_ticks: u32,
    /// T-17: force factor on the hooked player
    pub player_hook_force: f32,
    /// Players collide with each other
    pub player_collision: bool,
    /// Players can hook each other
    pub player_hooking: bool,

    /// T-18: hammer damage
    pub hammer_damage: i32,
    /// T-18: hammer fire delay (ms)
    pub hammer_fire_delay: u32,
    /// T-19: strength of the hammer knockback (original: 10)
    pub hammer_knockback: f32,
    /// T-20: laser damage
    pub laser_damage: i32,
    /// T-20: laser fire delay (ms)
    pub laser_fire_delay: u32,
    /// T-21: laser range
    pub laser_reach: f32,
    /// T-22: delay until the bounce (ms)
    pub laser_bounce_delay: u32,
    /// T-22: number of bounces
    pub laser_bounce_num: u32,
    /// Range loss per bounce
    pub laser_bounce_cost: f32,
    /// E-052: laser knockback in shot direction (original: 0)
    pub laser_knockback: f32,
    /// T-23: maximum grenade damage (explosion center)
    pub grenade_damage: i32,
    /// T-23: grenade fire delay (ms)
    pub grenade_fire_delay: u32,
    /// T-24: grenade speed
    pub grenade_speed: f32,
    /// T-24: curvature of the trajectory
    pub grenade_curvature: f32,
    /// T-24: lifetime (s)
    pub grenade_lifetime: f32,
    /// T-25: explosion radius
    pub explosion_radius: f32,
    /// T-25: inner radius with full damage
    pub explosion_inner_radius: f32,
    /// T-25: maximum explosion force
    pub explosion_max_force: f32,
    /// T-27: maximum ammo
    pub max_ammo: i32,
    /// T-28: maximum health
    pub max_health: i32,
    /// T-28: maximum armor
    pub max_armor: i32,
    /// T-29: respawn time of the pickups (s)
    pub pickup_respawn: f32,
    /// T-30: minimum time until respawn after death (s)
    pub respawn_delay: f32,
    /// Automatic respawn without a click (s, original: 3)
    pub auto_respawn: f32,
    /// T-31: ground friction on ice (normal T-04)
    pub ice_friction: f32,
    /// T-32: acceleration on ice (normal T-03)
    pub ice_accel: f32,
    /// T-33: force of the jump pad (units/tick)
    pub jump_pad_force: f32,
    /// T-35: speed of the accelerator (units/tick)
    pub conveyor_speed: f32,

    // Abilities in the adventure and in the spring battle (R2-M1, A-01 ff.)
    /// A-01: speed of the hook jerk towards the hook point (units/tick)
    #[cfg_attr(feature = "serde", serde(rename = "ruck_speed"))]
    pub jerk_speed: f32,
    /// A-02: cooldown of the hook jerk (ms)
    #[cfg_attr(feature = "serde", serde(rename = "ruck_cooldown"))]
    pub jerk_cooldown: u32,
    /// A-28: for this long the jerk pulls straight to the hook point at `jerk_speed` (ms)
    #[cfg_attr(feature = "serde", serde(rename = "ruck_time"))]
    pub jerk_time: u32,
    /// A-04: fall speed while stomping (units/tick)
    pub stomp_speed: f32,
    /// A-05: radius of the shockwave on impact (units)
    pub stomp_radius: f32,
    /// A-06: cling duration of the ice grip (ms)
    pub grip_time: u32,
    /// A-07: slide speed while clinging (units/tick)
    pub grip_slide_speed: f32,
    /// A-08: wall jump sideways (units/tick)
    pub wall_jump_x: f32,
    /// A-08: wall jump upwards (units/tick)
    pub wall_jump_y: f32,
    /// A-09: maximum fall speed while gliding (units/tick)
    pub glide_fall_speed: f32,
    /// A-10: air control while gliding (normal T-07)
    pub glide_control_speed: f32,

    // Creatures in the adventure (A1.2)
    /// A-03: pull of the pull hook on small enemies (units/tick²)
    pub pull_accel: f32,
    /// A-11: invulnerable after a hit in the adventure (ms, E-234)
    pub hit_invulnerable: u32,
    /// A-12: knockback on touching an enemy (units/tick)
    pub hit_knockback: f32,
    /// A-13: damage of the stomp shockwave to enemies
    pub stomp_damage: i32,
    /// A-14: stun from the shockwave (ms)
    pub stomp_stun: u32,
    /// A-15: range from which loot flies to Elora (units, E-236)
    pub loot_magnet: f32,

    // Upgrades in the adventure (A1.3); default values = multiplayer
    /// A-16: radius of the hammer hit (original: half the body size)
    pub hammer_reach: f32,
    /// A-17: hammer stuns enemies (ms)
    pub hammer_stun: u32,
    /// A-18: factor for the knockback on enemies
    pub creature_knockback: f32,
    /// A-19: hammer hits all enemies around Elora (shockwave)
    pub hammer_shockwave: bool,
    /// A-20: small follow-up explosions of a grenade
    pub grenade_shards: u32,
    /// A-21: laser hits this many additional enemies
    pub laser_pierce: u32,
    /// A-22: damage from thorns (death tiles in the adventure, E-283)
    pub thorn_damage: i32,
    /// A-23: walking speed in the colorful rush (factor, E-311)
    pub daze_speed: f32,
    /// A-24: sinking in quicksand (units per tick, E-318)
    pub quicksand_sink: f32,
    /// A-25: walking speed in quicksand (factor)
    pub quicksand_speed: f32,
    /// A-26: damage when Elora sinks in completely (then back to the edge)
    pub quicksand_damage: i32,
    /// A-27: walking speed with a full heat bar (factor, E-320)
    pub heat_speed: f32,
    /// A-29: wind pushes Elora in the air (units/tick² at wind 1, R2-W1)
    pub wind_push: f32,
    /// A-30: damage of a lightning strike (E-336)
    pub lightning_damage: i32,
    /// A-31: radius of the strike (units)
    pub lightning_radius: f32,
    /// A-32: warning before the strike (ms)
    pub lightning_warn: u32,
    /// A-33: average interval between strikes at full strength (ms)
    pub lightning_every: u32,
    /// A-34: wind deflects grenades (curvature like T-…, per wind 1)
    pub grenade_wind: f32,
    /// A-35: wet or snowy ground: share of the ice effect at full wetness
    pub wet_slip: f32,
    /// A-36: this long thin ice holds before it breaks under Elora (ms, R2-M2.4)
    pub thin_ice_break: u32,
    /// A-37: broken thin ice grows back (ms)
    pub thin_ice_regrow: u32,
    /// A-38: damage in ice water (then back to the edge)
    pub ice_water_damage: i32,
    /// A-39: snow chunks per avalanche
    pub avalanche_rocks: u32,
    /// A-40: interval between the snow chunks of an avalanche (ms)
    pub avalanche_gap: u32,
    /// A-41: rest after an avalanche before the same slope goes off again (ms)
    pub avalanche_rest: u32,
    /// A-42: pulling up on the climbing wall (units/tick, 0 = sliding only); the session sets
    /// it after the spring spark of the frost spring (D-M24-03)
    pub grip_climb: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            ground_control_speed: 10.5,
            ground_control_accel: 2.2,
            ground_friction: 0.5,
            ground_jump_impulse: 13.6,
            air_jump_impulse: 11.5,
            air_control_speed: 5.0,
            air_control_accel: 1.6,
            air_friction: 0.95,
            gravity: 0.5,
            velramp_start: 550.0,
            velramp_range: 2000.0,
            velramp_curvature: 1.4,
            hook_length: 400.0,
            hook_fire_speed: 85.0,
            hook_drag_accel: 3.0,
            hook_drag_speed: 14.0,
            player_hook_ticks: 55,
            player_hook_force: 1.5,
            player_collision: true,
            player_hooking: true,
            hammer_damage: 3,
            hammer_fire_delay: 125,
            hammer_knockback: 11.0,
            laser_damage: 5,
            laser_fire_delay: 750,
            laser_reach: 850.0,
            laser_bounce_delay: 150,
            laser_bounce_num: 1,
            laser_bounce_cost: 0.0,
            laser_knockback: 2.0,
            grenade_damage: 6,
            grenade_fire_delay: 500,
            grenade_speed: 1050.0,
            grenade_curvature: 7.0,
            grenade_lifetime: 2.0,
            explosion_radius: 135.0,
            explosion_inner_radius: 48.0,
            explosion_max_force: 12.5,
            max_ammo: 10,
            max_health: 10,
            max_armor: 10,
            pickup_respawn: 15.0,
            respawn_delay: 0.5,
            auto_respawn: 3.0,
            ice_friction: 0.985,
            ice_accel: 0.35,
            jump_pad_force: 20.0,
            conveyor_speed: 4.0,
            jerk_speed: 26.0,
            jerk_cooldown: 800,
            jerk_time: 320,
            stomp_speed: 22.0,
            stomp_radius: 64.0,
            grip_time: 1000,
            grip_slide_speed: 1.0,
            wall_jump_x: 9.0,
            wall_jump_y: 12.0,
            glide_fall_speed: 2.0,
            glide_control_speed: 7.0,
            pull_accel: 2.5,
            hit_invulnerable: 1000,
            hit_knockback: 8.0,
            stomp_damage: 3,
            stomp_stun: 1500,
            loot_magnet: 96.0,
            hammer_reach: 14.0,
            hammer_stun: 0,
            creature_knockback: 1.0,
            hammer_shockwave: false,
            grenade_shards: 0,
            laser_pierce: 0,
            thorn_damage: 2,
            daze_speed: 0.55,
            quicksand_sink: 0.22,
            quicksand_speed: 0.4,
            quicksand_damage: 1,
            heat_speed: 0.7,
            wind_push: 0.15,
            lightning_damage: 2,
            lightning_radius: 56.0,
            lightning_warn: 900,
            lightning_every: 9000,
            grenade_wind: 2.2,
            wet_slip: 0.33,
            thin_ice_break: 600,
            thin_ice_regrow: 5500,
            ice_water_damage: 1,
            avalanche_rocks: 7,
            avalanche_gap: 350,
            avalanche_rest: 8000,
            grip_climb: 0.0,
        }
    }
}

/// Converts milliseconds to ticks as integers, like in the original.
pub fn ms_to_ticks(ms: u32) -> u32 {
    ms * crate::TICKS_PER_SECOND / 1000
}

/// Converts seconds to ticks (rounded down, negative values = 0).
#[allow(clippy::cast_sign_loss)] // excluded by `max(0.0)`
pub fn secs_to_ticks(secs: f32) -> u64 {
    (secs.max(0.0) * crate::TICKS_PER_SECOND as f32) as u64
}
