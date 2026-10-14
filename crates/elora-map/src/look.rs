//! Look of a map (E-130 to E-132): materials, decoration, background layers, animations.
//!
//! The simulation sees none of it; only the client draws it.

use elora_sim::Vec2;

/// Color with 8 bits per channel (not premultiplied).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    /// Opaque white.
    pub const WHITE: Self = Self([255, 255, 255, 255]);

    /// Opaque color from `0xRRGGBB`.
    pub const fn hex(rgb: u32) -> Self {
        let [_, r, g, b] = rgb.to_be_bytes();
        Self([r, g, b, 255])
    }
}

/// Kind of weather (R2-W1, E-333).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherKind {
    /// No weather (as before).
    #[default]
    Clear,
    /// Rain.
    Rain,
    /// Thunderstorm: rain, gusts and lightning.
    Storm,
    /// Fog.
    Fog,
    /// Wind with leaves.
    Leaves,
    /// Wind with petals.
    Petals,
    /// Sandstorm.
    Sandstorm,
    /// Snow.
    Snow,
    /// Blizzard: snow with strong wind.
    Blizzard,
}

impl WeatherKind {
    /// All kinds in display order.
    pub const ALL: [Self; 9] = [
        Self::Clear,
        Self::Rain,
        Self::Storm,
        Self::Fog,
        Self::Leaves,
        Self::Petals,
        Self::Sandstorm,
        Self::Snow,
        Self::Blizzard,
    ];

    /// Key for data and translation (`assets/adventure/worldmap.toml`, `weather.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::Rain => "rain",
            Self::Storm => "storm",
            Self::Fog => "fog",
            Self::Leaves => "leaves",
            Self::Petals => "blossoms",
            Self::Sandstorm => "sandstorm",
            Self::Snow => "snow",
            Self::Blizzard => "blizzard",
        }
    }

    /// Kind for a data key.
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.key() == key)
    }
}

/// Weather of a map (R2-W1, E-329): kind, strength (0 = barely, 1 = full) and wind
/// (−1 = strongly to the left, 1 = strongly to the right).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Weather {
    /// Kind of weather.
    pub kind: WeatherKind,
    /// Strength: 0 = barely, 1 = full.
    pub intensity: f32,
    /// Wind: −1 = strongly to the left, 1 = strongly to the right.
    pub wind: f32,
}

impl Weather {
    /// No weather.
    pub const CLEAR: Self = Self {
        kind: WeatherKind::Clear,
        intensity: 0.0,
        wind: 0.0,
    };

    /// Is there any weather at all?
    pub fn is_clear(&self) -> bool {
        self.kind == WeatherKind::Clear
    }
}

/// Sky: vertical gradient behind everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sky {
    /// Color at the top.
    pub top: Rgba,
    /// Color at the bottom.
    pub bottom: Rgba,
}

impl Default for Sky {
    /// Previous sky (E-089).
    fn default() -> Self {
        Self {
            top: Rgba::hex(0xa9cde8),
            bottom: Rgba::hex(0x7ea8cb),
        }
    }
}

/// Graphic of a decoration object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Art {
    /// Built-in graphic from Elora's set (style A, M6.3), e.g. `"bush-1"`.
    Builtin(String),
    /// SVG embedded in the map (index into [`crate::Map::images`]).
    Image(u16),
}

/// Reference to an animation with a time offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvRef {
    /// Index into [`crate::Map::envelopes`].
    pub index: u16,
    /// Offset in milliseconds (same animation, different phase).
    pub offset_ms: i32,
}

/// Freely placed vector object (decoration in front of/behind the playing field or in a
/// background layer).
#[derive(Debug, Clone, PartialEq)]
pub struct Decor {
    /// Graphic.
    pub art: Art,
    /// Center in world units (in background layers relative to the layer).
    pub pos: Vec2,
    /// Size factor (1 = original size).
    pub scale: f32,
    /// Rotation in degrees, clockwise.
    pub rotation: f32,
    /// Mirrored horizontally.
    pub flip_x: bool,
    /// Tint (multiplied), white = unchanged.
    pub tint: Rgba,
    /// Movement and rotation (envelope of kind [`EnvKind::Position`]).
    pub pos_env: Option<EnvRef>,
    /// Color (envelope of kind [`EnvKind::Color`]).
    pub color_env: Option<EnvRef>,
}

impl Decor {
    /// Object without transformation, tint or animation.
    pub fn new(art: Art, pos: Vec2) -> Self {
        Self {
            art,
            pos,
            scale: 1.0,
            rotation: 0.0,
            flip_x: false,
            tint: Rgba::WHITE,
            pos_env: None,
            color_env: None,
        }
    }
}

/// Background layer with parallax (E-131).
#[derive(Debug, Clone, PartialEq)]
pub struct Background {
    /// Name in the editor.
    pub name: String,
    /// How strongly the layer follows the camera: 1 = like the playing field, 0 = fixed.
    pub parallax: Vec2,
    /// Offset of the layer in world units.
    pub offset: Vec2,
    /// Repeat the content horizontally (clouds, hills); spacing in world units.
    pub repeat_x: Option<f32>,
    /// Objects of the layer.
    pub items: Vec<Decor>,
}

/// What an animation changes (as in the original).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvKind {
    /// Offset x, y (world units) and rotation (degrees).
    Position,
    /// Color r, g, b, a (0 to 1, multiplied).
    Color,
}

impl EnvKind {
    /// Value without effect.
    pub fn neutral(self) -> [f32; 4] {
        match self {
            Self::Position => [0.0; 4],
            Self::Color => [1.0; 4],
        }
    }
}

/// Transition from one point to the next (as in the original).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curve {
    /// Jump to the next value only at the next point.
    Step,
    /// Constant speed.
    Linear,
    /// start slowly
    Slow,
    /// start quickly
    Fast,
    /// smooth at both ends
    Smooth,
}

impl Curve {
    /// Fraction 0..=1 of the way to the next point.
    pub fn ease(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Step => 0.0,
            Self::Linear => t,
            Self::Slow => t * t * t,
            Self::Fast => 1.0 - (1.0 - t).powi(3),
            Self::Smooth => t * t * (3.0 - 2.0 * t),
        }
    }
}

/// Point of an animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvPoint {
    /// Time since the start of the loop.
    pub time_ms: u32,
    /// Value (meaning depends on [`EnvKind`]).
    pub value: [f32; 4],
    /// Transition to the next point.
    pub curve: Curve,
}

/// Animation (envelope, E-132): points with curves, runs in a loop.
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    /// Name in the editor.
    pub name: String,
    /// What the animation changes.
    pub kind: EnvKind,
    /// Bound to the server's game time (everyone sees the same phase) instead of the client's
    /// clock.
    pub synced: bool,
    /// Sorted by time, ascending.
    pub points: Vec<EnvPoint>,
}

impl Envelope {
    /// Length of one loop.
    pub fn duration_ms(&self) -> u32 {
        self.points.last().map_or(0, |p| p.time_ms)
    }

    /// Value at time `time_ms` (runs in a loop).
    pub fn eval(&self, time_ms: i64) -> [f32; 4] {
        let (Some(first), Some(last)) = (self.points.first(), self.points.last()) else {
            return self.kind.neutral();
        };
        let duration = i64::from(last.time_ms);
        if duration == 0 {
            return first.value;
        }
        let t = time_ms.rem_euclid(duration) as u32;
        let next = self.points.partition_point(|p| p.time_ms <= t);
        if next == 0 {
            return first.value;
        }
        let a = &self.points[next - 1];
        let Some(b) = self.points.get(next) else {
            return a.value;
        };
        let f = a
            .curve
            .ease((t - a.time_ms) as f32 / (b.time_ms - a.time_ms) as f32);
        std::array::from_fn(|i| a.value[i] + (b.value[i] - a.value[i]) * f)
    }
}

/// SVG embedded in the map (the map maker's own decoration).
///
/// The map only checks size and name; parsing (without external references) is done by the
/// client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// File name.
    pub name: String,
    /// SVG source.
    pub svg: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(points: &[(u32, f32, Curve)]) -> Envelope {
        Envelope {
            name: "t".into(),
            kind: EnvKind::Position,
            synced: false,
            points: points
                .iter()
                .map(|&(time_ms, v, curve)| EnvPoint {
                    time_ms,
                    value: [v, 0.0, 0.0, 0.0],
                    curve,
                })
                .collect(),
        }
    }

    #[test]
    #[allow(clippy::float_cmp)] // exact key values
    fn envelope_interpolates_and_loops() {
        let e = env(&[
            (0, 0.0, Curve::Linear),
            (1000, 10.0, Curve::Step),
            (2000, 0.0, Curve::Linear),
        ]);
        assert_eq!(e.eval(0)[0], 0.0);
        assert_eq!(e.eval(500)[0], 5.0);
        // Step holds the value until the next point
        assert_eq!(e.eval(1500)[0], 10.0);
        // Loop, also for negative times (offset)
        assert_eq!(e.eval(2500)[0], 5.0);
        assert_eq!(e.eval(-1500)[0], 5.0);
        assert_eq!(env(&[]).eval(10), [0.0; 4]);
        assert_eq!(env(&[(0, 3.0, Curve::Linear)]).eval(77)[0], 3.0);
    }

    #[test]
    #[allow(clippy::float_cmp)] // exact values at the ends
    fn curves_start_and_end_right() {
        for c in [Curve::Linear, Curve::Slow, Curve::Fast, Curve::Smooth] {
            assert_eq!(c.ease(0.0), 0.0);
            assert!((c.ease(1.0) - 1.0).abs() < 1e-6);
        }
        assert!(Curve::Slow.ease(0.5) < 0.5 && Curve::Fast.ease(0.5) > 0.5);
        assert_eq!(Curve::Smooth.ease(0.5), 0.5);
    }
}
