//! Inhalte als Daten (`assets/adventure/*.toml`): Gegenstände, Fähigkeitenbaum, Waffen-Ausbau,
//! Läden und Fortschrittswerte ([`docs/release-2/fortschritt.md`]).

use std::collections::BTreeMap;

use elora_sim::{Ability, CreatureKind, Weapon};
use serde::{Deserialize, Serialize};

/// Text in beiden Sprachen (E-217).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Text {
    pub de: String,
    pub en: String,
}

impl Text {
    /// Text für ein Sprachkürzel (`de`, sonst Englisch).
    pub fn get(&self, lang: &str) -> &str {
        if lang == "de" { &self.de } else { &self.en }
    }
}

/// Platz für Ausrüstung (P-20).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    Hat,
    Cape,
    Boots,
    Pendant,
}

impl Slot {
    pub const ALL: [Self; 4] = [Self::Hat, Self::Cape, Self::Boots, Self::Pendant];
}

/// Seltenheit (P-22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    #[default]
    Common,
    Rare,
    Guardian,
}

/// Ein Bonus aus Fähigkeitenbaum, Ausrüstung oder Waffen-Ausbau. Nie Tempo, Sprung oder
/// Hook-Zug (E-212, P-21).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bonus {
    MaxHealth(i32),
    Armor(i32),
    /// Schaden aller Waffen in Prozent.
    DamagePct(f32),
    /// Feuerverzögerung in Prozent (negativ = schneller).
    FireDelayPct(f32),
    /// Rückstoß auf Gegner in Prozent.
    KnockbackPct(f32),
    Ammo(i32),
    /// Beute-Magnet (Einheiten, A-15).
    Magnet(f32),
    /// Mehr Glanztropfen in Prozent.
    DropsPct(f32),
    /// Schutz nach Treffer (ms, A-11).
    InvulnerableMs(i32),
    /// Heilpflanzen heilen mehr.
    HealBonus(i32),
    /// Einmal je Karte mit so viel Leben weitermachen statt zu sterben.
    SecondChance(i32),
    /// Abklingzeit Hook-Ruck (ms, negativ = kürzer, A-02).
    RuckCooldownMs(i32),
    HookLengthPct(f32),
    /// Stoßwelle des Stampfens (Einheiten, A-05).
    StompRadius(f32),
    StompDamage(i32),
    /// Haftdauer Eisgriff (ms, A-06).
    GripMs(i32),
    /// Fallen beim Gleiten (Einheiten/Tick, negativ = langsamer, A-09).
    GlideFall(f32),
    /// Hammer betäubt Gegner (ms).
    HammerStunMs(i32),
    HammerDamage(i32),
    HammerReachPct(f32),
    /// Hammer trifft alle Gegner um Elora.
    HammerShockwave(i32),
    ExplosionPct(f32),
    /// Kleine Nach-Explosionen der Granate.
    GrenadeShards(i32),
    LaserBounces(i32),
    /// Laser trifft bis zu so viele Gegner zusätzlich.
    LaserPierce(i32),
    /// Ladezeit des Lasers in Prozent.
    LaserDelayPct(f32),
}

/// Verbrauchsgegenstand (P-25).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Leben auffüllen.
    Heal(i32),
    /// Hook-Ruck ohne Abklingzeit für so viele Sekunden.
    Tau(u32),
}

/// Art eines Gegenstands (Reiter im Inventar, P-24).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemKind {
    /// Die Währung Glanztropfen.
    Currency,
    Equipment {
        slot: Slot,
        #[serde(default)]
        rarity: Rarity,
        bonuses: Vec<Bonus>,
    },
    Consumable {
        effect: Effect,
    },
    Material,
    /// Schlüssel und Aufgabengegenstände: nicht verkaufbar.
    Key,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: Text,
    #[serde(default)]
    pub desc: Text,
    /// Kaufpreis in Glanztropfen (0 = nicht im Laden).
    #[serde(default)]
    pub price: u32,
    #[serde(flatten)]
    pub kind: ItemKind,
}

/// Zweig des Fähigkeitenbaums.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Branch {
    Movement,
    Combat,
    Spring,
}

/// Knoten des Fähigkeitenbaums (E-242).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillNode {
    pub id: String,
    pub branch: Branch,
    pub name: Text,
    #[serde(default)]
    pub desc: Text,
    pub ranks: u8,
    /// Knoten darüber (mindestens Rang 1 nötig).
    #[serde(default)]
    pub requires: Option<String>,
    /// Nötige Gebietsfähigkeit (Bewegungs-Zweig).
    #[serde(default)]
    pub ability: Option<Ability>,
    /// Wirkung je Rang.
    pub per_rank: Vec<Bonus>,
}

/// Menge eines Materials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cost {
    pub item: String,
    pub count: u32,
}

/// Ausbaustufe einer Waffe bei Klonk (P-12, P-13).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Upgrade {
    pub weapon: Weapon,
    pub level: u8,
    pub name: Text,
    #[serde(default)]
    pub desc: Text,
    pub glanztropfen: u32,
    #[serde(default)]
    pub materials: Vec<Cost>,
    pub bonuses: Vec<Bonus>,
}

/// Laden (z. B. Lotte).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shop {
    pub id: String,
    pub stock: Vec<String>,
}

/// Fortschrittswerte (P-01, P-02, P-05, P-25, P-26, P-30).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progression {
    pub max_level: u32,
    pub xp_base: u32,
    pub xp_per_level: u32,
    pub base_health: i32,
    /// Alle so viele Stufen +1 Leben.
    pub health_every: u32,
    pub consumable_max: u32,
    pub sell_pct: u32,
    pub death_loss_pct: u32,
}

/// Fehler in den Inhaltsdateien.
#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("{file}: {msg}")]
    Parse { file: &'static str, msg: String },
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug, Deserialize)]
struct ItemsFile {
    item: Vec<ItemDef>,
}
#[derive(Debug, Deserialize)]
struct SkillsFile {
    node: Vec<SkillNode>,
}
#[derive(Debug, Deserialize)]
struct UpgradesFile {
    upgrade: Vec<Upgrade>,
}
#[derive(Debug, Deserialize)]
struct ShopsFile {
    shop: Vec<Shop>,
}

/// Quelltexte der Inhaltsdateien.
#[derive(Debug, Clone, Copy)]
pub struct Sources<'a> {
    pub items: &'a str,
    pub skills: &'a str,
    pub upgrades: &'a str,
    pub shops: &'a str,
    pub progression: &'a str,
    pub creatures: &'a str,
}

impl Sources<'static> {
    /// Die mitgelieferten Inhalte.
    pub fn builtin() -> Self {
        Self {
            items: include_str!("../../../assets/adventure/items.toml"),
            skills: include_str!("../../../assets/adventure/skills.toml"),
            upgrades: include_str!("../../../assets/adventure/upgrades.toml"),
            shops: include_str!("../../../assets/adventure/shops.toml"),
            progression: include_str!("../../../assets/adventure/progression.toml"),
            creatures: include_str!("../../../assets/adventure/creatures.toml"),
        }
    }
}

/// Alle Inhalte des Abenteuers.
#[derive(Debug, Clone)]
pub struct Content {
    pub items: BTreeMap<String, ItemDef>,
    pub skills: Vec<SkillNode>,
    pub upgrades: Vec<Upgrade>,
    pub shops: BTreeMap<String, Shop>,
    pub progression: Progression,
    pub creatures: Vec<CreatureKind>,
}

/// Id der Währung.
pub const GLANZTROPFEN: &str = "glanztropfen";

fn parse<T: serde::de::DeserializeOwned>(file: &'static str, src: &str) -> Result<T, ContentError> {
    toml::from_str(src).map_err(|e| ContentError::Parse {
        file,
        msg: e.to_string(),
    })
}

impl Content {
    /// Die mitgelieferten Inhalte.
    ///
    /// # Panics
    /// Wenn die eingebetteten Dateien fehlerhaft sind (wird von Tests abgedeckt).
    pub fn builtin() -> Self {
        Self::load(&Sources::builtin()).unwrap_or_else(|e| panic!("assets/adventure: {e}"))
    }

    /// Liest und prüft die Inhalte.
    ///
    /// # Errors
    /// Bei ungültigem TOML oder Verweisen auf Unbekanntes.
    pub fn load(src: &Sources<'_>) -> Result<Self, ContentError> {
        let items: ItemsFile = parse("items.toml", src.items)?;
        let skills: SkillsFile = parse("skills.toml", src.skills)?;
        let upgrades: UpgradesFile = parse("upgrades.toml", src.upgrades)?;
        let shops: ShopsFile = parse("shops.toml", src.shops)?;
        let progression: Progression = parse("progression.toml", src.progression)?;
        let creatures = elora_sim::creature::kinds_from_toml(src.creatures).map_err(|msg| {
            ContentError::Parse {
                file: "creatures.toml",
                msg,
            }
        })?;
        let mut map = BTreeMap::new();
        for it in items.item {
            if let Some(old) = map.insert(it.id.clone(), it) {
                return Err(ContentError::Invalid(format!(
                    "Gegenstand `{}` doppelt",
                    old.id
                )));
            }
        }
        let c = Self {
            items: map,
            skills: skills.node,
            upgrades: upgrades.upgrade,
            shops: shops.shop.into_iter().map(|s| (s.id.clone(), s)).collect(),
            progression,
            creatures,
        };
        c.validate()?;
        Ok(c)
    }

    fn validate(&self) -> Result<(), ContentError> {
        let bad = |m: String| Err(ContentError::Invalid(m));
        if !matches!(
            self.items.get(GLANZTROPFEN).map(|i| &i.kind),
            Some(ItemKind::Currency)
        ) {
            return bad(format!("`{GLANZTROPFEN}` fehlt oder ist keine Währung"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for n in &self.skills {
            if !ids.insert(&n.id) {
                return bad(format!("Knoten `{}` doppelt", n.id));
            }
            if n.ranks == 0 || n.per_rank.is_empty() {
                return bad(format!("Knoten `{}` ohne Ränge oder Wirkung", n.id));
            }
        }
        for n in &self.skills {
            if let Some(r) = &n.requires
                && !self
                    .skills
                    .iter()
                    .any(|m| &m.id == r && m.branch == n.branch)
            {
                return bad(format!("Knoten `{}` braucht unbekanntes `{r}`", n.id));
            }
        }
        for u in &self.upgrades {
            for m in &u.materials {
                if !self.items.contains_key(&m.item) {
                    return bad(format!(
                        "Ausbau {:?} {}: Material `{}` unbekannt",
                        u.weapon, u.level, m.item
                    ));
                }
            }
        }
        for w in Weapon::ALL {
            let mut levels: Vec<u8> = self
                .upgrades
                .iter()
                .filter(|u| u.weapon == w)
                .map(|u| u.level)
                .collect();
            levels.sort_unstable();
            if levels
                .iter()
                .enumerate()
                .any(|(i, &l)| usize::from(l) != i + 1)
            {
                return bad(format!("Ausbaustufen von {w:?} nicht lückenlos ab 1"));
            }
        }
        for s in self.shops.values() {
            for it in &s.stock {
                match self.items.get(it) {
                    Some(d) if d.price > 0 => {}
                    _ => {
                        return bad(format!(
                            "Laden `{}`: `{it}` unbekannt oder ohne Preis",
                            s.id
                        ));
                    }
                }
            }
        }
        for k in &self.creatures {
            for l in &k.loot {
                if !self.items.contains_key(&l.item) {
                    return bad(format!("Beute von `{}`: `{}` unbekannt", k.name, l.item));
                }
            }
        }
        Ok(())
    }

    pub fn item(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }

    pub fn skill(&self, id: &str) -> Option<&SkillNode> {
        self.skills.iter().find(|n| n.id == id)
    }

    pub fn upgrade(&self, weapon: Weapon, level: u8) -> Option<&Upgrade> {
        self.upgrades
            .iter()
            .find(|u| u.weapon == weapon && u.level == level)
    }

    /// Erfahrung von Stufe `level` zur nächsten (P-02).
    pub fn xp_to_next(&self, level: u32) -> u32 {
        self.progression.xp_base + self.progression.xp_per_level * level
    }
}
