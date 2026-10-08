//! Decor by world state: faded flowers, freed springs, festival decorations (E-277, E-301).

use elora_map::Map;

use super::Session;
use crate::state::SaveGame;

/// Flag: number of freed springs (0–5).
pub const SPRINGS_FREED: &str = "springs_freed";

/// Flag of a festival in the village: decor `…-party` (garlands, lanterns) only hangs then (E-301).
pub const PARTY: &str = "party";

impl Session {
    /// Rearrange the decor according to the world state (after dialogs that set flags);
    /// `true` if it changed.
    pub fn refresh_decor(&mut self) -> bool {
        let before = (self.map.decor_front.clone(), self.map.decor_back.clone());
        self.map.decor_back.clone_from(&self.base_decor.0);
        self.map.decor_front.clone_from(&self.base_decor.1);
        adapt_decor(&mut self.map, &self.save);
        let weather = self.refresh_weather();
        weather || before.0 != self.map.decor_front || before.1 != self.map.decor_back
    }
}

/// Decor by world state: faded flowers become colourful (E-277), withered springs bloom
/// after `freed.<spring>` (`…-withered` → `…-freed`; decor names use `-`, the spring id `_`),
/// festival decorations only during the festival.
pub(super) fn adapt_decor(map: &mut Map, save: &SaveGame) {
    use elora_map::Art;
    recolor(map, save.flag(SPRINGS_FREED));
    let party = save.flag(PARTY) != 0;
    let keep =
        |d: &elora_map::Decor| party || !matches!(&d.art, Art::Builtin(n) if n.ends_with("-party"));
    map.decor_back.retain(keep);
    map.decor_front.retain(keep);
    for d in map.decor_back.iter_mut().chain(map.decor_front.iter_mut()) {
        if let Art::Builtin(name) = &d.art
            && let Some(stem) = name.strip_suffix("-withered")
            && save.flag(&format!("freed.{}", stem.replace('-', "_"))) != 0
        {
            d.art = Art::Builtin(format!("{stem}-freed"));
        }
    }
}

/// Faded decor (`…-pale`) gets part of its colour back with every freed spring
/// (`…-colourful`, E-277): each piece fixed by its position, everything after five springs.
fn recolor(map: &mut Map, freed: i64) {
    use elora_map::Art;
    let mut fix = |d: &mut elora_map::Decor| {
        if let Art::Builtin(name) = &d.art
            && let Some(stem) = name.strip_suffix("-pale")
        {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let h = (d.pos.x as i64 * 31 + d.pos.y as i64 * 17).rem_euclid(5);
            if h < freed {
                d.art = Art::Builtin(format!("{stem}-colourful"));
            }
        }
    };
    map.decor_back.iter_mut().for_each(&mut fix);
    map.decor_front.iter_mut().for_each(&mut fix);
    for b in &mut map.backgrounds {
        b.items.iter_mut().for_each(&mut fix);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_map::{Art, Decor};
    use elora_sim::Vec2;

    #[test]
    fn party_decor_and_freed_spring_follow_the_flags() {
        let mut m = Map::new("t", 4, 4);
        m.decor_back = vec![
            Decor::new(Art::Builtin("garland-party".into()), Vec2::ZERO),
            Decor::new(Art::Builtin("blossom-spring-withered".into()), Vec2::ZERO),
        ];
        let mut save = Session::new_game(crate::Content::builtin()).save;
        let mut a = m.clone();
        adapt_decor(&mut a, &save);
        assert_eq!(a.decor_back.len(), 1, "no decoration without the festival");
        assert_eq!(
            a.decor_back[0].art,
            Art::Builtin("blossom-spring-withered".into())
        );
        save.set_flag(PARTY, 1);
        save.set_flag("freed.blossom_spring", 1);
        adapt_decor(&mut m, &save);
        assert_eq!(m.decor_back.len(), 2);
        assert_eq!(
            m.decor_back[1].art,
            Art::Builtin("blossom-spring-freed".into())
        );
    }

    #[test]
    fn freed_springs_bring_colour_back() {
        let mut m = Map::new("t", 4, 4);
        m.decor_front = (0..20)
            .map(|k| {
                Decor::new(
                    Art::Builtin("flower-bed-pale".into()),
                    Vec2::new(k as f32 * 32.0, 0.0),
                )
            })
            .collect();
        m.decor_back = vec![Decor::new(Art::Builtin("house-oma".into()), Vec2::ZERO)];
        let colourful = |m: &Map| {
            m.decor_front
                .iter()
                .filter(|d| d.art == Art::Builtin("flower-bed-colourful".into()))
                .count()
        };
        let mut none = m.clone();
        recolor(&mut none, 0);
        assert_eq!(colourful(&none), 0);
        let mut some = m.clone();
        recolor(&mut some, 2);
        assert!(colourful(&some) > 0 && colourful(&some) < 20);
        recolor(&mut m, 5);
        assert_eq!(colourful(&m), 20, "all five springs");
        assert_eq!(m.decor_back[0].art, Art::Builtin("house-oma".into()));
    }
}
