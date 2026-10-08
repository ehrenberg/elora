//! Companions (E-308): they follow Elora while their condition holds and stay in their
//! home zone.

use elora_sim::{Vec2, World};

use super::{Session, SessionEvent, inside, outcomes};

impl Session {
    /// Companions appear next to Elora as soon as their condition holds, and leave when not.
    pub(super) fn sync_followers(&mut self, world: &mut World) {
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

    /// Companions in their home zone: flag `<id>.daheim`, the companion stays there.
    pub(super) fn followers_home(&mut self, world: &mut World) -> Vec<SessionEvent> {
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
            self.save.set_flag(&format!("{id}.home"), 1);
            out.extend(outcomes(self.save.update_quests(&self.content)));
        }
        out
    }
}
