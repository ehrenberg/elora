//! Map objects: doors, chests, switches, collectibles, save points and transitions.

use elora_map::ObjectKind;
use elora_map::adventure::SwitchTrigger;
use elora_sim::{Tile, Vec2, World};

use super::{
    HOOK_PICK_RANGE, Session, SessionEvent, key, outcomes, set_map_tile, tile_range,
    world_max_health,
};
use crate::data::ItemKind;
use crate::state::{Location, Notice};

impl Session {
    /// Pull hook: collectibles on the hook are collected, loot flies to Elora.
    pub(super) fn hook_pickups(
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

    /// Collect collectible `id` (once per save game).
    pub(super) fn collect(&mut self, id: &str) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        let Some(ObjectKind::Collectible { item }) =
            self.map.adventure.object(id).map(|o| o.kind.clone())
        else {
            return out;
        };
        let k = key("find", &self.map_name, id);
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

    /// Doors open as soon as their condition holds, and stay open (E-254).
    pub(super) fn update_doors(&mut self, world: &mut World) -> Vec<SessionEvent> {
        let mut changed = false;
        for o in self.map.adventure.objects.clone() {
            if let ObjectKind::Door { size, open_if } = &o.kind {
                let k = key("door", &self.map_name, &o.id);
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

    /// Flip switches of a kind near `at`.
    pub(super) fn switches_at(
        &mut self,
        at: Vec2,
        trigger: SwitchTrigger,
        range: f32,
    ) -> Vec<SessionEvent> {
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

    /// Pull switches on the path of the hook tip.
    pub(super) fn switches_along(&mut self, from: Vec2, to: Vec2, range: f32) -> Vec<SessionEvent> {
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

    pub(super) fn toggle(&mut self, id: &str) -> Vec<SessionEvent> {
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

    pub(super) fn interact(&mut self, world: &mut World, id: &str) -> Vec<SessionEvent> {
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
                self.save.set_flag(&key("chest", &self.map_name, &o.id), 1);
                let mut out = vec![SessionEvent::ChestOpened { pos: o.pos }];
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

    /// Rest at the spring stone (P-31): refill health and armour, remember the place.
    pub(super) fn rest(&mut self, world: &mut World, id: &str) {
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

    /// Is the chest already open, the switch flipped, the collectible found …?
    pub fn object_done(&self, id: &str) -> bool {
        let Some(o) = self.map.adventure.object(id) else {
            return false;
        };
        let m = &self.map_name;
        match &o.kind {
            ObjectKind::Chest { .. } => self.save.flag(&key("chest", m, id)) != 0,
            ObjectKind::Collectible { .. } => self.save.flag(&key("find", m, id)) != 0,
            ObjectKind::Switch { flag, .. } => self.save.flag(flag) != 0,
            ObjectKind::Door { .. } => self.save.flag(&key("door", m, id)) != 0,
            ObjectKind::HealPlant { .. } => self.plants_used.contains(id),
            _ => false,
        }
    }
}
