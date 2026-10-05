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
    /// Berührung löst den bunten Rausch aus (so viele ms, E-311): Elora läuft langsamer.
    #[cfg_attr(feature = "serde", serde(default))]
    pub daze_ms: u32,
    /// Panzer (Sandkrabbe, E-317): Treffer von der Seite oder von unten prallen ab, nur von
    /// oben (Schlag, Granate darauf) und Stampfen wirken.
    #[cfg_attr(feature = "serde", serde(default))]
    pub armor: bool,
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
        /// Wirft im Bogen (Schwerkraft) statt gerade zu schießen (Eichhornpirat, R2-M2.2).
        #[cfg_attr(feature = "serde", serde(default))]
        lob: bool,
    },
    /// Schwebt um den Startpunkt und verfolgt Elora in Sichtweite. Mit `hover` bleibt er so
    /// hoch über ihr und lässt alle `drop_ms` einen Funken fallen, der `glow_ms` lang am Boden
    /// glüht (Funkenmotte, R2-M2.3).
    Flyer {
        speed: f32,
        sight: f32,
        #[cfg_attr(feature = "serde", serde(default))]
        hover: f32,
        #[cfg_attr(feature = "serde", serde(default))]
        drop_ms: u32,
        #[cfg_attr(feature = "serde", serde(default))]
        drop_damage: i32,
        #[cfg_attr(feature = "serde", serde(default))]
        glow_ms: u32,
    },
    /// Hüter aus der Luft (R2-M2.1, E-298, E-299): kreist über dem Startpunkt und lässt
    /// Geschosse fallen, visiert Elora an und stürzt herab; danach liegt er benommen am
    /// Boden – **nur dann verwundbar** – und steigt wieder auf. Ab `enrage_at` (Anteil des
    /// Lebens) schneller und zwei Sturzflüge hintereinander, ab `summon_at` ruft er Helfer.
    Diver(Box<DiverDef>),
    /// Steckt im Boden und schießt hoch, wenn Elora näher als `sight` ist (Wurzelschlange,
    /// R2-M2.2); bleibt `out_ms` draußen – **nur dann verwundbar und gefährlich** – und
    /// wartet danach mindestens `hide_ms` versteckt.
    /// Beim Auftauchen wächst sie `rise_ms` lang aus dem Boden (noch harmlos).
    Burrower {
        sight: f32,
        out_ms: u32,
        hide_ms: u32,
        #[cfg_attr(feature = "serde", serde(default))]
        rise_ms: u32,
    },
    /// Hüter am Boden (Wurzelwächter, R2-M2.2, E-307): steht, stößt Wurzeln aus dem Boden
    /// (mit Warnung), Kerne in der Rinde lassen sich per Tauziehen mit dem Hook lösen –
    /// **nur dann verwundbar**. Ab `enrage_at` schneller und mit Wurzelwänden, beim letzten
    /// Kern Wurzeln an zwei Stellen.
    Warden(Box<WardenDef>),
    /// Begleiter (Pilzkind, E-308): folgt Elora am Boden, springt über Stufen, wartet an
    /// Lücken und Gefahren; unverwundbar und harmlos.
    Follower { speed: f32, jump: f32 },
    /// Hüter im Sand (Sandschlange, R2-M2.3, E-316): zieht als Sandspur zu Elora, der Sand
    /// bebt, dann schießt sie im Bogen heraus und liegt nach der Landung benommen am Boden –
    /// **nur im Bogen und benommen verwundbar**. Ab `enrage_at` schneller und Treibsand im
    /// Kessel, ab `double_at` zwei Bögen hintereinander. Stampfen trifft doppelt.
    Serpent(Box<SerpentDef>),
    /// Wandert unter dem Sand (nur die Sandspur ist zu sehen) auf Elora zu, kündigt sich
    /// `warn_ms` lang an und springt im Bogen auf sie zu (Dünenwurm, R2-M2.3); nach der
    /// Landung taucht er ein und ruht `rest_ms`. **Nur im Sprung verwundbar und gefährlich.**
    Leaper {
        sight: f32,
        speed: f32,
        warn_ms: u32,
        jump_x: f32,
        jump_y: f32,
        rest_ms: u32,
    },
}

/// Werte des Hüters aus der Luft ([`Behavior::Diver`]).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiverDef {
    /// Wacht auf, sobald Elora so nah ist.
    pub sight: f32,
    /// Kreis über dem Startpunkt: halbe Breite und Höhe, Tempo (Einheiten/Tick).
    pub circle: [f32; 2],
    pub speed: f32,
    /// So lange kreisen, bevor er anvisiert.
    pub circle_ms: u32,
    /// Warnung vor dem Sturzflug (bleibt stehen und zittert).
    pub aim_ms: u32,
    pub dive_speed: f32,
    /// Benommen am Boden.
    pub stun_ms: u32,
    /// Geschosse beim Kreisen (fallen nach unten); 0 = keine.
    #[cfg_attr(feature = "serde", serde(default))]
    pub drop_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub drop_speed: f32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub drop_damage: i32,
    /// Ab diesem Anteil des Lebens wütend (1,35-faches Tempo, doppelter Sturzflug); 0 = nie.
    #[cfg_attr(feature = "serde", serde(default))]
    pub enrage_at: f32,
    /// Ab diesem Anteil des Lebens ruft er `summon` (Art), höchstens `summon_max` zugleich.
    #[cfg_attr(feature = "serde", serde(default))]
    pub summon_at: f32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summon: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summon_max: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summon_ms: u32,
}

/// Werte des Hüters am Boden ([`Behavior::Warden`]).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WardenDef {
    /// Wacht auf, sobald Elora so nah ist.
    pub sight: f32,
    /// Abstand der Wurzelstöße und Warnzeit davor (Boden bebt).
    pub attack_ms: u32,
    pub warn_ms: u32,
    /// Wurzelstoß: Breite und Höhe der Trefferzone, Schaden.
    pub spike_width: f32,
    pub spike_height: f32,
    pub spike_damage: i32,
    /// So lange vom Wächter weg ziehen, bis ein Kern sich löst; so lange ist er dann offen.
    pub pull_ms: u32,
    pub open_ms: u32,
    /// Kerne in der Rinde (wachsen nach, wenn alle gezogen sind).
    pub cores: u32,
    /// Ab diesem Anteil des Lebens wütend: schneller, Wurzelwände; 0 = nie.
    #[cfg_attr(feature = "serde", serde(default))]
    pub enrage_at: f32,
    /// Wurzelwand: alle `wall_every_ms`, steht `wall_ms`, so viele Tiles hoch.
    #[cfg_attr(feature = "serde", serde(default))]
    pub wall_every_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub wall_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub wall_height: u32,
}

/// Werte des Hüters im Sand ([`Behavior::Serpent`]).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SerpentDef {
    /// Wacht auf, sobald Elora so nah ist.
    pub sight: f32,
    /// Tempo der Sandspur (Einheiten/Tick) und Mindestzeit unter dem Sand.
    pub speed: f32,
    pub trail_ms: u32,
    /// Sand bebt so lange, bevor sie herausschießt.
    pub warn_ms: u32,
    /// Sprung: waagerecht (zur Mitte des Kessels) und nach oben.
    pub jump_x: f32,
    pub jump_y: f32,
    /// Benommen am Boden nach der Landung.
    pub stun_ms: u32,
    /// Ab diesem Anteil des Lebens wütend: schneller, Treibsand im Kessel; 0 = nie.
    #[cfg_attr(feature = "serde", serde(default))]
    pub enrage_at: f32,
    /// Ab diesem Anteil des Lebens zwei Bögen hintereinander; 0 = nie.
    #[cfg_attr(feature = "serde", serde(default))]
    pub double_at: f32,
    /// Treibsand: alle `sand_every_ms` für `sand_ms`, so viele Tiles breit.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sand_every_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sand_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sand_width: u32,
}

/// Zustand des Hüters im Sand (in [`Creature::mode`]).
pub mod serpent {
    pub const SLEEP: u8 = 0;
    /// Unter dem Sand, nur die Spur ist zu sehen.
    pub const TRAIL: u8 = 1;
    /// Sand bebt an [`super::Creature::goal`].
    pub const WARN: u8 = 2;
    /// Im Bogen durch die Luft: gefährlich und verwundbar.
    pub const LEAP: u8 = 3;
    /// Benommen am Boden: verwundbar.
    pub const STUNNED: u8 = 4;
}

/// Zustand des Hüters am Boden (in [`Creature::mode`]).
pub mod warden {
    pub const SLEEP: u8 = 0;
    pub const IDLE: u8 = 1;
    /// Boden bebt an [`super::Creature::goal`], gleich kommt der Wurzelstoß.
    pub const WARN: u8 = 2;
    /// Ein Kern ist gezogen: verwundbar.
    pub const OPEN: u8 = 3;
}

/// Zustand eines Hüters aus der Luft (in [`Creature::mode`]).
pub mod diver {
    /// Wartet, bis Elora kommt.
    pub const SLEEP: u8 = 0;
    pub const CIRCLE: u8 = 1;
    pub const AIM: u8 = 2;
    pub const DIVE: u8 = 3;
    /// Benommen am Boden: verwundbar.
    pub const STUNNED: u8 = 4;
    /// Steigt zurück zum Kreis.
    pub const RISE: u8 = 5;
}

/// Zustand der Wurzelschlange (in [`Creature::mode`]).
pub mod burrow {
    pub const HIDDEN: u8 = 0;
    pub const OUT: u8 = 1;
    /// Wächst gerade aus dem Boden (harmlos, nicht verwundbar).
    pub const RISING: u8 = 2;
}

/// Zustand des Dünenwurms (in [`Creature::mode`]).
pub mod leaper {
    /// Unter dem Sand: harmlos, unverwundbar.
    pub const UNDER: u8 = 0;
    /// Sand bebt, gleich springt er.
    pub const WARN: u8 = 1;
    /// Im Sprung: gefährlich und verwundbar.
    pub const LEAP: u8 = 2;
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
    /// Zustand mehrstufiger Verhalten (Hüter, [`diver`]).
    pub mode: u8,
    /// Ziel des Verhaltens (Sturzflug-Richtung).
    pub goal: Vec2,
    /// Zähler im Zustand (Sturzflüge hintereinander, Winkel beim Kreisen in 1/1000;
    /// beim Wurzelwächter: verbleibende Kerne).
    pub count: u32,
    /// Tauziehen am Wurzelwächter (Ticks) und Takt der Wurzelwände.
    pub tug: u32,
    pub wall_timer: u32,
}

impl Creature {
    /// Kann der Gegner gerade Schaden nehmen? Hüter aus der Luft nur benommen (E-299).
    pub fn vulnerable(&self, kind: &CreatureKind) -> bool {
        match kind.behavior {
            Behavior::Diver(_) => self.mode == diver::STUNNED,
            Behavior::Burrower { .. } => self.mode == burrow::OUT,
            Behavior::Warden(_) => self.mode == warden::OPEN,
            Behavior::Follower { .. } => false,
            Behavior::Leaper { .. } => self.mode == leaper::LEAP,
            Behavior::Serpent(_) => matches!(self.mode, serpent::LEAP | serpent::STUNNED),
            _ => true,
        }
    }

    /// Schadet die Berührung gerade? (Wurzelschlange nur draußen, Begleiter nie.)
    pub fn harmful(&self, kind: &CreatureKind) -> bool {
        match kind.behavior {
            Behavior::Diver(_) => self.mode != diver::STUNNED,
            Behavior::Burrower { .. } => self.mode == burrow::OUT,
            Behavior::Warden(_) => !matches!(self.mode, warden::OPEN | warden::SLEEP),
            Behavior::Follower { .. } => false,
            Behavior::Leaper { .. } => self.mode == leaper::LEAP,
            Behavior::Serpent(_) => self.mode == serpent::LEAP,
            _ => true,
        }
    }

    /// Kann der Hook ihn greifen? (Versteckte Schlangen und Begleiter nicht.)
    pub fn hookable(&self, kind: &CreatureKind) -> bool {
        match kind.behavior {
            Behavior::Burrower { .. } => self.mode == burrow::OUT,
            Behavior::Follower { .. } => false,
            Behavior::Leaper { .. } => self.mode == leaper::LEAP,
            Behavior::Serpent(_) => matches!(self.mode, serpent::LEAP | serpent::STUNNED),
            _ => true,
        }
    }

    /// Prallt ein Treffer aus Richtung `src` am Panzer ab? Nur Treffer von oben wirken
    /// (E-317); ohne Richtung (Stampfen, Werkzeuge) immer.
    pub fn armor_blocks(&self, kind: &CreatureKind, src: Option<Vec2>) -> bool {
        kind.armor && src.is_some_and(|p| p.y > self.pos.y - kind.size[1] / 2.0)
    }
}

/// Geschoss eines Gegners (z. B. Pollenkugel): fliegt gerade.
#[derive(Debug, Clone, PartialEq)]
pub struct CreatureShot {
    pub owner: u32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub damage: i32,
    pub ticks: u32,
    /// Schwerkraft je Tick (0 = fliegt gerade; Nüsse im Bogen).
    pub gravity: f32,
    /// Glüht nach der Landung noch so viele Ticks am Boden (Funken, 0 = verschwindet).
    pub glow: u32,
    /// Liegt glühend am Boden.
    pub landed: bool,
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
    /// Fest verankert (Wurzelwächter): der Hook hält, zieht Elora aber nicht hin.
    pub anchor: bool,
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
