//! Look of the world per frame: weather, post-processing, effects and animations.

use elora_client::scene::Scene;
use elora_render::Camera;
use elora_sim::{Collision, Event};

use crate::{App, draw, figure, skins};

/// Faint glow of the star islands in the dark (M2.5.2).
const ISLAND_GLOW: f32 = 0.55;

/// Light in the dark besides the decoration (drawing only): star islands glow faintly from
/// their star crystals (M2.5.2), lit note stones and star bridges shine (M2.5.3). Centre, radius
/// and intensity.
fn glows(world: &elora_sim::World) -> Vec<(elora_sim::Vec2, f32, f32)> {
    let mut out: Vec<_> = world
        .islands
        .iter()
        .map(|island| {
            let at = crate::map_art::island_crystal(island.pos, island.size);
            (at, island.size.x * 0.5 + 40.0, ISLAND_GLOW)
        })
        .collect();
    let tick = world.tick;
    for stone in world.note_stones.iter().filter(|s| s.lit(tick)) {
        out.push((stone.centre(), elora_sim::notes::STONE_LIGHT, 1.0));
    }
    for bridge in world.bridges.iter().filter(|b| b.active(tick)) {
        let mut x = bridge.at.x;
        while x <= bridge.at.x + bridge.size.x {
            out.push((
                elora_sim::Vec2::new(x, bridge.at.y),
                elora_sim::notes::BRIDGE_LIGHT * 1.4,
                0.8,
            ));
            x += elora_sim::notes::BRIDGE_LIGHT;
        }
    }
    out
}

/// Saturation of the adventure world by number of freed springs (E-328): pale, then vivid.
const SATURATION_BY_SPRINGS: [f32; 6] = [0.62, 0.74, 0.84, 0.92, 0.97, 1.0];

impl App {
    /// Post-processing of the world for this frame: heat shimmer in the desert (E-320, stronger in
    /// the blazing sun and with the heat) and colour that returns with the springs
    /// (E-328, more colourful per freed spring; the change glides over a few seconds).
    pub(crate) fn world_look(
        &mut self,
        dt: f32,
        tick: u64,
        camera: &Camera,
        elora: Option<elora_sim::Vec2>,
    ) {
        let session = self.adventure.as_ref().map(|a| &a.session);
        let haze = session
            .filter(|s| s.hot())
            .map_or(0.0, |s| (if s.in_sun { 0.6 } else { 0.3 }) + 0.4 * s.heat);
        // frost edge from a third of the cold bar on, when full it pulses slightly (E-342)
        let pulse = (tick as f32 * 0.12).sin() * 0.08;
        let frost = session.filter(|s| s.chilly()).map_or(0.0, |s| {
            ((s.cold - 0.33) / 0.67).clamp(0.0, 1.0) + if s.frozen { pulse } else { 0.0 }
        });
        let target = session.map_or(1.0, |s| {
            let freed = s.save.flag(elora_adventure::session::SPRINGS_FREED);
            SATURATION_BY_SPRINGS[usize::try_from(freed.clamp(0, 5)).unwrap_or(0)]
        });
        let step = dt * 0.2;
        self.saturation += (target - self.saturation).clamp(-step, step);
        if let Some(gfx) = &mut self.gfx {
            let secs = tick as f32 / elora_sim::TICKS_PER_SECOND as f32;
            gfx.renderer.set_heat_haze(haze, secs);
            gfx.renderer.set_saturation(self.saturation);
            gfx.renderer.set_frost(frost);
            gfx.renderer.set_grade(self.weather.grade());
        }
        self.night(camera, elora);
    }

    /// Night of dark areas (R2-M2.5, M2.5.1): darkness of the area (or the debug preview),
    /// Elora's light and the glowing decoration in view. `elora` is her drawn (interpolated)
    /// position, so the light moves exactly with her and the camera.
    fn night(&mut self, camera: &Camera, elora: Option<elora_sim::Vec2>) {
        let session = self.adventure.as_ref().map(|a| &a.session);
        let darkness = session
            .map_or(0.0, elora_adventure::Session::darkness)
            .max(self.sandbox.night_preview)
            * self.settings.graphics.night.factor();
        let Some(gfx) = &mut self.gfx else { return };
        if darkness <= 0.0 {
            gfx.renderer.set_darkness(0.0, &[]);
            return;
        }
        let (w, h) = gfx.renderer.size();
        #[allow(clippy::cast_precision_loss)]
        let screen = elora_sim::Vec2::new(w as f32, h as f32);
        let per_px = camera.units_per_pixel(screen).max(0.0001);
        let elora = elora.map(|pos| (pos, elora_map::lights::elora_radius(), 1.0));
        let lights: Vec<elora_render::Light> = elora
            .into_iter()
            .chain(
                elora_map::lights::map_lights(&self.sandbox.map)
                    .into_iter()
                    .map(|(p, r)| (p, r, 1.0)),
            )
            .chain(glows(&self.sandbox.world))
            .map(|(pos, r, intensity)| elora_render::Light {
                pos: camera.world_to_screen(pos, screen),
                radius: r / per_px,
                intensity,
            })
            .filter(|l| {
                l.pos.x > -l.radius
                    && l.pos.y > -l.radius
                    && l.pos.x < screen.x + l.radius
                    && l.pos.y < screen.y + l.radius
            })
            .collect();
        gfx.renderer.set_darkness(darkness, &lights);
    }

    /// Weather of the map (online: sent along by the server) – rendering only (R2-W1).
    pub(crate) fn update_weather(&mut self, dt: f32, camera: &Camera) {
        let map = match &self.online {
            Some(o) => o.client.map.as_ref(),
            None => Some(&self.sandbox.map),
        };
        // in the adventure the weather of the session applies (guardians can change it, R2-M2.4)
        let weather = match &self.adventure {
            Some(a) => a.session.map.weather,
            None => map.map_or(elora_map::Weather::CLEAR, |m| m.weather),
        };
        // in the adventure the lightning comes from the simulation (with warning and damage, E-336)
        let random_bolts = self.adventure.is_none();
        self.weather.update(
            dt,
            weather,
            self.settings.graphics.weather,
            camera,
            map,
            random_bolts,
        );
        // ambient track: rain, wind, sand, thunder (W1.5, E-338)
        let thunder = self.weather.take_thunder();
        let shelter = self.weather.shelter();
        self.sounds
            .weather(dt, weather, shelter, thunder, camera.center);
        // fireplaces crackle, the closer the louder (zones `feuer…`, R2-M2.4)
        let fire = self.adventure.as_ref().map_or(0.0, |a| {
            a.session
                .map
                .adventure
                .objects
                .iter()
                .filter(|o| o.id.starts_with("fire"))
                .filter_map(|o| o.kind.area().map(|s| o.pos + s * 0.5))
                .map(|c| c.distance(camera.center))
                .fold(f32::MAX, f32::min)
        });
        let level = if self.adventure.is_some() {
            (1.0 - (fire - 150.0) / 450.0).clamp(0.0, 1.0) * 0.8
        } else {
            0.0
        };
        self.sounds.fire(level);
    }

    /// Advance effects and character animations, sync the own skin.
    pub(crate) fn update_looks(
        &mut self,
        dt: f32,
        scene: &Scene,
        collision: &Collision,
        events: &[Event],
    ) {
        self.figures.update(dt, scene, collision, events);
        self.emotes.update(dt);
        // lightning from the simulation (adventure, R2-W1): draw warning and strike
        for e in events {
            match *e {
                elora_sim::Event::LightningWarn { pos } => self.weather.sim_warn(pos),
                elora_sim::Event::Lightning { pos } => self.weather.sim_strike(pos),
                _ => {}
            }
        }
        if let Some(o) = &mut self.online {
            for (slot, emote) in o.client.take_emotes() {
                self.emotes.show(slot, emote);
            }
        }
        let skins = self
            .online
            .as_ref()
            .map_or(&self.no_skins, |o| &o.client.skins);
        let own = self.net.skin;
        self.effects
            .update(dt, events, scene, self.figures.landings(), |c| {
                let skin = if c.local {
                    own
                } else {
                    skins.get(&c.slot).copied().unwrap_or_default()
                };
                skins::tint(skin, c.team, c.dummy, draw::team_color).colors[figure::KEY_BODY]
            });
        // glow mushrooms near the camera let little stars rise (R2-M2.2)
        if self.online.is_none() {
            let near: Vec<elora_sim::Vec2> = self
                .sandbox
                .map
                .decor_front
                .iter()
                .chain(&self.sandbox.map.decor_back)
                .filter(|d| d.art == elora_map::Art::Builtin("glow_mushrooms".into()))
                .map(|d| d.pos)
                .filter(|p| p.distance(scene.camera) < 1100.0)
                .collect();
            self.effects.glow_spores(dt, &near);
        }
        if let Some(o) = &mut self.online {
            // only sends a message on change
            o.client.set_skin(self.net.skin);
        }
        self.auto_switch(scene, events);
        self.play_sounds(scene, events);
    }
}
