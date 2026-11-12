//! The night of the Sternschlucht in the session (R2-M2.5): how dark a map is, how far Elora's
//! own light reaches, the notes she plays on note stones and the tunes among them.

use elora_sim::{Event, TICKS_PER_SECOND, World};

use super::quests::outcomes;
use super::{Session, SessionEvent};
use crate::quest::TuneDef;

/// How many of the last played notes the session keeps (M2.5.3).
pub const NOTES_KEPT: usize = 16;

/// Night tea ([`crate::data::Effect::Glow`]): Elora's light reaches this many times as far …
const GLOW_REACH: f32 = 2.0;
/// … for this long (s), fading back over the last seconds.
const GLOW_SECS: u32 = 30;
const GLOW_FADE_SECS: u32 = 3;

impl Session {
    /// Darkness of the map (night, R2-M2.5, M2.5.1): 0 = daylight, 1 = black outside the lights.
    pub fn darkness(&self) -> f32 {
        self.content.area_of(&self.map_name).map_or(0.0, |a| {
            a.map_darkness
                .get(&self.map_name)
                .copied()
                .unwrap_or(a.darkness)
                .clamp(0.0, 1.0)
        })
    }

    /// How far Elora's own light reaches, as a factor on its radius: 1 normally, more after a
    /// night tea. Drawing only – the creatures of the night do not count her light.
    pub fn light_reach(&self) -> f32 {
        let left = self.glow_until.saturating_sub(self.ticks) as f32;
        let fade = (GLOW_FADE_SECS * TICKS_PER_SECOND) as f32;
        1.0 + (GLOW_REACH - 1.0) * (left / fade).min(1.0)
    }

    /// Elora's light reaches further from now on (night tea); it lasts across map changes.
    pub(super) fn start_glow(&mut self) {
        self.glow_until = self.ticks + u64::from(GLOW_SECS * TICKS_PER_SECOND);
    }

    /// Did Elora just play `tune` on the note stones (the last notes, in this order)?
    pub fn played(&self, tune: &[u8]) -> bool {
        !tune.is_empty() && self.notes.ends_with(tune)
    }

    /// Remembers the notes Elora played in this world tick (M2.5.3) and sets off the tunes she
    /// has completed with them.
    pub(super) fn hear_notes(&mut self, world: &World, out: &mut Vec<SessionEvent>) {
        let before = self.notes.len();
        let mine = world.events.iter().filter_map(|e| match *e {
            Event::Note { player, note, .. } if player == self.player => Some(note),
            _ => None,
        });
        self.notes.extend(mine);
        if self.notes.len() == before {
            return;
        }
        let surplus = self.notes.len().saturating_sub(NOTES_KEPT);
        self.notes.drain(..surplus);
        self.play_tunes(out);
    }

    /// Tunes of this map (`[[tune]]` in `quests.toml`, M2.5.6) that Elora has just played for
    /// the first time, while their condition holds: their effects run, once per save game.
    fn play_tunes(&mut self, out: &mut Vec<SessionEvent>) {
        let played: Vec<TuneDef> = self
            .content
            .tunes
            .iter()
            .filter(|t| t.map == self.map_name && self.save.flag(&t.flag()) == 0)
            .filter(|t| t.pitches().is_some_and(|p| self.played(&p)))
            .filter(|t| {
                t.cond
                    .as_deref()
                    .is_none_or(|c| self.save.holds(&self.content, c))
            })
            .cloned()
            .collect();
        for tune in played {
            self.save.set_flag(&tune.flag(), 1);
            let outs = self.save.run(&self.content, &tune.actions);
            out.push(SessionEvent::TunePlayed { id: tune.id });
            out.extend(outcomes(outs));
        }
    }
}
