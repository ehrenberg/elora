//! Einstellbare Simulationswerte (docs/handbuch/tuning.md, E-023).
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

    // Fähigkeiten im Abenteuer und im Quellenkampf (R2-M1, A-01 ff.)
    /// A-01: Tempo des Hook-Rucks zum Hook-Punkt (Einheiten/Tick)
    pub ruck_speed: f32,
    /// A-02: Abklingzeit des Hook-Rucks (ms)
    pub ruck_cooldown: u32,
    /// A-04: Fallgeschwindigkeit beim Stampfen (Einheiten/Tick)
    pub stomp_speed: f32,
    /// A-05: Radius der Stoßwelle beim Aufprall (Einheiten)
    pub stomp_radius: f32,
    /// A-06: Haftdauer des Eisgriffs (ms)
    pub grip_time: u32,
    /// A-07: Rutschtempo beim Haften (Einheiten/Tick)
    pub grip_slide_speed: f32,
    /// A-08: Wandsprung seitlich (Einheiten/Tick)
    pub wall_jump_x: f32,
    /// A-08: Wandsprung nach oben (Einheiten/Tick)
    pub wall_jump_y: f32,
    /// A-09: maximale Fallgeschwindigkeit beim Gleiten (Einheiten/Tick)
    pub glide_fall_speed: f32,
    /// A-10: Luftsteuerung beim Gleiten (normal T-07)
    pub glide_control_speed: f32,

    // Kreaturen im Abenteuer (A1.2)
    /// A-03: Zug des Heranhookens auf kleine Gegner (Einheiten/Tick²)
    pub pull_accel: f32,
    /// A-11: Unverwundbar nach einem Treffer im Abenteuer (ms, E-234)
    pub hit_invulnerable: u32,
    /// A-12: Rückstoß bei Berührung eines Gegners (Einheiten/Tick)
    pub hit_knockback: f32,
    /// A-13: Schaden der Stampf-Stoßwelle an Gegnern
    pub stomp_damage: i32,
    /// A-14: Betäubung durch die Stoßwelle (ms)
    pub stomp_stun: u32,
    /// A-15: Reichweite, aus der Beute zu Elora fliegt (Einheiten, E-236)
    pub loot_magnet: f32,

    // Ausbau im Abenteuer (A1.3); Standardwerte = Mehrspieler
    /// A-16: Radius des Hammer-Treffers (Original: halbe Körpergröße)
    pub hammer_reach: f32,
    /// A-17: Hammer betäubt Gegner (ms)
    pub hammer_stun: u32,
    /// A-18: Faktor für den Rückstoß auf Gegner
    pub creature_knockback: f32,
    /// A-19: Hammer trifft alle Gegner um Elora (Schockwelle)
    pub hammer_shockwave: bool,
    /// A-20: kleine Nach-Explosionen einer Granate
    pub grenade_shards: u32,
    /// A-21: Laser trifft so viele Gegner zusätzlich
    pub laser_pierce: u32,
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
            ruck_speed: 16.0,
            ruck_cooldown: 800,
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
