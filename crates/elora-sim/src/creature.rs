//! Kreaturen im Abenteuer (R2-M1, A1.2, E-232 bis E-238): Gegner mit Verhalten und Leben,
//! ihre Geschosse und herumliegende Beute.
//!
//! Die Arten kommen als Daten (`assets/adventure/creatures.toml`); die Simulation kennt nur
//! die Verhaltensmuster. Alles ist deterministisch wie der Rest der Welt.

use crate::math::Vec2;

/// Eine Gegnerart.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreatureKind {
    /// Schlüssel für Karten, Grafik und Übersetzung (z. B. `stachelkaefer`).
    pub name: String,
    /// Kollisionsbox (Breite, Höhe) in Einheiten; Ursprung in der Mitte.
    pub size: [f32; 2],
    pub health: i32,
    /// Schaden bei Berührung (E-232).
    #[cfg_attr(feature = "serde", serde(default))]
    pub touch_damage: i32,
    /// Klein: Heranhooken zieht ihn zu Elora (E-233).
    #[cfg_attr(feature = "serde", serde(default))]
    pub small: bool,
    /// Boss oder besonderer Gegner: bleibt besiegt (E-235, wertet das Abenteuer aus).
    #[cfg_attr(feature = "serde", serde(default))]
    pub boss: bool,
    /// Erfahrung beim Besiegen (wertet das Abenteuer aus).
    #[cfg_attr(feature = "serde", serde(default))]
    pub xp: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub loot: Vec<LootEntry>,
    pub behavior: Behavior,
}

impl CreatureKind {
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.size[0], self.size[1])
    }

    /// Radius für Treffer-Prüfungen (halbe größere Seite).
    pub fn radius(&self) -> f32 {
        self.size[0].max(self.size[1]) / 2.0
    }
}

/// Verhaltensmuster.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum Behavior {
    /// Läuft hin und her, dreht an Wänden und (wenn gewünscht) an Kanten.
    Walker { speed: f32, turn_at_edges: bool },
    /// Wartet am Boden und springt Elora an, sobald sie in Sichtweite ist.
    Hopper {
        wait_ms: u32,
        jump_x: f32,
        jump_y: f32,
        sight: f32,
    },
    /// Steht fest und schießt auf Elora, wenn sie in Reichweite und Sicht ist.
    Turret {
        interval_ms: u32,
        range: f32,
        shot_speed: f32,
        shot_damage: i32,
    },
    /// Schwebt um den Startpunkt und verfolgt Elora in Sichtweite.
    Flyer { speed: f32, sight: f32 },
}

/// Eintrag der Beutetabelle: `min`–`max` Stück mit Wahrscheinlichkeit `chance`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LootEntry {
    pub item: String,
    pub min: u32,
    pub max: u32,
    #[cfg_attr(feature = "serde", serde(default = "one"))]
    pub chance: f32,
}

#[cfg(feature = "serde")]
fn one() -> f32 {
    1.0
}

/// Ein lebender Gegner.
#[derive(Debug, Clone, PartialEq)]
pub struct Creature {
    /// Fest für die Lebensdauer der Welt (Hook, Grafik, Ereignisse).
    pub id: u32,
    /// Index in [`crate::World::creature_kinds`].
    pub kind: usize,
    pub pos: Vec2,
    pub vel: Vec2,
    pub home: Vec2,
    pub health: i32,
    /// Blickrichtung: -1 links, 1 rechts.
    pub facing: i8,
    /// Zähler des Verhaltens (Warten, Nachladen).
    pub timer: u32,
    /// Betäubt (Ticks, Stampfen A-14).
    pub stun: u32,
    /// Tick des letzten Treffers (Lebensbalken, E-238).
    pub hit_tick: Option<u64>,
    pub grounded: bool,
}

/// Geschoss eines Gegners (z. B. Pollenkugel): fliegt gerade.
#[derive(Debug, Clone, PartialEq)]
pub struct CreatureShot {
    pub owner: u32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub damage: i32,
    pub ticks: u32,
}

/// Herumliegende Beute (E-236).
#[derive(Debug, Clone, PartialEq)]
pub struct Loot {
    pub id: u32,
    pub item: String,
    pub count: u32,
    pub pos: Vec2,
    pub vel: Vec2,
    /// Alter in Ticks (erst nach kurzer Zeit einsammelbar).
    pub age: u32,
}

/// Ziel für den Hook (Kreaturen aus Sicht einer Figur).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HookTarget {
    pub id: u32,
    pub pos: Vec2,
    pub radius: f32,
    pub small: bool,
}

/// Fester Pseudo-Zufall (splitmix64) – gleiche Eingabe, gleiches Ergebnis.
pub(crate) fn rng(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Zufallszahl 0..1 aus einem Startwert.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn rng_f32(seed: u64) -> f32 {
    (rng(seed) >> 40) as f32 / (1u64 << 24) as f32
}

/// Datei der Gegnerarten (`assets/adventure/creatures.toml`).
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct KindsFile {
    creature: Vec<CreatureKind>,
}

/// Liest Gegnerarten aus TOML.
///
/// # Errors
/// Bei ungültigem TOML oder fehlenden Feldern.
#[cfg(feature = "serde")]
pub fn kinds_from_toml(src: &str) -> Result<Vec<CreatureKind>, String> {
    toml::from_str::<KindsFile>(src)
        .map(|f| f.creature)
        .map_err(|e| e.to_string())
}
