//! Content as data (`assets/adventure/*.toml`): items, skill tree, weapon upgrades, shops and
//! progression values ([`docs/release-2/progression.md`]).

use std::collections::BTreeMap;

use elora_sim::{Ability, CreatureKind, Weapon};
use serde::{Deserialize, Serialize};

use crate::dialog::{CharacterDef, Dialog};
use crate::quest::QuestDef;

/// Text in both languages (E-217).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Text {
    pub de: String,
    pub en: String,
}

impl Text {
    /// Text for a language code (`de`, otherwise English).
    pub fn get(&self, lang: &str) -> &str {
        if lang == "de" { &self.de } else { &self.en }
    }
}

/// Equipment slot (P-20).
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

/// Rarity (P-22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    #[default]
    Common,
    Rare,
    Guardian,
}

/// A bonus from skill tree, equipment or weapon upgrades. Never speed, jump or hook pull
/// (E-212, P-21).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bonus {
    MaxHealth(i32),
    Armor(i32),
    /// Damage of all weapons in percent.
    DamagePct(f32),
    /// Fire delay in percent (negative = faster).
    FireDelayPct(f32),
    /// Knockback on enemies in percent.
    KnockbackPct(f32),
    Ammo(i32),
    /// Loot magnet (units, A-15).
    Magnet(f32),
    /// More gleam drops in percent.
    DropsPct(f32),
    /// Protection after a hit (ms, A-11).
    InvulnerableMs(i32),
    /// Healing plants heal more.
    HealBonus(i32),
    /// Once per map, carry on with this much health instead of dying.
    SecondChance(i32),
    /// Hook jerk cooldown (ms, negative = shorter, A-02).
    RuckCooldownMs(i32),
    HookLengthPct(f32),
    /// Shock wave of the stomp (units, A-05).
    StompRadius(f32),
    StompDamage(i32),
    /// Ice grip hold time (ms, A-06).
    GripMs(i32),
    /// Falling while gliding (units/tick, negative = slower, A-09).
    GlideFall(f32),
    /// Hammer stuns enemies (ms).
    HammerStunMs(i32),
    HammerDamage(i32),
    HammerReachPct(f32),
    /// Hammer hits all enemies around Elora.
    HammerShockwave(i32),
    ExplosionPct(f32),
    /// Small after-explosions of the grenade.
    GrenadeShards(i32),
    LaserBounces(i32),
    /// Laser hits up to this many additional enemies.
    LaserPierce(i32),
    /// Laser charge time in percent.
    LaserDelayPct(f32),
    /// Filling of the heat bar in percent (negative = slower, E-320).
    HeatPct(f32),
    /// Filling of the cold bar in percent (negative = slower, E-342).
    ColdPct(f32),
}

/// Consumable (P-25).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Refill health.
    Heal(i32),
    /// Hook jerk without cooldown for this many seconds.
    Tau(u32),
    /// Refill health and empty the heat bar (cactus fruit, E-320).
    Cool(i32),
    /// Refill health and empty the cold bar (herbal tea, E-342).
    Warm(i32),
}

/// Kind of an item (tab in the inventory, P-24).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemKind {
    /// The currency gleam drops.
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
    /// Keys and quest items: not sellable.
    Key,
    /// Collectible (glitter stone, memory rune …): not sellable, gets counted.
    Collectible,
    /// Ammunition from chests (E-243): refills the weapon, does not go into the inventory.
    Ammo {
        weapon: Weapon,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: Text,
    #[serde(default)]
    pub desc: Text,
    /// Purchase price in gleam drops (0 = not in the shop).
    #[serde(default)]
    pub price: u32,
    #[serde(flatten)]
    pub kind: ItemKind,
}

/// Branch of the skill tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Branch {
    Movement,
    Combat,
    Spring,
}

/// Node of the skill tree (E-242).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillNode {
    pub id: String,
    pub branch: Branch,
    pub name: Text,
    #[serde(default)]
    pub desc: Text,
    pub ranks: u8,
    /// Node above (requires at least rank 1).
    #[serde(default)]
    pub requires: Option<String>,
    /// Required area ability (movement branch).
    #[serde(default)]
    pub ability: Option<Ability>,
    /// Effect per rank.
    pub per_rank: Vec<Bonus>,
}

/// Amount of a material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cost {
    pub item: String,
    pub count: u32,
}

/// Upgrade level of a weapon at Klonk (P-12, P-13).
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

/// Discount from a certain affection to the shop owner (E-248).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discount {
    pub affection: i32,
    pub pct: u32,
}

/// Shop (e.g. Lotte).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shop {
    pub id: String,
    /// Character whose affection determines the discount.
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub discount: Vec<Discount>,
    pub stock: Vec<String>,
}

/// Progression values (P-01, P-02, P-05, P-25, P-26, P-30).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progression {
    pub max_level: u32,
    pub xp_base: u32,
    pub xp_per_level: u32,
    pub base_health: i32,
    /// +1 health every this many levels.
    pub health_every: u32,
    pub consumable_max: u32,
    pub sell_pct: u32,
    pub death_loss_pct: u32,
    pub start_map: String,
    pub start_spawn: String,
}

/// Area on the world map (E-264).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Area {
    pub id: String,
    pub name: Text,
    /// Position on the map (0..1, top left = 0, 0).
    pub pos: [f32; 2],
    /// Colour as `#rrggbb`.
    pub color: String,
    /// Maps of this area start like this (`wiese-` → `wiese-1`, `wiese-2` …).
    pub maps: String,
    /// Background music: `assets/music/<music>.ogg` (E-285).
    #[serde(default)]
    pub music: Option<String>,
    /// Music during a festival (flag `fest`, E-301).
    #[serde(default)]
    pub party_music: Option<String>,
    /// Battle music while a guardian is awake (otherwise `boss`).
    #[serde(default)]
    pub boss_music: Option<String>,
    /// Spring of the area: freed as soon as the flag `befreit.<spring>` is set.
    #[serde(default)]
    pub spring: Option<String>,
    /// Hot area (desert, E-320): heat bar and shimmer.
    #[serde(default)]
    pub hot: bool,
    /// Cold area (Frostspitzen, E-342): cold bar and frost border.
    #[serde(default)]
    pub cold: bool,
    /// Chapter of the area and its guardian (kind from `creatures.toml`): defeating it shows
    /// the victory screen with `victory` and the title of honour `honor`.
    #[serde(default)]
    pub chapter: Option<u32>,
    #[serde(default)]
    pub guardian: Option<String>,
    #[serde(default)]
    pub victory: Option<Text>,
    #[serde(default)]
    pub honor: Option<Text>,
    /// Weather on entering (R2-W1, E-331): gloomy while the spring is silent, otherwise `weather`.
    #[serde(default)]
    pub weather: Vec<WeatherChance>,
    #[serde(default)]
    pub weather_gloomy: Vec<WeatherChance>,
    /// Without its own spring (Tauwinkel): gloomy until this many springs are freed.
    #[serde(default)]
    pub clears_after_springs: Option<i64>,
}

/// A possible weather with weight and ranges for strength and wind (R2-W1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherChance {
    /// `schoen`, `regen`, `gewitter`, `nebel`, `blaetter`, `blueten`, `sandsturm`, `schnee`,
    /// `schneesturm` (siehe [`elora_map::WeatherKind::key`]).
    pub kind: String,
    #[serde(default = "one_u32")]
    pub weight: u32,
    #[serde(default = "default_intensity")]
    pub intensity: [f32; 2],
    #[serde(default = "default_wind")]
    pub wind: [f32; 2],
}

fn one_u32() -> u32 {
    1
}

fn default_intensity() -> [f32; 2] {
    [0.5, 0.8]
}

fn default_wind() -> [f32; 2] {
    [-0.3, 0.3]
}

impl Area {
    /// Is the spring of this area freed?
    pub fn freed(&self, save: &crate::SaveGame) -> bool {
        self.spring
            .as_ref()
            .is_some_and(|s| save.flag(&format!("befreit.{s}")) != 0)
    }
}

/// Error in the content files.
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
#[derive(Debug, Deserialize)]
struct WorldMapFile {
    area: Vec<Area>,
}
#[derive(Debug, Deserialize)]
struct QuestsFile {
    quest: Vec<QuestDef>,
}
#[derive(Debug, Deserialize)]
struct CharactersFile {
    character: Vec<CharacterDef>,
}

/// Bundled dialogs (`assets/adventure/dialogs/<id>.toml`).
macro_rules! dialogs {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_str!(concat!("../../../assets/adventure/dialogs/", $name, ".toml")))),*]
    };
}
const DIALOG_FILES: &[(&str, &str)] = dialogs!(
    "oma",
    "tueftel",
    "lotte",
    "klonk",
    "pip",
    "schild-start",
    "schild-hecke",
    "schild-plattform",
    "schild-hook",
    "schild-hammer",
    "schild-brunnen",
    "schild-ostpfad",
    "schild-wiese",
    "wabe",
    "hummel",
    "schild-wurzeln",
    "schild-ruck",
    "plumm",
    "pilzkind",
    "pilzkind_froh",
    "pilzmama",
    "waechter",
    "schild-westhang",
    "schild-wald",
    "schild-pilzring",
    "schild-zug",
    "sirup",
    "palma",
    "schlange",
    "tafel-1",
    "tafel-2",
    "tafel-kammer",
    "ruinenquelle",
    "giessstelle-1",
    "giessstelle-2",
    "giessstelle-3",
    "oase-bluete",
    "schild-wueste",
    "schild-treibsand",
    "schild-hitze",
    "schild-kammer",
    "schild-stampf",
    "flocke",
    "bolle",
    "kiesel",
    "wicke",
    "bolle-huette",
    "kiesel-huette",
    "wicke-huette",
    "kristella",
    "graue-stelle",
    "schild-bergsteig",
    "schild-kaelte",
    "schild-eis",
    "schild-lawine",
    "schild-kamin",
);

/// Sources of the content files.
#[derive(Debug, Clone, Copy)]
pub struct Sources<'a> {
    pub items: &'a str,
    pub skills: &'a str,
    pub upgrades: &'a str,
    pub shops: &'a str,
    pub progression: &'a str,
    pub creatures: &'a str,
    pub quests: &'a str,
    pub characters: &'a str,
    /// Dialogs: id and content.
    pub dialogs: &'a [(&'a str, &'a str)],
    pub world_map: &'a str,
}

impl Sources<'static> {
    /// The bundled content.
    pub fn builtin() -> Self {
        Self {
            items: include_str!("../../../assets/adventure/items.toml"),
            skills: include_str!("../../../assets/adventure/skills.toml"),
            upgrades: include_str!("../../../assets/adventure/upgrades.toml"),
            shops: include_str!("../../../assets/adventure/shops.toml"),
            progression: include_str!("../../../assets/adventure/progression.toml"),
            creatures: include_str!("../../../assets/adventure/creatures.toml"),
            quests: include_str!("../../../assets/adventure/quests.toml"),
            characters: include_str!("../../../assets/adventure/characters.toml"),
            dialogs: DIALOG_FILES,
            world_map: include_str!("../../../assets/adventure/worldmap.toml"),
        }
    }
}

/// All content of the adventure.
#[derive(Debug, Clone)]
pub struct Content {
    pub items: BTreeMap<String, ItemDef>,
    pub skills: Vec<SkillNode>,
    pub upgrades: Vec<Upgrade>,
    pub shops: BTreeMap<String, Shop>,
    pub progression: Progression,
    pub creatures: Vec<CreatureKind>,
    pub quests: Vec<QuestDef>,
    pub characters: BTreeMap<String, CharacterDef>,
    pub dialogs: BTreeMap<String, Dialog>,
    /// Areas of the world map (E-264).
    pub areas: Vec<Area>,
}

/// Id of the currency.
pub const GLANZTROPFEN: &str = "glanztropfen";

fn parse<T: serde::de::DeserializeOwned>(file: &'static str, src: &str) -> Result<T, ContentError> {
    toml::from_str(src).map_err(|e| ContentError::Parse {
        file,
        msg: e.to_string(),
    })
}

impl Content {
    /// The bundled content.
    ///
    /// # Panics
    /// If the embedded files are faulty (covered by tests).
    pub fn builtin() -> Self {
        Self::load(&Sources::builtin()).unwrap_or_else(|e| panic!("assets/adventure: {e}"))
    }

    /// Read content from a folder (`assets/adventure`) – for the editor (reload without
    /// restart, E-270). Dialogs: all `dialogs/*.toml`.
    ///
    /// # Errors
    /// Missing or invalid files.
    pub fn from_dir(dir: &std::path::Path) -> Result<Self, ContentError> {
        let read = |name: &str| {
            std::fs::read_to_string(dir.join(name))
                .map_err(|e| ContentError::Invalid(format!("{name}: {e}")))
        };
        let (items, skills, upgrades, shops, progression, creatures, quests, characters, world_map) = (
            read("items.toml")?,
            read("skills.toml")?,
            read("upgrades.toml")?,
            read("shops.toml")?,
            read("progression.toml")?,
            read("creatures.toml")?,
            read("quests.toml")?,
            read("characters.toml")?,
            read("worldmap.toml")?,
        );
        let mut dialogs: Vec<(String, String)> = std::fs::read_dir(dir.join("dialogs"))
            .map_err(|e| ContentError::Invalid(format!("dialogs: {e}")))?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .filter_map(|p| {
                let id = p.file_stem()?.to_str()?.to_owned();
                Some(std::fs::read_to_string(&p).map(|t| (id, t)))
            })
            .collect::<Result<_, _>>()
            .map_err(|e| ContentError::Invalid(format!("dialogs: {e}")))?;
        dialogs.sort();
        let refs: Vec<(&str, &str)> = dialogs
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        Self::load(&Sources {
            items: &items,
            skills: &skills,
            upgrades: &upgrades,
            shops: &shops,
            progression: &progression,
            creatures: &creatures,
            quests: &quests,
            characters: &characters,
            dialogs: &refs,
            world_map: &world_map,
        })
    }

    /// Reads and checks the content.
    ///
    /// # Errors
    /// On invalid TOML or references to unknown things.
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
        let quests: QuestsFile = parse("quests.toml", src.quests)?;
        let world_map: WorldMapFile = parse("worldmap.toml", src.world_map)?;
        let characters: CharactersFile = parse("characters.toml", src.characters)?;
        let mut dialogs = BTreeMap::new();
        for &(id, text) in src.dialogs {
            let mut d: Dialog = toml::from_str(text)
                .map_err(|e| ContentError::Invalid(format!("dialogs/{id}.toml: {e}")))?;
            id.clone_into(&mut d.id);
            dialogs.insert(id.to_owned(), d);
        }
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
            quests: quests.quest,
            characters: characters
                .character
                .into_iter()
                .map(|c| (c.id.clone(), c))
                .collect(),
            dialogs,
            areas: world_map.area,
        };
        c.validate()?;
        crate::check::story(&c).map_err(ContentError::Invalid)?;
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
        for a in &self.areas {
            for w in a.weather.iter().chain(&a.weather_gloomy) {
                let range_ok = |r: [f32; 2], lo: f32| r[0] <= r[1] && r[0] >= lo && r[1] <= 1.0;
                if elora_map::WeatherKind::from_key(&w.kind).is_none()
                    || w.weight == 0
                    || !range_ok(w.intensity, 0.0)
                    || !range_ok(w.wind, -1.0)
                {
                    return bad(format!("Gebiet `{}`: Wetter `{}` ungültig", a.id, w.kind));
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

    /// Area a map belongs to.
    pub fn area_of(&self, map: &str) -> Option<&Area> {
        self.areas.iter().find(|a| map.starts_with(&a.maps))
    }

    pub fn quest(&self, id: &str) -> Option<&QuestDef> {
        self.quests.iter().find(|q| q.id == id)
    }

    pub fn dialog(&self, id: &str) -> Option<&Dialog> {
        self.dialogs.get(id)
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

    /// Experience from level `level` to the next (P-02).
    pub fn xp_to_next(&self, level: u32) -> u32 {
        self.progression.xp_base + self.progression.xp_per_level * level
    }
}
