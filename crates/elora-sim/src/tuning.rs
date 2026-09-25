//! Einstellbare Simulationswerte (docs/04-tuning.md, E-023).
//!
//! Alle Werte gelten pro Tick bei [`crate::TICKS_PER_SECOND`].

/// Bewegungs- und Hook-Tuning. Die Defaults sind die angenommenen Startwerte
/// T-02 bis T-17.
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
        }
    }
}
