//! Weather of the map (R2-W1): rolled from the area's lists, brighter once the spring is
//! freed, and how the simulation feels it.

use super::{SPRINGS_FREED, Session};

impl Session {
    /// Is the area still gloomy? As long as its spring is silent; without its own spring until
    /// `clears_after_springs` springs are freed (E-331).
    pub(super) fn gloomy(&self, area: &crate::data::Area) -> bool {
        match area.clears_after_springs {
            Some(n) => self.save.flag(SPRINGS_FREED) < n,
            None => area.spring.is_some() && !area.freed(&self.save),
        }
    }

    /// Weather for the map `name` (R2-W1): own weather from the editor comes first, guardians'
    /// arenas stay fair (D-W1-01), otherwise rolled from the area's lists – fixed by map and
    /// play time, so it does not change when loading.
    pub fn pick_weather(&self, name: &str) -> elora_map::Weather {
        use elora_map::{Weather, WeatherKind};
        if !self.base_weather.is_clear() {
            return self.base_weather;
        }
        if name.ends_with("-arena") {
            return Weather::CLEAR;
        }
        let Some(area) = self.content.area_of(name) else {
            return Weather::CLEAR;
        };
        let list = if self.gloomy(area) && !area.weather_gloomy.is_empty() {
            &area.weather_gloomy
        } else {
            &area.weather
        };
        let total: u32 = list.iter().map(|w| w.weight).sum();
        if total == 0 {
            return Weather::CLEAR;
        }
        let mut seed = name.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
        }) ^ self.save.play_time_secs.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        let mut next = || {
            seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = seed;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        };
        let mut roll = next() % u64::from(total);
        let chosen = list
            .iter()
            .find(|w| {
                let hit = roll < u64::from(w.weight);
                roll = roll.saturating_sub(u64::from(w.weight));
                hit
            })
            .unwrap_or(&list[0]);
        let mut unit = || (next() >> 40) as f32 / (1u64 << 24) as f32;
        let lerp = |r: [f32; 2], t: f32| r[0] + (r[1] - r[0]) * t;
        let kind = WeatherKind::from_key(&chosen.kind).unwrap_or_default();
        if kind == WeatherKind::Clear {
            return Weather::CLEAR;
        }
        Weather {
            kind,
            intensity: lerp(chosen.intensity, unit()),
            wind: lerp(chosen.wind, unit()),
        }
    }

    /// Does the area get brighter (spring freed while Elora is there)? Then roll again.
    pub(super) fn refresh_weather(&mut self) -> bool {
        let Some(area) = self.content.area_of(&self.map_name) else {
            return false;
        };
        let gloomy = self.gloomy(area);
        if gloomy == self.gloomy {
            return false;
        }
        self.gloomy = gloomy;
        let weather = self.pick_weather(&self.map_name.clone());
        let changed = weather != self.map.weather;
        self.map.weather = weather;
        changed
    }
}

/// How the simulation feels the weather (R2-W1, E-330): wind (sandstorm and blizzard always
/// blow), gusts, wetness in rain and snow, lightning in thunderstorms.
pub fn weather_env(w: elora_map::Weather) -> Option<elora_sim::WeatherEnv> {
    use elora_map::WeatherKind as K;
    if w.is_clear() {
        return None;
    }
    let strong = matches!(w.kind, K::Sandstorm | K::Blizzard);
    let wind = if strong && w.wind.abs() < 0.2 {
        0.7
    } else {
        w.wind
    };
    let (wet, lightning) = match w.kind {
        K::Rain => (w.intensity, 0.0),
        K::Storm => (w.intensity, w.intensity),
        K::Snow | K::Blizzard => (w.intensity * 0.8, 0.0),
        _ => (0.0, 0.0),
    };
    Some(elora_sim::WeatherEnv {
        wind,
        gusty: matches!(
            w.kind,
            K::Storm | K::Blizzard | K::Sandstorm | K::Leaves | K::Petals
        ),
        wet,
        lightning,
    })
}
