//! Wetter in der Simulation (R2-W1, W1.4, E-330, E-336): nur im Abenteuer. Wind mit Böen
//! schiebt Elora in der Luft und lenkt Granaten ab, Nässe macht den Boden weicher, im
//! Gewitter schlagen Blitze mit Warnung in Eloras Nähe ein.
//!
//! Alles deterministisch aus dem Tick; im Mehrspieler bleibt [`World::weather`] leer.

use crate::character::PHYS_SIZE;
use crate::creature::{rng, rng_f32};
use crate::event::{DeathCause, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;
use crate::world::World;

/// Wetter, wie die Simulation es spürt (die Sitzung setzt es aus dem Wetter der Karte).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WeatherEnv {
    /// Grundwind (−1..1).
    pub wind: f32,
    /// Böen (Gewitter, Stürme, Wind mit Blättern oder Blüten).
    pub gusty: bool,
    /// Nässe des Bodens (Regen, Schnee): 0..1.
    pub wet: f32,
    /// Blitze (Gewitter): Stärke 0..1, 0 = keine.
    pub lightning: f32,
}

/// Blitz-Zustand der Welt.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Lightning {
    /// Ticks bis zur nächsten Warnung (`None` = noch nicht gestellt).
    pub timer: Option<u32>,
    /// Angekündigter Einschlag: Ort am Boden und verbleibende Ticks.
    pub strike: Option<(Vec2, u32)>,
}

impl World {
    /// Wind (mit Böen) und Nässe dieses Ticks in die Kollision; Blitze. Vor den Figuren.
    pub(crate) fn tick_weather(&mut self) {
        let Some(w) = self.weather.filter(|_| self.adventure) else {
            self.collision.wind = 0.0;
            self.collision.wet = 0.0;
            return;
        };
        #[allow(clippy::cast_precision_loss)]
        let gust = if w.gusty {
            (self.tick as f32 * 0.031).sin() * 0.3 * w.wind.abs().max(0.3)
        } else {
            0.0
        };
        self.collision.wind = (w.wind + gust).clamp(-1.3, 1.3);
        self.collision.wet = w.wet.clamp(0.0, 1.0);
        if w.lightning > 0.0 && !self.prediction {
            self.tick_lightning(w.lightning);
        }
    }

    fn tick_lightning(&mut self, strength: f32) {
        // angekündigter Einschlag
        if let Some((pos, left)) = self.lightning.strike {
            if left > 1 {
                self.lightning.strike = Some((pos, left - 1));
            } else {
                self.lightning.strike = None;
                self.strike(pos);
            }
            return;
        }
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let every = (ms_to_ticks(self.tuning.lightning_every) as f32 / strength.max(0.3)) as u32;
        let timer = self.lightning.timer.get_or_insert(every / 2);
        if *timer > 0 {
            *timer -= 1;
            return;
        }
        // Ziel in der Nähe einer Figur, auf dem Boden darunter
        let r = rng(self.tick ^ 0x5eed_b1a5);
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        {
            self.lightning.timer = Some((every as f32 * (0.6 + rng_f32(r ^ 1) * 0.8)) as u32);
        }
        let Some(p) = self
            .players
            .iter()
            .filter_map(|p| p.as_ref()?.character.as_ref())
            .map(|c| c.core.pos)
            .next()
        else {
            return;
        };
        #[allow(clippy::cast_precision_loss)]
        let x = p.x + (r % 361) as f32 - 180.0;
        let ts = crate::TILE_SIZE as f32;
        let ground = (0..30).find_map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let at = Vec2::new(x, p.y - 160.0 + k as f32 * ts);
            let t = self.collision.tile_at(at);
            (t.is_solid() || t == crate::Tile::Platform).then(|| (at.y / ts).floor() * ts)
        });
        if let Some(y) = ground {
            let pos = Vec2::new(x, y);
            self.events.push(Event::LightningWarn { pos });
            self.lightning.strike = Some((pos, ms_to_ticks(self.tuning.lightning_warn).max(1)));
        }
    }

    /// Einschlag: Schaden und Stoß im Umkreis (E-336).
    fn strike(&mut self, pos: Vec2) {
        self.events.push(Event::Lightning { pos });
        let (radius, damage, kb) = (
            self.tuning.lightning_radius,
            self.tuning.lightning_damage,
            self.tuning.hit_knockback,
        );
        let hits: Vec<(usize, Vec2)> = self
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| Some((i, p.as_ref()?.character.as_ref()?.core.pos)))
            .filter(|&(_, c)| {
                (c + Vec2::new(0.0, PHYS_SIZE / 2.0)).distance(pos) < radius + PHYS_SIZE / 2.0
            })
            .collect();
        for (i, c) in hits {
            let side = if c.x < pos.x { -1.0 } else { 1.0 };
            self.take_damage(
                i,
                Vec2::new(side * kb * 0.6, -kb),
                damage,
                None,
                DeathCause::World,
            );
        }
    }
}
