//! Aussehen einer Karte (E-130 bis E-132): Materialien, Deko, Hintergrund-Ebenen, Animationen.
//!
//! Die Simulation sieht davon nichts; nur der Client zeichnet es.

use elora_sim::Vec2;

/// Farbe mit 8 Bit je Kanal (nicht vormultipliziert).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const WHITE: Self = Self([255, 255, 255, 255]);

    pub const fn hex(rgb: u32) -> Self {
        let [_, r, g, b] = rgb.to_be_bytes();
        Self([r, g, b, 255])
    }
}

/// Art des Wetters (R2-W1, E-333).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherKind {
    /// Kein Wetter (wie bisher).
    #[default]
    Clear,
    Rain,
    /// Gewitter: Regen, Böen und Blitze.
    Storm,
    Fog,
    /// Wind mit Blättern.
    Leaves,
    /// Wind mit Blütenblättern.
    Petals,
    Sandstorm,
    Snow,
    Blizzard,
}

impl WeatherKind {
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

    /// Schlüssel für Daten und Übersetzung (`assets/adventure/worldmap.toml`, `weather.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            Self::Clear => "schoen",
            Self::Rain => "regen",
            Self::Storm => "gewitter",
            Self::Fog => "nebel",
            Self::Leaves => "blaetter",
            Self::Petals => "blueten",
            Self::Sandstorm => "sandsturm",
            Self::Snow => "schnee",
            Self::Blizzard => "schneesturm",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.key() == key)
    }
}

/// Wetter einer Karte (R2-W1, E-329): Art, Stärke (0 = kaum, 1 = voll) und Wind
/// (−1 = stark nach links, 1 = stark nach rechts).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Weather {
    pub kind: WeatherKind,
    pub intensity: f32,
    pub wind: f32,
}

impl Weather {
    pub const CLEAR: Self = Self {
        kind: WeatherKind::Clear,
        intensity: 0.0,
        wind: 0.0,
    };

    /// Gibt es überhaupt Wetter?
    pub fn is_clear(&self) -> bool {
        self.kind == WeatherKind::Clear
    }
}

/// Himmel: senkrechter Verlauf hinter allem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sky {
    pub top: Rgba,
    pub bottom: Rgba,
}

impl Default for Sky {
    /// Bisheriger Himmel (E-089).
    fn default() -> Self {
        Self {
            top: Rgba::hex(0xa9cde8),
            bottom: Rgba::hex(0x7ea8cb),
        }
    }
}

/// Grafik eines Deko-Objekts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Art {
    /// Eingebaute Grafik aus Eloras Satz (Stil A, M6.3), z. B. `"bush-1"`.
    Builtin(String),
    /// In die Karte eingebettetes SVG (Index in [`crate::Map::images`]).
    Image(u16),
}

/// Verweis auf eine Animation mit Zeitversatz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvRef {
    /// Index in [`crate::Map::envelopes`].
    pub index: u16,
    /// Versatz in Millisekunden (gleiche Animation, andere Phase).
    pub offset_ms: i32,
}

/// Frei platziertes Vektor-Objekt (Deko vor/hinter der Spielfläche oder in einer Hintergrund-Ebene).
#[derive(Debug, Clone, PartialEq)]
pub struct Decor {
    pub art: Art,
    /// Mittelpunkt in Welteinheiten (in Hintergrund-Ebenen relativ zur Ebene).
    pub pos: Vec2,
    pub scale: f32,
    /// Drehung in Grad, im Uhrzeigersinn.
    pub rotation: f32,
    pub flip_x: bool,
    /// Färbung (multipliziert), Weiß = unverändert.
    pub tint: Rgba,
    /// Bewegung und Drehung (Envelope der Art [`EnvKind::Position`]).
    pub pos_env: Option<EnvRef>,
    /// Farbe (Envelope der Art [`EnvKind::Color`]).
    pub color_env: Option<EnvRef>,
}

impl Decor {
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

/// Hintergrund-Ebene mit Parallax (E-131).
#[derive(Debug, Clone, PartialEq)]
pub struct Background {
    pub name: String,
    /// Wie stark die Ebene der Kamera folgt: 1 = wie die Spielfläche, 0 = fest.
    pub parallax: Vec2,
    pub offset: Vec2,
    /// Inhalt waagerecht wiederholen (Wolken, Hügel); Abstand in Welteinheiten.
    pub repeat_x: Option<f32>,
    pub items: Vec<Decor>,
}

/// Was eine Animation verändert (wie im Original).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvKind {
    /// Versatz x, y (Welteinheiten) und Drehung (Grad).
    Position,
    /// Farbe r, g, b, a (0 bis 1, multipliziert).
    Color,
}

impl EnvKind {
    /// Wert ohne Wirkung.
    pub fn neutral(self) -> [f32; 4] {
        match self {
            Self::Position => [0.0; 4],
            Self::Color => [1.0; 4],
        }
    }
}

/// Übergang von einem Punkt zum nächsten (wie im Original).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curve {
    Step,
    Linear,
    /// langsam beginnen
    Slow,
    /// schnell beginnen
    Fast,
    /// weich an beiden Enden
    Smooth,
}

impl Curve {
    /// Anteil 0..=1 des Wegs zum nächsten Punkt.
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvPoint {
    pub time_ms: u32,
    pub value: [f32; 4],
    /// Übergang zum nächsten Punkt.
    pub curve: Curve,
}

/// Animation (Envelope, E-132): Punkte mit Kurven, läuft in Schleife.
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    pub name: String,
    pub kind: EnvKind,
    /// An die Spielzeit des Servers gebunden (alle sehen dieselbe Phase) statt an die Uhr des Clients.
    pub synced: bool,
    /// Nach Zeit aufsteigend sortiert.
    pub points: Vec<EnvPoint>,
}

impl Envelope {
    /// Länge einer Schleife.
    pub fn duration_ms(&self) -> u32 {
        self.points.last().map_or(0, |p| p.time_ms)
    }

    /// Wert zur Zeit `time_ms` (läuft in Schleife).
    pub fn eval(&self, time_ms: i64) -> [f32; 4] {
        let (Some(first), Some(last)) = (self.points.first(), self.points.last()) else {
            return self.kind.neutral();
        };
        let duration = i64::from(last.time_ms);
        if duration == 0 {
            return first.value;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let t = time_ms.rem_euclid(duration) as u32;
        let next = self.points.partition_point(|p| p.time_ms <= t);
        if next == 0 {
            return first.value;
        }
        let a = &self.points[next - 1];
        let Some(b) = self.points.get(next) else {
            return a.value;
        };
        #[allow(clippy::cast_precision_loss)]
        let f = a
            .curve
            .ease((t - a.time_ms) as f32 / (b.time_ms - a.time_ms) as f32);
        std::array::from_fn(|i| a.value[i] + (b.value[i] - a.value[i]) * f)
    }
}

/// In die Karte eingebettetes SVG (eigene Deko des Kartenbauers).
///
/// Die Karte prüft nur Größe und Name; das Parsen (ohne externe Verweise) macht der Client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub name: String,
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
    #[allow(clippy::float_cmp)]
    fn envelope_interpolates_and_loops() {
        let e = env(&[
            (0, 0.0, Curve::Linear),
            (1000, 10.0, Curve::Step),
            (2000, 0.0, Curve::Linear),
        ]);
        assert_eq!(e.eval(0)[0], 0.0);
        assert_eq!(e.eval(500)[0], 5.0);
        // Step hält den Wert bis zum nächsten Punkt
        assert_eq!(e.eval(1500)[0], 10.0);
        // Schleife, auch für negative Zeiten (Versatz)
        assert_eq!(e.eval(2500)[0], 5.0);
        assert_eq!(e.eval(-1500)[0], 5.0);
        assert_eq!(env(&[]).eval(10), [0.0; 4]);
        assert_eq!(env(&[(0, 3.0, Curve::Linear)]).eval(77)[0], 3.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn curves_start_and_end_right() {
        for c in [Curve::Linear, Curve::Slow, Curve::Fast, Curve::Smooth] {
            assert_eq!(c.ease(0.0), 0.0);
            assert!((c.ease(1.0) - 1.0).abs() < 1e-6);
        }
        assert!(Curve::Slow.ease(0.5) < 0.5 && Curve::Fast.ease(0.5) > 0.5);
        assert_eq!(Curve::Smooth.ease(0.5), 0.5);
    }
}
