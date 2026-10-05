//! Laufendes Abenteuer auf einer Karte (A1.6): baut die Welt aus Karte und Spielstand und
//! wertet nach jedem Tick aus – Beute, Gegner, Türen, Truhen, Schalter, Heilpflanzen,
//! Sammelstücke, Zonen, Übergänge, Speicherpunkte, Tod (E-219, E-220, E-252 bis E-261).
//!
//! Reine Logik: Laden der Karten und Schreiben der Spielstände macht der Aufrufer
//! ([`SessionEvent::Travel`], [`SessionEvent::Save`]).

use std::collections::{BTreeMap, BTreeSet};

use elora_map::adventure::SwitchTrigger;
use elora_map::{Map, ObjectKind};
use elora_sim::{Event, HookState, PHYS_SIZE, TILE_SIZE, Tile, Tuning, Vec2, Weapon, World};

use crate::data::{Content, ItemKind, Text};
use crate::quest::Outcome;
use crate::script::Open;
use crate::state::{Location, Notice, SaveGame};

/// Reichweite der Aktionstaste (Einheiten).
pub const INTERACT_RANGE: f32 = 48.0;
/// Ab dieser Nähe ruft eine Figur ihren Zuruf (einmal je Besuch).
pub const BARK_RANGE: f32 = 160.0;
/// Berührung von Heilpflanzen und Sammelstücken.
const TOUCH_RANGE: f32 = PHYS_SIZE;
/// Abstand des Hooks zu einem Hook-Schalter.
const HOOK_SWITCH_RANGE: f32 = 36.0;
/// Leuchtpilze: so nah (halbe Breite, halbe Höhe um ihren Fuß) und so lange der Rausch.
const MUSHROOM_RANGE: Vec2 = Vec2::new(40.0, 40.0);
const MUSHROOM_DAZE_MS: u32 = 5500;
/// So lange muss Elora in den Pilzen stehen.
const MUSHROOM_DELAY_MS: u32 = 1200;
/// So nah muss der Hook an einem Sammelstück oder an Beute sein (Heranhooken).
const HOOK_PICK_RANGE: f32 = 28.0;
/// Leben nach „Zweite Chance“ (Knoten, P-xx).
const SECOND_CHANCE_HEALTH: i32 = 3;

/// Was der Aufrufer tun oder zeigen soll.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    Notice(Notice),
    /// Laden, Schmiede oder Baum öffnen.
    Open(Open),
    /// Gespräch beginnen (der Aufrufer führt [`crate::Conversation`]).
    Talk {
        npc: String,
        dialog: String,
    },
    /// Zuruf als Sprechblase über der Figur.
    Bark {
        npc: String,
        text: Text,
    },
    /// Truhe ist verschlossen.
    Locked {
        object: String,
    },
    /// Andere Karte betreten (der Aufrufer lädt sie und ruft [`Session::enter`]).
    Travel {
        map: String,
        spawn: String,
    },
    /// Spielstand jetzt schreiben (Speicherpunkt).
    Save,
    /// Elora ist erschöpft (E-261); `lost` Glanztropfen (E-220).
    Died {
        lost: u32,
    },
    /// Tiles der Welt haben sich geändert (Tür, Bröckelboden) – Grafik nachziehen.
    TilesChanged,
}

/// Wobei die Aktionstaste hilft (Anzeige „E …“).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prompt {
    Talk,
    Open,
    Use,
    Enter,
    Rest,
}

/// Eine Figur zum Zeichnen.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcView {
    pub id: String,
    pub character: String,
    pub pos: Vec2,
    pub facing: i8,
}

/// Das laufende Abenteuer.
#[derive(Debug, Clone)]
pub struct Session {
    pub content: Content,
    /// Arbeitsstand; geschrieben wird er nur bei [`SessionEvent::Save`] und Kartenwechsel.
    pub save: SaveGame,
    pub map_name: String,
    /// Karte mit dem aktuellen Zustand von Türen und Bröckelboden.
    pub map: Map,
    /// Hook-Spitze im letzten Tick (Heranhooken prüft die ganze Flugstrecke).
    last_hook: Option<Vec2>,
    /// So viele Ticks steht Elora schon in Leuchtpilzen (bunter Rausch nach 1,2 s).
    in_mushrooms: u32,
    /// Begleiter in der Welt: Figur → Gegner-Id (E-308).
    followers: BTreeMap<String, u32>,
    /// Deko der Karte, wie sie in der Datei steht (für [`Self::refresh_decor`]).
    base_decor: (Vec<elora_map::Decor>, Vec<elora_map::Decor>),
    pub player: usize,
    /// Gegner-Id der Welt → Objekt-Id der Karte.
    creatures: BTreeMap<u32, String>,
    plants_used: BTreeSet<String>,
    barked: BTreeSet<String>,
    inside: BTreeSet<String>,
    second_chance_used: bool,
    tau_until: u64,
    hook_armed: bool,
    ticks: u64,
    /// Aktuelle Lage der NPCs (Laufweg), Id → Position.
    npc_pos: BTreeMap<String, Vec2>,
    pending: Vec<SessionEvent>,
    pub dead: bool,
}

fn key(kind: &str, map: &str, id: &str) -> String {
    format!("{kind}:{map}:{id}")
}

fn tile_range(pos: Vec2, size: (u8, u8)) -> impl Iterator<Item = (i32, i32)> {
    #[allow(clippy::cast_possible_truncation)]
    let (x0, y0) = (
        (pos.x / TILE_SIZE as f32) as i32,
        (pos.y / TILE_SIZE as f32) as i32,
    );
    (0..i32::from(size.1))
        .flat_map(move |dy| (0..i32::from(size.0)).map(move |dx| (x0 + dx, y0 + dy)))
}

fn set_map_tile(map: &mut Map, tx: i32, ty: i32, t: Tile) {
    if let (Ok(x), Ok(y)) = (usize::try_from(tx), usize::try_from(ty))
        && x < map.width
        && y < map.height
    {
        map.tiles[y * map.width + x] = t;
    }
}

fn inside(p: Vec2, at: Vec2, size: Vec2) -> bool {
    p.x >= at.x && p.y >= at.y && p.x <= at.x + size.x && p.y <= at.y + size.y
}

impl Session {
    /// Sitzung für einen Spielstand; danach [`Self::enter`] mit `save.location`.
    pub fn new(content: Content, save: SaveGame) -> Self {
        Self {
            content,
            save,
            map_name: String::new(),
            map: Map::new("", 1, 1),
            player: 0,
            creatures: BTreeMap::new(),
            plants_used: BTreeSet::new(),
            barked: BTreeSet::new(),
            inside: BTreeSet::new(),
            second_chance_used: false,
            tau_until: 0,
            hook_armed: true,
            ticks: 0,
            npc_pos: BTreeMap::new(),
            base_decor: (Vec::new(), Vec::new()),
            followers: BTreeMap::new(),
            in_mushrooms: 0,
            last_hook: None,
            pending: Vec::new(),
            dead: false,
        }
    }

    /// Neues Spiel am Start aus `progression.toml`.
    pub fn new_game(content: Content) -> Self {
        let p = &content.progression;
        let start = Location {
            map: p.start_map.clone(),
            spawn: p.start_spawn.clone(),
        };
        let save = SaveGame::new(&content, start);
        Self::new(content, save)
    }

    /// Karte `name` betreten, Elora an Objekt `spawn` (Eingang oder Speicherpunkt). Liefert
    /// die Welt; Gegner und Heilpflanzen sind frisch (E-235, E-258), Bosse bleiben besiegt.
    pub fn enter(&mut self, name: &str, mut map: Map, spawn: &str, base: &Tuning) -> World {
        let c = &self.content;
        // Weltzustand: Bröckelboden und Türen
        for &(tx, ty) in self.save.broken.get(name).into_iter().flatten() {
            set_map_tile(&mut map, tx, ty, Tile::Air);
        }
        for o in &map.adventure.objects.clone() {
            if let ObjectKind::Door { size, .. } = &o.kind {
                let open = self.save.flag(&key("tuer", name, &o.id)) != 0;
                for (tx, ty) in tile_range(o.pos, *size) {
                    set_map_tile(
                        &mut map,
                        tx,
                        ty,
                        if open { Tile::Air } else { Tile::Unhookable },
                    );
                }
            }
        }
        // das Fest dauert, bis Elora das Dorf verlässt (E-301)
        let hub = c.areas.first().map(|a| a.id.as_str());
        if self.save.flag(PARTY) != 0 && c.area_of(name).map(|a| a.id.as_str()) != hub {
            self.save.set_flag(PARTY, 0);
        }
        self.base_decor = (map.decor_back.clone(), map.decor_front.clone());
        adapt_decor(&mut map, &self.save);
        let tuning = self.save.tuning(c, base);
        let mut world = map.world(tuning);
        world.creature_kinds.clone_from(&c.creatures);
        world.adventure = true;

        let objects = &map.adventure.objects;
        let pos = objects
            .iter()
            .find(|o| o.id == spawn)
            .or_else(|| objects.iter().find(|o| matches!(o.kind, ObjectKind::Spawn)))
            .map(|o| o.pos)
            .or_else(|| world.spawn_points.first().copied())
            .unwrap_or(Vec2::new(64.0, 64.0));
        let player = world.join();
        world.set_abilities(player, self.save.abilities());
        if let Some(p) = world.players[player].as_mut() {
            p.respawn_disabled = true;
        }
        world.spawn_character(player, pos);
        let stats = self.save.stats(c);
        let max_ammo = world.tuning.max_ammo;
        if let Some(ch) = world.character_mut(player) {
            ch.health = self.save.health.clamp(1, world_max_health(&self.save, c));
            ch.armor = stats.armor.max(0);
            for &w in self.save.weapons.keys() {
                if w != Weapon::Hammer {
                    let ammo = self.save.ammo.get(&w).copied().unwrap_or(max_ammo);
                    ch.arsenal.give(w, ammo, max_ammo);
                }
            }
        }

        self.creatures.clear();
        for o in objects {
            if let ObjectKind::Creature { kind, persistent } = &o.kind {
                if *persistent && self.save.defeated.contains(&key("gegner", name, &o.id)) {
                    continue;
                }
                if let Some(k) = world.creature_kind(kind)
                    && let Some(id) = world.add_creature(k, o.pos)
                {
                    self.creatures.insert(id, o.id.clone());
                }
            }
        }
        world.events.clear();
        self.followers.clear();

        name.clone_into(&mut self.map_name);
        self.inside = Self::areas_at(&map, pos);
        self.map = map;
        self.player = player;
        self.plants_used.clear();
        self.barked.clear();
        self.second_chance_used = false;
        self.dead = false;
        self.save.location = Location {
            map: name.to_owned(),
            spawn: spawn.to_owned(),
        };
        // für die Weltkarte (E-264)
        self.save.set_flag(&format!("besucht:{name}"), 1);
        let reached = self.save.on_reach(&self.content, name, None);
        self.push_outcomes(reached);
        self.sync_followers(&mut world);
        world
    }

    fn push_outcomes(&mut self, outs: Vec<Outcome>) {
        for o in outs {
            self.pending.push(match o {
                Outcome::Notice(n) => SessionEvent::Notice(n),
                Outcome::Open(o) => SessionEvent::Open(o),
            });
        }
    }

    /// Ids der Zonen und Übergänge, in denen `p` liegt.
    fn areas_at(map: &Map, p: Vec2) -> BTreeSet<String> {
        map.adventure
            .objects
            .iter()
            .filter(|o| matches!(o.kind, ObjectKind::Zone { .. } | ObjectKind::Exit { .. }))
            .filter(|o| o.kind.area().is_some_and(|s| inside(p, o.pos, s)))
            .map(|o| o.id.clone())
            .collect()
    }

    /// Nach jedem Tick der Welt; `interact` = Aktionstaste in diesem Tick gedrückt.
    #[allow(clippy::too_many_lines)]
    pub fn tick(&mut self, world: &mut World, interact: bool) -> Vec<SessionEvent> {
        let mut out = std::mem::take(&mut self.pending);
        self.ticks += 1;
        if self
            .ticks
            .is_multiple_of(u64::from(elora_sim::TICKS_PER_SECOND))
        {
            self.save.play_time_secs += 1;
        }
        let me = self.player;
        let events = world.events.clone();
        for e in &events {
            match e {
                Event::CreatureDeath { id, kind, .. } => {
                    if let Some(obj) = self.creatures.remove(id) {
                        let persistent = self.map.adventure.object(&obj).is_some_and(|o| {
                            matches!(
                                o.kind,
                                ObjectKind::Creature {
                                    persistent: true,
                                    ..
                                }
                            )
                        });
                        let boss = world.creature_kinds.get(*kind).is_some_and(|k| k.boss);
                        if persistent || boss {
                            self.save
                                .defeated
                                .insert(key("gegner", &self.map_name, &obj));
                        }
                    }
                    // Hüter besiegt: Merker für Türen, Gespräche und Aufgaben (R2-M2.1)
                    if let Some(k) = world.creature_kinds.get(*kind).filter(|k| k.boss) {
                        self.save.set_flag(&format!("besiegt.{}", k.name), 1);
                        out.extend(outcomes(self.save.update_quests(&self.content)));
                    }
                    let n = self
                        .save
                        .on_event(&self.content, &world.creature_kinds, me, e);
                    out.extend(n.into_iter().map(SessionEvent::Notice));
                }
                Event::LootCollect { .. } => {
                    let n = self
                        .save
                        .on_event(&self.content, &world.creature_kinds, me, e);
                    out.extend(n.into_iter().map(SessionEvent::Notice));
                }
                Event::TileBroken { tx, ty } => {
                    self.save
                        .broken
                        .entry(self.map_name.clone())
                        .or_default()
                        .insert((*tx, *ty));
                    set_map_tile(&mut self.map, *tx, *ty, Tile::Air);
                    out.push(SessionEvent::TilesChanged);
                }
                // Wurzelwände: nur zeitweise, nicht im Spielstand
                Event::TileSet { tx, ty, tile } => {
                    set_map_tile(&mut self.map, *tx, *ty, *tile);
                    out.push(SessionEvent::TilesChanged);
                }
                Event::HammerHit { owner, pos } if *owner == me => {
                    out.extend(self.switches_at(*pos, SwitchTrigger::Hammer, 32.0));
                }
                Event::Death { player, pos, .. } if *player == me => {
                    if self.save.stats(&self.content).second_chance > 0 && !self.second_chance_used
                    {
                        self.second_chance_used = true;
                        world.spawn_character(me, *pos);
                        if let Some(ch) = world.character_mut(me) {
                            ch.health = SECOND_CHANCE_HEALTH;
                        }
                    } else {
                        let lost = self.save.die(&self.content);
                        self.dead = true;
                        out.push(SessionEvent::Died { lost });
                        return out;
                    }
                }
                _ => {}
            }
        }

        let Some(ch) = world.character(me) else {
            return out;
        };
        let pos = ch.core.pos;
        let (health, hook_state, hook_pos) = (ch.health, ch.core.hook_state, ch.core.hook_pos);
        self.save.health = health;
        for w in [Weapon::Grenade, Weapon::Laser] {
            if let Some(a) = ch.arsenal.slot(w).ammo
                && ch.arsenal.has(w)
            {
                self.save.ammo.insert(w, a);
            }
        }
        if world.tick < self.tau_until
            && let Some(ch) = world.character_mut(me)
        {
            ch.core.ruck_cooldown = 0;
        }

        // Heranhooken (R2-M2.2): Zugschalter einmal je Schuss, Sammelstücke und Beute am Hook
        // Fähigkeiten der Figur (Spielstand, im Testspiel auch über F1 eingeschaltet)
        let pull = world
            .character(me)
            .is_some_and(|c| c.core.abilities.has(elora_sim::Ability::Pull));
        // Strecke der Hook-Spitze seit dem letzten Tick (sie fliegt 80 Einheiten je Tick)
        let hooking = matches!(hook_state, HookState::Flying | HookState::Grabbed);
        let from = self.last_hook.filter(|_| hooking).unwrap_or(hook_pos);
        self.last_hook = hooking.then_some(hook_pos);
        if pull && hooking {
            out.extend(self.hook_pickups(world, from, hook_pos, pos));
        }
        if hooking && pull {
            if self.hook_armed {
                let hits = self.switches_along(from, hook_pos, HOOK_SWITCH_RANGE);
                if !hits.is_empty() {
                    self.hook_armed = false;
                }
                out.extend(hits);
            }
        } else {
            self.hook_armed = true;
        }

        self.npc_pos = self
            .npcs(world)
            .into_iter()
            .map(|n| (n.id, n.pos))
            .collect();
        out.extend(self.followers_home(world));
        self.mushroom_daze(world, pos);
        out.extend(self.touch(world, pos));
        out.extend(self.areas(pos));
        out.extend(self.barks(pos));
        if interact && let Some((id, _)) = self.interactable(pos) {
            out.extend(self.interact(world, &id));
        }
        // nach Schaltern und Truhen, damit Türen im selben Tick aufgehen
        out.extend(self.update_doors(world));
        out
    }

    /// Heranhooken: Sammelstücke am Hook werden eingesammelt, Beute fliegt zu Elora.
    fn hook_pickups(
        &mut self,
        world: &mut World,
        from: Vec2,
        hook: Vec2,
        elora: Vec2,
    ) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        let near_path =
            |p: Vec2| Vec2::closest_point_on_segment(from, hook, p).distance(p) < HOOK_PICK_RANGE;
        let near: Vec<String> = self
            .map
            .adventure
            .objects
            .iter()
            .filter(|o| matches!(o.kind, ObjectKind::Collectible { .. }) && near_path(o.pos))
            .map(|o| o.id.clone())
            .collect();
        for id in near {
            out.extend(self.collect(&id));
        }
        for l in &mut world.loot {
            if near_path(l.pos) {
                l.pos = elora;
                l.vel = Vec2::ZERO;
            }
        }
        out
    }

    /// Sammelstück `id` einsammeln (einmalig je Spielstand).
    fn collect(&mut self, id: &str) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        let Some(ObjectKind::Collectible { item }) =
            self.map.adventure.object(id).map(|o| o.kind.clone())
        else {
            return out;
        };
        let k = key("fund", &self.map_name, id);
        if self.save.flag(&k) == 0 && self.save.add_item(&self.content, &item, 1).is_ok() {
            self.save.set_flag(&k, 1);
            out.push(SessionEvent::Notice(Notice::Item {
                id: item.clone(),
                count: 1,
            }));
            let up = self.save.update_quests(&self.content);
            out.extend(outcomes(up));
        }
        out
    }

    /// Heilpflanzen und Sammelstücke bei Berührung.
    fn touch(&mut self, world: &mut World, pos: Vec2) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        let heal_bonus = self.save.stats(&self.content).heal_bonus;
        let max = world_max_health(&self.save, &self.content);
        for o in self.map.adventure.objects.clone() {
            if o.pos.distance(pos) > TOUCH_RANGE {
                continue;
            }
            match &o.kind {
                ObjectKind::HealPlant { heal } if !self.plants_used.contains(&o.id) => {
                    if let Some(ch) = world.character_mut(self.player)
                        && ch.health < max
                    {
                        ch.health = (ch.health + heal + heal_bonus).min(max);
                        self.save.health = ch.health;
                        self.plants_used.insert(o.id.clone());
                    }
                }
                ObjectKind::Collectible { .. } => out.extend(self.collect(&o.id)),
                _ => {}
            }
        }
        out
    }

    /// Zonen („Ort erreichen“) und Übergänge beim Hineinlaufen.
    fn areas(&mut self, pos: Vec2) -> Vec<SessionEvent> {
        let now = Self::areas_at(&self.map, pos);
        let mut out = Vec::new();
        for id in now.difference(&self.inside) {
            let Some(o) = self.map.adventure.object(id) else {
                continue;
            };
            match &o.kind {
                ObjectKind::Zone { .. } => {
                    let r = self.save.on_reach(&self.content, &self.map_name, Some(id));
                    out.extend(outcomes(r));
                }
                ObjectKind::Exit {
                    map,
                    spawn,
                    on_touch: true,
                    ..
                } => out.push(SessionEvent::Travel {
                    map: map.clone(),
                    spawn: spawn.clone(),
                }),
                _ => {}
            }
        }
        self.inside = now;
        out
    }

    /// Türen öffnen sich, sobald ihre Bedingung gilt, und bleiben offen (E-254).
    fn update_doors(&mut self, world: &mut World) -> Vec<SessionEvent> {
        let mut changed = false;
        for o in self.map.adventure.objects.clone() {
            if let ObjectKind::Door { size, open_if } = &o.kind {
                let k = key("tuer", &self.map_name, &o.id);
                if self.save.flag(&k) == 0 && self.save.holds(&self.content, open_if) {
                    self.save.set_flag(&k, 1);
                    for (tx, ty) in tile_range(o.pos, *size) {
                        world.collision.set_tile(tx, ty, Tile::Air);
                        set_map_tile(&mut self.map, tx, ty, Tile::Air);
                    }
                    changed = true;
                }
            }
        }
        if changed {
            vec![SessionEvent::TilesChanged]
        } else {
            Vec::new()
        }
    }

    fn barks(&mut self, pos: Vec2) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        for o in &self.map.adventure.objects {
            if let ObjectKind::Npc {
                dialog, character, ..
            } = &o.kind
                && self.present(character)
                && o.pos.distance(pos) < BARK_RANGE
                && !self.barked.contains(&o.id)
            {
                self.barked.insert(o.id.clone());
                if let Some(t) = crate::dialog::bark(&self.content, &self.save, dialog) {
                    out.push(SessionEvent::Bark {
                        npc: o.id.clone(),
                        text: t.clone(),
                    });
                }
            }
        }
        out
    }

    /// Schalter einer Art nahe `at` umlegen.
    fn switches_at(&mut self, at: Vec2, trigger: SwitchTrigger, range: f32) -> Vec<SessionEvent> {
        let ids: Vec<String> = self
            .map
            .adventure
            .objects
            .iter()
            .filter(|o| matches!(&o.kind, ObjectKind::Switch { trigger: t, .. } if *t == trigger))
            .filter(|o| o.pos.distance(at) < range)
            .map(|o| o.id.clone())
            .collect();
        ids.iter().flat_map(|id| self.toggle(id)).collect()
    }

    /// Zugschalter auf der Strecke der Hook-Spitze.
    fn switches_along(&mut self, from: Vec2, to: Vec2, range: f32) -> Vec<SessionEvent> {
        let ids: Vec<String> = self
            .map
            .adventure
            .objects
            .iter()
            .filter(|o| {
                matches!(&o.kind, ObjectKind::Switch { trigger, .. } if *trigger == SwitchTrigger::Hook)
            })
            .filter(|o| Vec2::closest_point_on_segment(from, to, o.pos).distance(o.pos) < range)
            .map(|o| o.id.clone())
            .collect();
        ids.iter().flat_map(|id| self.toggle(id)).collect()
    }

    fn toggle(&mut self, id: &str) -> Vec<SessionEvent> {
        let Some(ObjectKind::Switch { flag, once, .. }) =
            self.map.adventure.object(id).map(|o| o.kind.clone())
        else {
            return Vec::new();
        };
        let v = self.save.flag(&flag);
        if once && v != 0 {
            return Vec::new();
        }
        self.save.set_flag(&flag, i64::from(v == 0));
        outcomes(self.save.update_quests(&self.content))
    }

    /// Begleiter erscheinen neben Elora, sobald ihre Bedingung gilt, und gehen, wenn nicht.
    fn sync_followers(&mut self, world: &mut World) {
        let Some(elora) = world.character(self.player).map(|c| c.core.pos) else {
            return;
        };
        for (id, ch) in &self.content.characters {
            let (Some(kind), Some(cond)) = (&ch.follower, &ch.follow_if) else {
                continue;
            };
            let wanted = self.save.holds(&self.content, cond);
            match (wanted, self.followers.get(id).copied()) {
                (true, None) => {
                    if let Some(k) = world.creature_kind(kind)
                        && let Some(cid) = world.add_creature(k, elora + Vec2::new(-40.0, -8.0))
                    {
                        self.followers.insert(id.clone(), cid);
                    }
                }
                (false, Some(cid)) => {
                    world.creatures.retain(|c| c.id != cid);
                    self.followers.remove(id);
                }
                _ => {}
            }
        }
    }

    /// Wer 1,2 s in Leuchtpilzen steht, bekommt den bunten Rausch (E-311).
    fn mushroom_daze(&mut self, world: &mut World, pos: Vec2) {
        let inside = self
            .map
            .decor_front
            .iter()
            .chain(&self.map.decor_back)
            .any(|d| {
                d.art == elora_map::Art::Builtin("leuchtpilze".into())
                    && (d.pos.x - pos.x).abs() < MUSHROOM_RANGE.x
                    && (d.pos.y - pos.y).abs() < MUSHROOM_RANGE.y
            });
        self.in_mushrooms = if inside { self.in_mushrooms + 1 } else { 0 };
        if self.in_mushrooms >= elora_sim::tuning::ms_to_ticks(MUSHROOM_DELAY_MS) {
            self.in_mushrooms = 0;
            let ticks = elora_sim::tuning::ms_to_ticks(MUSHROOM_DAZE_MS);
            if let Some(ch) = world.character_mut(self.player)
                && ch.core.dazed == 0
            {
                ch.core.dazed = ticks;
            }
        }
    }

    /// Begleiter in ihrer Heimat-Zone: Merker `<id>.daheim`, der Begleiter bleibt dort.
    fn followers_home(&mut self, world: &mut World) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        let arrived: Vec<String> = self
            .followers
            .iter()
            .filter_map(|(id, cid)| {
                let zone = self.content.characters.get(id)?.home_zone.as_ref()?;
                let z = self.map.adventure.object(zone)?;
                let size = z.kind.area()?;
                let c = world.creatures.iter().find(|c| c.id == *cid)?;
                inside(c.pos, z.pos, size).then(|| id.clone())
            })
            .collect();
        for id in arrived {
            if let Some(cid) = self.followers.remove(&id) {
                world.creatures.retain(|c| c.id != cid);
            }
            self.save.set_flag(&format!("{id}.daheim"), 1);
            out.extend(outcomes(self.save.update_quests(&self.content)));
        }
        out
    }

    /// Fähigkeiten und Waffen aus dem Spielstand in die laufende Welt übernehmen (nach
    /// Gesprächen, die etwas freischalten: Hook-Ruck bei Tüftel, Granatwerfer bei Klonk),
    /// Begleiter erscheinen lassen.
    pub fn sync_world(&mut self, world: &mut World) {
        self.sync_followers(world);
        // dazu, was schon in der Welt gilt (im Testspiel über F1 eingeschaltet)
        let current = world
            .player(self.player)
            .map_or(elora_sim::Abilities::NONE, |p| p.abilities);
        world.set_abilities(self.player, self.save.abilities().union(current));
        let max_ammo = world.tuning.max_ammo;
        if let Some(ch) = world.character_mut(self.player) {
            for &w in self.save.weapons.keys() {
                if w != Weapon::Hammer && !ch.arsenal.has(w) {
                    let ammo = self.save.ammo.get(&w).copied().unwrap_or(max_ammo);
                    ch.arsenal.give(w, ammo, max_ammo);
                }
            }
        }
    }

    /// Deko neu nach dem Weltzustand richten (nach Gesprächen, die Merker setzen);
    /// `true`, wenn sie sich geändert hat.
    pub fn refresh_decor(&mut self) -> bool {
        let before = (self.map.decor_front.clone(), self.map.decor_back.clone());
        self.map.decor_back.clone_from(&self.base_decor.0);
        self.map.decor_front.clone_from(&self.base_decor.1);
        adapt_decor(&mut self.map, &self.save);
        before.0 != self.map.decor_front || before.1 != self.map.decor_back
    }

    /// Ist die Figur gerade zu sehen (`show_if` in `characters.toml`)?
    pub fn present(&self, character: &str) -> bool {
        self.content
            .characters
            .get(character)
            .and_then(|c| c.show_if.as_deref())
            .is_none_or(|s| self.save.holds(&self.content, s))
    }

    /// Nächstes Objekt in Reichweite der Aktionstaste mit Hinweis.
    pub fn interactable(&self, pos: Vec2) -> Option<(String, Prompt)> {
        self.map
            .adventure
            .objects
            .iter()
            .filter_map(|o| {
                let prompt = match &o.kind {
                    ObjectKind::Npc { character, .. } if self.present(character) => Prompt::Talk,
                    ObjectKind::Chest { .. }
                        if self.save.flag(&key("truhe", &self.map_name, &o.id)) == 0 =>
                    {
                        Prompt::Open
                    }
                    ObjectKind::Switch {
                        trigger: SwitchTrigger::Interact,
                        once,
                        flag,
                    } if !(*once && self.save.flag(flag) != 0) => Prompt::Use,
                    ObjectKind::SavePoint => Prompt::Rest,
                    ObjectKind::Exit {
                        on_touch: false, ..
                    } => Prompt::Enter,
                    _ => return None,
                };
                let center = match o.kind.area() {
                    Some(s) if !matches!(o.kind, ObjectKind::Door { .. }) => o.pos + s * 0.5,
                    _ => self.npc_pos.get(&o.id).copied().unwrap_or(o.pos),
                };
                let d = center.distance(pos);
                let reach = o
                    .kind
                    .area()
                    .map_or(INTERACT_RANGE, |s| INTERACT_RANGE + s.x.max(s.y) / 2.0);
                (d < reach).then(|| (o.id.clone(), prompt, d))
            })
            .min_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(id, p, _)| (id, p))
    }

    fn interact(&mut self, world: &mut World, id: &str) -> Vec<SessionEvent> {
        let Some(o) = self.map.adventure.object(id).cloned() else {
            return Vec::new();
        };
        match &o.kind {
            ObjectKind::Npc { dialog, .. } => vec![SessionEvent::Talk {
                npc: o.id.clone(),
                dialog: dialog.clone(),
            }],
            ObjectKind::Chest { contents, lock } => {
                if !lock.is_empty() && !self.save.holds(&self.content, lock) {
                    return vec![SessionEvent::Locked { object: o.id }];
                }
                self.save.set_flag(&key("truhe", &self.map_name, &o.id), 1);
                let mut out = Vec::new();
                let max_ammo = world.tuning.max_ammo;
                for (item, n) in contents {
                    if let Some(ItemKind::Ammo { weapon }) =
                        self.content.item(item).map(|i| &i.kind)
                    {
                        if let Some(ch) = world.character_mut(self.player)
                            && ch.arsenal.has(*weapon)
                        {
                            ch.arsenal.give(*weapon, max_ammo, max_ammo);
                        }
                    } else if self.save.add_item(&self.content, item, *n).is_err() {
                        continue;
                    }
                    out.push(SessionEvent::Notice(Notice::Item {
                        id: item.clone(),
                        count: *n,
                    }));
                }
                out.extend(outcomes(self.save.update_quests(&self.content)));
                out
            }
            ObjectKind::Switch { .. } => self.toggle(id),
            ObjectKind::SavePoint => {
                self.rest(world, &o.id);
                vec![SessionEvent::Save]
            }
            ObjectKind::Exit { map, spawn, .. } => vec![SessionEvent::Travel {
                map: map.clone(),
                spawn: spawn.clone(),
            }],
            _ => Vec::new(),
        }
    }

    /// Am Quellstein rasten (P-31): Leben und Rüstung auffüllen, Ort merken.
    fn rest(&mut self, world: &mut World, id: &str) {
        let at = Location {
            map: self.map_name.clone(),
            spawn: id.to_owned(),
        };
        self.save.rest(&self.content, at);
        let max = world_max_health(&self.save, &self.content);
        let armor = self.save.stats(&self.content).armor.max(0);
        if let Some(ch) = world.character_mut(self.player) {
            ch.health = max;
            ch.armor = armor;
        }
        self.save.health = max;
    }

    /// Verbrauchsgegenstand benutzen (Inventar, A1.7): Heilen in der Welt, Tautrank.
    ///
    /// # Errors
    /// Siehe [`SaveGame::use_item`].
    pub fn use_item(&mut self, world: &mut World, id: &str) -> Result<(), crate::Refusal> {
        let effect = self.save.use_item(&self.content, id)?;
        match effect {
            crate::data::Effect::Heal(_) => {
                if let Some(ch) = world.character_mut(self.player) {
                    ch.health = self.save.health;
                }
            }
            crate::data::Effect::Tau(secs) => {
                self.tau_until = world.tick + u64::from(secs * elora_sim::TICKS_PER_SECOND);
            }
        }
        Ok(())
    }

    /// Figuren mit Blickrichtung zu Elora und kurzem Laufweg (E-257).
    pub fn npcs(&self, world: &World) -> Vec<NpcView> {
        let elora = world.character(self.player).map(|c| c.core.pos);
        #[allow(clippy::cast_precision_loss)]
        let t = world.tick as f32 / elora_sim::TICKS_PER_SECOND as f32;
        self.map
            .adventure
            .objects
            .iter()
            .filter_map(|o| match &o.kind {
                ObjectKind::Npc {
                    character,
                    facing,
                    walk,
                    ..
                } if self.present(character) => {
                    let offset = if *walk > 0.0 {
                        (t * 0.4).sin() * walk
                    } else {
                        0.0
                    };
                    let pos = o.pos + Vec2::new(offset, 0.0);
                    let fixed = self
                        .content
                        .characters
                        .get(character)
                        .is_some_and(|c| c.fixed);
                    let facing = match elora {
                        Some(e) if !fixed && e.distance(pos) < BARK_RANGE => {
                            if e.x < pos.x {
                                -1
                            } else {
                                1
                            }
                        }
                        _ => *facing,
                    };
                    Some(NpcView {
                        id: o.id.clone(),
                        character: character.clone(),
                        pos,
                        facing,
                    })
                }
                _ => None,
            })
            .collect()
    }

    /// Wo der Hinweis zur Aktionstaste steht: über NPCs (auf ihrem Laufweg) und Objekten,
    /// bei Bereichen über der Mitte der Oberkante.
    pub fn anchor(&self, id: &str) -> Option<Vec2> {
        let o = self.map.adventure.object(id)?;
        Some(match o.kind.area() {
            Some(size) => o.pos + Vec2::new(size.x / 2.0, 0.0),
            None => self.npc_pos.get(id).copied().unwrap_or(o.pos),
        })
    }

    /// Ist die Truhe schon offen, der Schalter umgelegt, das Sammelstück gefunden …?
    pub fn object_done(&self, id: &str) -> bool {
        let Some(o) = self.map.adventure.object(id) else {
            return false;
        };
        let m = &self.map_name;
        match &o.kind {
            ObjectKind::Chest { .. } => self.save.flag(&key("truhe", m, id)) != 0,
            ObjectKind::Collectible { .. } => self.save.flag(&key("fund", m, id)) != 0,
            ObjectKind::Switch { flag, .. } => self.save.flag(flag) != 0,
            ObjectKind::Door { .. } => self.save.flag(&key("tuer", m, id)) != 0,
            ObjectKind::HealPlant { .. } => self.plants_used.contains(id),
            _ => false,
        }
    }
}

/// Merker: Anzahl befreiter Quellen (0–5).
pub const SPRINGS_FREED: &str = "quellen_befreit";

/// Verblasste Deko (`…-blass`) bekommt mit jeder befreiten Quelle zum Teil ihre Farbe zurück
/// (`…-bunt`, E-277): je Stück fest über seine Lage verteilt, nach fünf Quellen alles.
/// Merker eines Fests im Dorf: Deko `…-fest` (Girlanden, Laternen) hängt nur dann (E-301).
pub const PARTY: &str = "fest";

/// Deko nach dem Weltzustand: verblasste Blumen werden bunt (E-277), verdorrte Quellen
/// blühen nach `befreit.<name>` (`…-verdorrt` → `…-befreit`), Festschmuck nur beim Fest.
fn adapt_decor(map: &mut Map, save: &SaveGame) {
    use elora_map::Art;
    recolor(map, save.flag(SPRINGS_FREED));
    let party = save.flag(PARTY) != 0;
    let keep =
        |d: &elora_map::Decor| party || !matches!(&d.art, Art::Builtin(n) if n.ends_with("-fest"));
    map.decor_back.retain(keep);
    map.decor_front.retain(keep);
    for d in map.decor_back.iter_mut().chain(map.decor_front.iter_mut()) {
        if let Art::Builtin(name) = &d.art
            && let Some(stem) = name.strip_suffix("-verdorrt")
            && save.flag(&format!("befreit.{stem}")) != 0
        {
            d.art = Art::Builtin(format!("{stem}-befreit"));
        }
    }
}

fn recolor(map: &mut Map, freed: i64) {
    use elora_map::Art;
    let mut fix = |d: &mut elora_map::Decor| {
        if let Art::Builtin(name) = &d.art
            && let Some(stem) = name.strip_suffix("-blass")
        {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let h = (d.pos.x as i64 * 31 + d.pos.y as i64 * 17).rem_euclid(5);
            if h < freed {
                d.art = Art::Builtin(format!("{stem}-bunt"));
            }
        }
    };
    map.decor_back.iter_mut().for_each(&mut fix);
    map.decor_front.iter_mut().for_each(&mut fix);
    for b in &mut map.backgrounds {
        b.items.iter_mut().for_each(&mut fix);
    }
}

fn outcomes(v: Vec<Outcome>) -> Vec<SessionEvent> {
    v.into_iter()
        .map(|o| match o {
            Outcome::Notice(n) => SessionEvent::Notice(n),
            Outcome::Open(o) => SessionEvent::Open(o),
        })
        .collect()
}

fn world_max_health(save: &SaveGame, c: &Content) -> i32 {
    save.max_health(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_map::{Art, Decor};

    #[test]
    fn characters_appear_when_their_condition_holds() {
        let mut s = Session::new_game(crate::Content::builtin());
        assert!(s.present("oma"));
        assert!(!s.present("hummel"), "erst nach dem Kampf");
        s.save.set_flag("besiegt.brummbaer", 1);
        assert!(s.present("hummel"));
    }

    #[test]
    fn party_decor_and_freed_spring_follow_the_flags() {
        let mut m = Map::new("t", 4, 4);
        m.decor_back = vec![
            Decor::new(Art::Builtin("girlande-fest".into()), Vec2::ZERO),
            Decor::new(Art::Builtin("bluetenquelle-verdorrt".into()), Vec2::ZERO),
        ];
        let mut save = Session::new_game(crate::Content::builtin()).save;
        let mut a = m.clone();
        adapt_decor(&mut a, &save);
        assert_eq!(a.decor_back.len(), 1, "ohne Fest kein Schmuck");
        assert_eq!(
            a.decor_back[0].art,
            Art::Builtin("bluetenquelle-verdorrt".into())
        );
        save.set_flag(PARTY, 1);
        save.set_flag("befreit.bluetenquelle", 1);
        adapt_decor(&mut m, &save);
        assert_eq!(m.decor_back.len(), 2);
        assert_eq!(
            m.decor_back[1].art,
            Art::Builtin("bluetenquelle-befreit".into())
        );
    }

    #[test]
    fn freed_springs_bring_colour_back() {
        let mut m = Map::new("t", 4, 4);
        m.decor_front = (0..20)
            .map(|k| {
                Decor::new(
                    Art::Builtin("beet-blass".into()),
                    Vec2::new(k as f32 * 32.0, 0.0),
                )
            })
            .collect();
        m.decor_back = vec![Decor::new(Art::Builtin("haus-oma".into()), Vec2::ZERO)];
        let bunt = |m: &Map| {
            m.decor_front
                .iter()
                .filter(|d| d.art == Art::Builtin("beet-bunt".into()))
                .count()
        };
        let mut none = m.clone();
        recolor(&mut none, 0);
        assert_eq!(bunt(&none), 0);
        let mut some = m.clone();
        recolor(&mut some, 2);
        assert!(bunt(&some) > 0 && bunt(&some) < 20);
        recolor(&mut m, 5);
        assert_eq!(bunt(&m), 20, "alle fünf Quellen");
        assert_eq!(m.decor_back[0].art, Art::Builtin("haus-oma".into()));
    }
}
