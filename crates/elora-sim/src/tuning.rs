//! Einstellbare Simulationswerte (docs/04-tuning.md, E-023).
//!
//! Alle Werte gelten pro Tick bei [`crate::TICKS_PER_SECOND`].

/// Bewegungs-, Hook- und Waffen-Tuning. Die Defaults sind die angenommenen
/// Startwerte T-02 bis T-30 sowie E-052.
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
    /// T-16: maximale Dauer, die ein Spieler einen anderen festhalten kann (Ticks)
    pub player_hook_ticks: u32,
    /// T-17: Kraftfaktor auf den gehookten Spieler
    pub player_hook_force: f32,
    /// Spieler kollidieren miteinander
    pub player_collision: bool,
    /// Spieler können einander hooken
    pub player_hooking: bool,

    /// T-18: Hammer-Schaden
    pub hammer_damage: i32,
    /// T-18: Hammer-Feuerverzögerung (ms)
    pub hammer_fire_delay: u32,
    /// T-19: Stärke des Hammer-Knockbacks (Original: 10)
    pub hammer_knockback: f32,
    /// T-20: Laser-Schaden
    pub laser_damage: i32,
    /// T-20: Laser-Feuerverzögerung (ms)
    pub laser_fire_delay: u32,
    /// T-21: Laser-Reichweite
    pub laser_reach: f32,
    /// T-22: Verzögerung bis zum Abprall (ms)
    pub laser_bounce_delay: u32,
    /// T-22: Anzahl Abpraller
    pub laser_bounce_num: u32,
    /// Reichweitenverlust pro Abprall
    pub laser_bounce_cost: f32,
    /// E-052: Laser-Knockback in Schussrichtung (Original: 0)
    pub laser_knockback: f32,
    /// T-23: maximaler Granaten-Schaden (Explosionsmitte)
    pub grenade_damage: i32,
    /// T-23: Granaten-Feuerverzögerung (ms)
    pub grenade_fire_delay: u32,
    /// T-24: Granaten-Geschwindigkeit
    pub grenade_speed: f32,
    /// T-24: Krümmung der Flugbahn
    pub grenade_curvature: f32,
    /// T-24: Lebensdauer (s)
    pub grenade_lifetime: f32,
    /// T-25: Explosionsradius
    pub explosion_radius: f32,
    /// T-25: innerer Radius mit vollem Schaden
    pub explosion_inner_radius: f32,
    /// T-25: maximale Explosionskraft
    pub explosion_max_force: f32,
    /// T-27: maximale Munition
    pub max_ammo: i32,
    /// T-28: maximale Lebenspunkte
    pub max_health: i32,
    /// T-28: maximale Rüstung
    pub max_armor: i32,
    /// T-29: Respawn-Zeit der Pickups (s)
    pub pickup_respawn: f32,
    /// T-30: Mindestzeit bis zum Respawn nach dem Tod (s)
    pub respawn_delay: f32,
    /// Automatischer Respawn ohne Klick (s, Original: 3)
    pub auto_respawn: f32,
    /// T-31: Bodenreibung auf Eis (normal T-04)
    pub ice_friction: f32,
    /// T-32: Beschleunigung auf Eis (normal T-03)
    pub ice_accel: f32,
    /// T-33: Kraft des Sprungfelds (Einheiten/Tick)
    pub jump_pad_force: f32,
    /// T-35: Geschwindigkeit des Beschleunigers (Einheiten/Tick)
    pub conveyor_speed: f32,
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
        }
    }
}

/// Wandelt Millisekunden wie im Original ganzzahlig in Ticks um.
pub fn ms_to_ticks(ms: u32) -> u32 {
    ms * crate::TICKS_PER_SECOND / 1000
}

/// Wandelt Sekunden in Ticks um (abgerundet, negative Werte = 0).
#[allow(clippy::cast_sign_loss)] // durch `max(0.0)` ausgeschlossen
pub fn secs_to_ticks(secs: f32) -> u64 {
    (secs.max(0.0) * crate::TICKS_PER_SECOND as f32) as u64
}
