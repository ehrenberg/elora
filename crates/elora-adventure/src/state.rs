//! Save game and its rules: levels, skill tree, inventory, equipment, weapons, shops,
//! death and saving (E-219, E-220, E-241 to E-244).

use std::collections::{BTreeMap, BTreeSet};

use elora_sim::{Abilities, Ability, Event, Tuning, Weapon};
use serde::{Deserialize, Serialize};

use crate::data::{Branch, Content, Effect, GLANZTROPFEN, ItemKind, Slot};
use crate::stats::Stats;

/// Flag after Tüftel's work with the spring spark of the frost spring (D-M24-03).
pub const STRONG_GRIP: &str = "eisgriff.stark";
/// Pulling up on the climbing wall with strengthened ice grip (A-42, units/tick).
pub const STRONG_GRIP_CLIMB: f32 = 1.6;

/// Place in the adventure: map and save point or entrance.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Location {
    pub map: String,
    pub spawn: String,
}

/// Everything a save game contains (P-33).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveGame {
    pub level: u32,
    /// Experience within the current level.
    pub xp: u32,
    pub health: i32,
    pub glanztropfen: u32,
    /// Collected since the last save point (lost on death, E-220).
    pub glanz_since_save: u32,
    /// Additional dewdrop points from quests.
    pub bonus_points: u32,
    /// Learned nodes with rank.
    pub skills: BTreeMap<String, u8>,
    pub inventory: BTreeMap<String, u32>,
    pub equipped: BTreeMap<Slot, String>,
    /// Owned weapons with upgrade level (0 = not upgraded).
    pub weapons: BTreeMap<Weapon, u8>,
    /// Area abilities (bits of [`Abilities`]).
    pub abilities: u8,
    /// World state: switches, doors, chests, quests, effects from dialogs …
    pub flags: BTreeMap<String, i64>,
    /// Broken crumble floor per map (E-230).
    pub broken: BTreeMap<String, BTreeSet<(i32, i32)>>,
    /// Defeated bosses and special enemies (E-235), e.g. `wiese-3:hummel`.
    pub defeated: BTreeSet<String>,
    pub location: Location,
    pub play_time_secs: u64,
    /// Started quests (A1.4).
    #[serde(default)]
    pub quests: BTreeMap<String, crate::quest::QuestState>,
    /// Affection per character (E-248).
    #[serde(default)]
    pub affection: BTreeMap<String, i32>,
    /// Ammunition of the weapons (E-243).
    #[serde(default)]
    pub ammo: BTreeMap<Weapon, i32>,
}

/// What the player should see (display, sound).
#[derive(Debug, Clone, PartialEq)]
pub enum Notice {
    LevelUp { level: u32 },
    Xp(u32),
    Item { id: String, count: u32 },
    QuestStarted(String),
    QuestStep(String),
    QuestDone(String),
    QuestFailed(String),
}

/// Why something is not possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("unbekannt")]
    Unknown,
    #[error("keine Punkte")]
    NoPoints,
    #[error("höchster Rang erreicht")]
    MaxRank,
    #[error("Knoten darüber fehlt")]
    Locked,
    #[error("Gebietsfähigkeit fehlt")]
    NeedsAbility,
    #[error("zu wenig Glanztropfen")]
    TooExpensive,
    #[error("Material fehlt")]
    MissingMaterial,
    #[error("nicht vorhanden")]
    NotOwned,
    #[error("passt nicht")]
    WrongKind,
    #[error("Tasche voll")]
    Full,
    #[error("Waffe fehlt")]
    NoWeapon,
    #[error("höchste Stufe erreicht")]
    MaxLevel,
}

impl SaveGame {
    /// New game: level 1, no weapon yet (Klonk hands out the hammer in the prologue), full health.
    pub fn new(content: &Content, start: Location) -> Self {
        Self {
            level: 1,
            xp: 0,
            health: content.progression.base_health,
            glanztropfen: 0,
            glanz_since_save: 0,
            bonus_points: 0,
            skills: BTreeMap::new(),
            inventory: BTreeMap::new(),
            equipped: BTreeMap::new(),
            weapons: BTreeMap::new(),
            abilities: 0,
            flags: BTreeMap::new(),
            broken: BTreeMap::new(),
            defeated: BTreeSet::new(),
            location: start,
            play_time_secs: 0,
            quests: BTreeMap::new(),
            affection: BTreeMap::new(),
            ammo: BTreeMap::new(),
        }
    }

    pub fn abilities(&self) -> Abilities {
        Abilities::from_bits(self.abilities)
    }

    pub fn grant_ability(&mut self, a: Ability) {
        let mut set = self.abilities();
        set.set(a, true);
        self.abilities = set.bits();
    }

    pub fn flag(&self, key: &str) -> i64 {
        self.flags.get(key).copied().unwrap_or(0)
    }

    pub fn set_flag(&mut self, key: &str, value: i64) {
        if value == 0 {
            self.flags.remove(key);
        } else {
            self.flags.insert(key.to_owned(), value);
        }
    }

    pub fn count(&self, item: &str) -> u32 {
        if item == GLANZTROPFEN {
            self.glanztropfen
        } else {
            self.inventory.get(item).copied().unwrap_or(0)
        }
    }

    // ------------------------------------------------------------ Levels

    /// Free dewdrop points.
    pub fn free_points(&self) -> u32 {
        let spent: u32 = self.skills.values().map(|&r| u32::from(r)).sum();
        (self.level - 1 + self.bonus_points).saturating_sub(spent)
    }

    /// Credit experience; returns level-ups. At the maximum level it stops.
    pub fn add_xp(&mut self, content: &Content, amount: u32) -> Vec<Notice> {
        let mut out = vec![Notice::Xp(amount)];
        self.xp += amount;
        while self.level < content.progression.max_level {
            let need = content.xp_to_next(self.level);
            if self.xp < need {
                break;
            }
            self.xp -= need;
            self.level += 1;
            self.health = self.max_health(content);
            out.push(Notice::LevelUp { level: self.level });
        }
        if self.level >= content.progression.max_level {
            self.xp = 0;
        }
        out
    }

    /// All values from tree, equipment and weapon upgrades.
    pub fn stats(&self, content: &Content) -> Stats {
        let mut s = Stats::default();
        for (id, &rank) in &self.skills {
            if let Some(n) = content.skill(id) {
                for _ in 0..rank {
                    for b in &n.per_rank {
                        s.add(b);
                    }
                }
            }
        }
        for id in self.equipped.values() {
            if let Some(ItemKind::Equipment { bonuses, .. }) = content.item(id).map(|i| &i.kind) {
                for b in bonuses {
                    s.add(b);
                }
            }
        }
        for (&w, &lvl) in &self.weapons {
            for l in 1..=lvl {
                if let Some(u) = content.upgrade(w, l) {
                    for b in &u.bonuses {
                        s.add(b);
                    }
                }
            }
        }
        s
    }

    /// Maximum health: base value + one every `health_every` levels + bonuses (E-241).
    pub fn max_health(&self, content: &Content) -> i32 {
        let p = &content.progression;
        let from_level = i32::try_from((self.level - 1) / p.health_every.max(1)).unwrap_or(0);
        p.base_health + from_level + self.stats(content).max_health
    }

    /// Simulation tuning for this save game.
    pub fn tuning(&self, content: &Content, base: &Tuning) -> Tuning {
        let mut t = self.stats(content).apply(base);
        t.max_health = self.max_health(content);
        // Spring spark of the frost spring (D-M24-03): climbing claws hold twice as long and
        // pull Elora up the wall
        if self.flag(STRONG_GRIP) != 0 {
            t.grip_time = t.grip_time.saturating_mul(2);
            t.grip_climb = STRONG_GRIP_CLIMB;
        }
        t
    }

    // ------------------------------------------------------------ Skill tree

    /// Can the node grow by one rank?
    ///
    /// # Errors
    /// With the reason why not.
    pub fn can_learn(&self, content: &Content, id: &str) -> Result<(), Refusal> {
        let n = content.skill(id).ok_or(Refusal::Unknown)?;
        let rank = self.skills.get(id).copied().unwrap_or(0);
        if rank >= n.ranks {
            return Err(Refusal::MaxRank);
        }
        if let Some(req) = &n.requires
            && self.skills.get(req).copied().unwrap_or(0) == 0
        {
            return Err(Refusal::Locked);
        }
        if let Some(a) = n.ability
            && !self.abilities().has(a)
        {
            return Err(Refusal::NeedsAbility);
        }
        if self.free_points() == 0 {
            return Err(Refusal::NoPoints);
        }
        Ok(())
    }

    /// # Errors
    /// See [`Self::can_learn`].
    pub fn learn(&mut self, content: &Content, id: &str) -> Result<u8, Refusal> {
        self.can_learn(content, id)?;
        let r = self.skills.entry(id.to_owned()).or_insert(0);
        *r += 1;
        Ok(*r)
    }

    /// Nodes of a branch in tree order.
    pub fn branch(
        content: &Content,
        branch: Branch,
    ) -> impl Iterator<Item = &crate::data::SkillNode> {
        content.skills.iter().filter(move |n| n.branch == branch)
    }

    // ------------------------------------------------------------ Inventory

    /// Add an item; consumables at most `consumable_max` (P-25).
    ///
    /// # Errors
    /// Unknown item or full bag for consumables.
    pub fn add_item(&mut self, content: &Content, id: &str, count: u32) -> Result<(), Refusal> {
        let def = content.item(id).ok_or(Refusal::Unknown)?;
        match def.kind {
            ItemKind::Currency => {
                self.glanztropfen += count;
                self.glanz_since_save += count;
            }
            // Ammunition refills the weapon in the world (session), not the inventory
            ItemKind::Ammo { .. } => {}
            ItemKind::Consumable { .. } => {
                let max = content.progression.consumable_max;
                let have = self.count(id);
                if have >= max {
                    return Err(Refusal::Full);
                }
                *self.inventory.entry(id.to_owned()).or_insert(0) = (have + count).min(max);
            }
            _ => *self.inventory.entry(id.to_owned()).or_insert(0) += count,
        }
        Ok(())
    }

    /// # Errors
    /// If not enough is available.
    pub fn remove_item(&mut self, id: &str, count: u32) -> Result<(), Refusal> {
        if id == GLANZTROPFEN {
            self.glanztropfen = self
                .glanztropfen
                .checked_sub(count)
                .ok_or(Refusal::TooExpensive)?;
            return Ok(());
        }
        let have = self.inventory.get_mut(id).ok_or(Refusal::NotOwned)?;
        *have = have.checked_sub(count).ok_or(Refusal::NotOwned)?;
        if *have == 0 {
            self.inventory.remove(id);
        }
        Ok(())
    }

    /// Use a consumable; the caller applies the effect (healing in the world).
    ///
    /// # Errors
    /// Not available or not a consumable.
    pub fn use_item(&mut self, content: &Content, id: &str) -> Result<Effect, Refusal> {
        let Some(ItemKind::Consumable { effect }) = content.item(id).map(|i| &i.kind) else {
            return Err(Refusal::WrongKind);
        };
        let effect = *effect;
        self.remove_item(id, 1)?;
        if let Effect::Heal(h) | Effect::Cool(h) | Effect::Warm(h) = effect {
            self.health = (self.health + h).min(self.max_health(content));
        }
        Ok(effect)
    }

    /// Put on equipment; a piece worn before goes back into the inventory.
    ///
    /// # Errors
    /// Not available or not equipment.
    pub fn equip(&mut self, content: &Content, id: &str) -> Result<(), Refusal> {
        let Some(ItemKind::Equipment { slot, .. }) = content.item(id).map(|i| &i.kind) else {
            return Err(Refusal::WrongKind);
        };
        let slot = *slot;
        self.remove_item(id, 1)?;
        if let Some(old) = self.equipped.insert(slot, id.to_owned()) {
            *self.inventory.entry(old).or_insert(0) += 1;
        }
        self.health = self.health.min(self.max_health(content));
        Ok(())
    }

    pub fn unequip(&mut self, content: &Content, slot: Slot) {
        if let Some(old) = self.equipped.remove(&slot) {
            *self.inventory.entry(old).or_insert(0) += 1;
        }
        self.health = self.health.min(self.max_health(content));
    }

    // ------------------------------------------------------------ Shops and upgrades

    /// # Errors
    /// Unknown shop or item, too expensive, bag full.
    pub fn buy(&mut self, content: &Content, shop: &str, id: &str) -> Result<(), Refusal> {
        let s = content.shops.get(shop).ok_or(Refusal::Unknown)?;
        if !s.stock.iter().any(|i| i == id) {
            return Err(Refusal::Unknown);
        }
        let price = self.price(content, shop, id).ok_or(Refusal::Unknown)?;
        if self.glanztropfen < price {
            return Err(Refusal::TooExpensive);
        }
        self.add_item(content, id, 1)?;
        self.glanztropfen -= price;
        Ok(())
    }

    /// Purchase price with discount based on affection to the owner (E-248).
    pub fn price(&self, content: &Content, shop: &str, id: &str) -> Option<u32> {
        let s = content.shops.get(shop)?;
        let base = content.item(id)?.price;
        let pct = s.owner.as_deref().map_or(0, |o| {
            let a = self.affection(o);
            s.discount
                .iter()
                .filter(|d| a >= d.affection)
                .map(|d| d.pct)
                .max()
                .unwrap_or(0)
        });
        Some(base - base * pct.min(100) / 100)
    }

    /// Selling price (P-26); keys and currency cannot be sold.
    pub fn sell_price(content: &Content, id: &str) -> Option<u32> {
        let d = content.item(id)?;
        if matches!(
            d.kind,
            ItemKind::Key | ItemKind::Currency | ItemKind::Collectible | ItemKind::Ammo { .. }
        ) {
            return None;
        }
        Some(d.price * content.progression.sell_pct / 100)
    }

    /// # Errors
    /// Not sellable or not available.
    pub fn sell(&mut self, content: &Content, id: &str) -> Result<u32, Refusal> {
        let price = Self::sell_price(content, id).ok_or(Refusal::WrongKind)?;
        self.remove_item(id, 1)?;
        self.glanztropfen += price;
        Ok(price)
    }

    pub fn give_weapon(&mut self, w: Weapon) {
        self.weapons.entry(w).or_insert(0);
    }

    /// Next upgrade level at Klonk (P-12, P-13).
    ///
    /// # Errors
    /// Weapon missing, highest level, too expensive, material missing.
    pub fn upgrade(&mut self, content: &Content, w: Weapon) -> Result<u8, Refusal> {
        let level = *self.weapons.get(&w).ok_or(Refusal::NoWeapon)?;
        let u = content.upgrade(w, level + 1).ok_or(Refusal::MaxLevel)?;
        if self.glanztropfen < u.glanztropfen {
            return Err(Refusal::TooExpensive);
        }
        if u.materials.iter().any(|m| self.count(&m.item) < m.count) {
            return Err(Refusal::MissingMaterial);
        }
        self.glanztropfen -= u.glanztropfen;
        for m in &u.materials {
            self.remove_item(&m.item, m.count)?;
        }
        self.weapons.insert(w, level + 1);
        Ok(level + 1)
    }

    // ------------------------------------------------------------ Death and saving

    /// Death (E-220, P-30): loses part of the gleam drops collected since saving; returns
    /// the loss. Elora goes back to the last save point.
    pub fn die(&mut self, content: &Content) -> u32 {
        let lost = (self.glanz_since_save * content.progression.death_loss_pct / 100)
            .min(self.glanztropfen);
        self.glanztropfen -= lost;
        self.glanz_since_save = 0;
        self.health = self.max_health(content);
        lost
    }

    /// Save point (P-31): refill health, remember the place.
    pub fn rest(&mut self, content: &Content, at: Location) {
        self.health = self.max_health(content);
        self.glanz_since_save = 0;
        self.location = at;
    }

    // ------------------------------------------------------------ World events

    /// Evaluate a simulation event for the own character `me`: experience from defeated
    /// enemies, loot (with gleam find).
    pub fn on_event(
        &mut self,
        content: &Content,
        kinds: &[elora_sim::CreatureKind],
        me: usize,
        e: &Event,
    ) -> Vec<Notice> {
        match e {
            Event::CreatureDeath {
                kind,
                killer: Some(k),
                ..
            } if *k == me => {
                let Some(k) = kinds.get(*kind) else {
                    return Vec::new();
                };
                let mut out = if k.xp > 0 {
                    self.add_xp(content, k.xp)
                } else {
                    Vec::new()
                };
                let map = self.location.map.clone();
                for o in self.on_defeat(content, &k.name, &map) {
                    if let crate::quest::Outcome::Notice(n) = o {
                        out.push(n);
                    }
                }
                out
            }
            Event::LootCollect {
                player,
                item,
                count,
                ..
            } if *player == me => {
                let mut count = *count;
                if item == GLANZTROPFEN {
                    let pct = self.stats(content).drops_pct;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let extra = (count as f32 * pct / 100.0).round().max(0.0) as u32;
                    count += extra;
                }
                let mut out = match self.add_item(content, item, count) {
                    Ok(()) => vec![Notice::Item {
                        id: item.clone(),
                        count,
                    }],
                    Err(_) => Vec::new(),
                };
                for o in self.update_quests(content) {
                    if let crate::quest::Outcome::Notice(n) = o {
                        out.push(n);
                    }
                }
                out
            }
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> (Content, SaveGame) {
        let c = Content::builtin();
        let g = SaveGame::new(&c, Location::default());
        (c, g)
    }

    #[test]
    fn levels_follow_the_curve_and_raise_health() {
        let (c, mut g) = game();
        assert_eq!(c.xp_to_next(1), 25);
        let n = g.add_xp(&c, 25);
        assert!(n.contains(&Notice::LevelUp { level: 2 }));
        assert_eq!(g.free_points(), 1);
        assert_eq!(g.max_health(&c), 10);
        g.add_xp(&c, c.xp_to_next(2));
        assert_eq!(
            (g.level, g.max_health(&c)),
            (3, 11),
            "alle 2 Stufen +1 Leben (E-241)"
        );
        g.add_xp(&c, 1_000_000);
        assert_eq!((g.level, g.xp), (30, 0));
        assert_eq!(g.max_health(&c), 24);
        assert_eq!(g.free_points(), 29);
    }

    #[test]
    fn skill_tree_needs_points_order_and_ability() {
        let (c, mut g) = game();
        assert_eq!(g.can_learn(&c, "kraft"), Err(Refusal::NoPoints));
        g.add_xp(&c, 100_000);
        assert_eq!(g.can_learn(&c, "schnelle_hand"), Err(Refusal::Locked));
        assert_eq!(
            g.can_learn(&c, "schneller_ruck"),
            Err(Refusal::NeedsAbility)
        );
        g.grant_ability(Ability::HookRuck);
        assert_eq!(g.learn(&c, "schneller_ruck"), Ok(1));
        for _ in 0..3 {
            g.learn(&c, "kraft").unwrap();
        }
        assert_eq!(g.learn(&c, "kraft"), Err(Refusal::MaxRank));
        let t = g.tuning(&c, &Tuning::default());
        assert_eq!(t.ruck_cooldown, 650);
        assert!(t.hammer_damage >= 4);
        let total: u32 = c.skills.iter().map(|n| u32::from(n.ranks)).sum();
        assert!(total > 29 + 2, "nicht alles erreichbar (E-242): {total}");
    }

    #[test]
    fn items_shop_and_equipment() {
        let (c, mut g) = game();
        assert_eq!(g.buy(&c, "lotte", "heiltrank"), Err(Refusal::TooExpensive));
        g.add_item(&c, GLANZTROPFEN, 300).unwrap();
        g.buy(&c, "lotte", "strohhut").unwrap();
        assert_eq!(g.glanztropfen, 100);
        g.equip(&c, "strohhut").unwrap();
        assert_eq!(g.max_health(&c), 11);
        assert_eq!(g.count("strohhut"), 0);
        g.unequip(&c, Slot::Hat);
        assert_eq!(g.sell(&c, "strohhut"), Ok(80), "40 % von 200");
        for _ in 0..5 {
            g.buy(&c, "lotte", "heiltrank").unwrap();
        }
        assert_eq!(g.buy(&c, "lotte", "heiltrank"), Err(Refusal::Full));
        g.health = 2;
        assert_eq!(g.use_item(&c, "heiltrank"), Ok(Effect::Heal(5)));
        assert_eq!(g.health, 7);
    }

    #[test]
    fn weapon_upgrades_cost_glanz_and_material() {
        let (c, mut g) = game();
        assert_eq!(g.upgrade(&c, Weapon::Laser), Err(Refusal::NoWeapon));
        assert_eq!(
            g.upgrade(&c, Weapon::Hammer),
            Err(Refusal::NoWeapon),
            "before Klonk"
        );
        g.weapons.insert(Weapon::Hammer, 0);
        g.add_item(&c, GLANZTROPFEN, 50).unwrap();
        assert_eq!(g.upgrade(&c, Weapon::Hammer), Err(Refusal::MissingMaterial));
        g.add_item(&c, "bernstein", 3).unwrap();
        assert_eq!(g.upgrade(&c, Weapon::Hammer), Ok(1));
        assert_eq!((g.glanztropfen, g.count("bernstein")), (10, 0));
        assert_eq!(g.tuning(&c, &Tuning::default()).hammer_damage, 4);
    }

    #[test]
    fn death_loses_a_quarter_since_last_save() {
        let (c, mut g) = game();
        g.add_item(&c, GLANZTROPFEN, 100).unwrap();
        g.rest(&c, Location::default());
        g.add_item(&c, GLANZTROPFEN, 40).unwrap();
        assert_eq!(g.die(&c), 10);
        assert_eq!(g.glanztropfen, 130);
        assert_eq!(g.die(&c), 0, "nichts mehr seit dem Speichern");
    }

    #[test]
    fn world_events_give_xp_and_loot() {
        let (c, mut g) = game();
        let kinds = &c.creatures;
        let k = kinds
            .iter()
            .position(|k| k.name == "stachelkaefer")
            .unwrap();
        let death = Event::CreatureDeath {
            id: 1,
            kind: k,
            pos: elora_sim::Vec2::ZERO,
            killer: Some(0),
        };
        g.on_event(&c, kinds, 0, &death);
        assert_eq!(g.xp, kinds[k].xp);
        assert!(
            g.on_event(&c, kinds, 1, &death).is_empty(),
            "fremder Sieg zählt nicht"
        );
        let loot = Event::LootCollect {
            player: 0,
            item: "bernstein".into(),
            count: 2,
            pos: elora_sim::Vec2::ZERO,
        };
        g.on_event(&c, kinds, 0, &loot);
        assert_eq!(g.count("bernstein"), 2);
    }
}
