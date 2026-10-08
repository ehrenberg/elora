//! Sound glue: music of the region and the sounds of a frame.

use elora_client::scene::Scene;
use elora_sim::Event;

use crate::App;

/// Music while a guardian is awake (`assets/music/boss.ogg`).
const BOSS_MUSIC: &str = "boss";

impl App {
    /// Sounds of the frame: events, characters, landings, new emotes and chat lines.
    pub(crate) fn play_sounds(&mut self, scene: &Scene, events: &[Event]) {
        // music of the region in the adventure (E-285), otherwise silent
        let track = self.adventure.as_ref().and_then(|a| {
            let s = &a.session;
            // awake guardian: battle music (R2-M2.1)
            let world = &self.sandbox.world;
            let boss = world.creatures.iter().any(|c| {
                world.creature_kinds.get(c.kind).is_some_and(|k| k.boss)
                    && c.mode != elora_sim::creature::diver::SLEEP
            });
            let area = s.content.area_of(&s.map_name)?;
            if boss {
                return Some(
                    area.boss_music
                        .clone()
                        .unwrap_or_else(|| BOSS_MUSIC.to_owned()),
                );
            }
            let party = s.save.flag(elora_adventure::session::PARTY) != 0;
            party
                .then(|| area.party_music.clone())
                .flatten()
                .or_else(|| area.music.clone())
        });
        self.sounds.music(track.as_deref());
        let mut extra: Vec<elora_audio::Cue> = self
            .emotes
            .take_new()
            .into_iter()
            .filter_map(|slot| scene.chars.iter().find(|c| c.slot == slot))
            .map(|c| elora_audio::Cue::at(elora_audio::Sound::Emote, c.pos()))
            .collect();
        let newest_chat = match &self.online {
            Some(o) => o.client.chat.back().map(|c| c.at),
            None => self.sandbox.notices.back().map(|c| c.at),
        };
        if newest_chat.is_some() && newest_chat != self.chat_heard {
            // don't voice the first line after the start (e.g. greeting on connect)
            if self.chat_heard.is_some() {
                extra.push(elora_audio::Cue::global(elora_audio::Sound::Chat));
            }
            self.chat_heard = newest_chat;
        }
        let dazed = scene.local().is_some_and(|c| c.ch.core.dazed > 0);
        if dazed && !self.was_dazed {
            extra.push(elora_audio::Cue::global(elora_audio::Sound::Daze));
        }
        self.was_dazed = dazed;
        extra.extend(self.menu.ui.sounds.drain(..).map(elora_audio::Cue::global));
        extra.append(&mut self.ui_cues);
        self.sounds
            .update(scene, events, self.figures.landings(), &extra, scene.camera);
    }
}
