//! The frame loop: frame time, advancing the game, building and presenting the frame.

use std::time::{Duration, Instant};

use elora_client::online::Status;
use elora_client::scene::Scene;
use elora_render::Camera;
use elora_sim::{Collision, Tuning, Vec2};

use crate::{App, FrameInfo, Screen, draw, emotes, game_ui, hud, map_view};

impl App {
    /// One frame: intro, menu, editor or the game.
    pub(crate) fn redraw(&mut self) {
        let now = Instant::now();
        let elapsed = self.frame_time(now);
        let dt = elapsed.as_secs_f32();
        if self.intro.is_some() {
            self.redraw_intro();
            return;
        }
        if self.screen == Screen::Menu {
            self.redraw_menu(dt);
            return;
        }
        if self.screen == Screen::Editor {
            self.figures.advance_time(dt);
            self.redraw_editor();
            return;
        }
        self.redraw_game(now, elapsed);
    }

    /// One frame of the game: advance, build world and HUD, present, then handle the choices.
    fn redraw_game(&mut self, now: Instant, elapsed: Duration) {
        let dt = elapsed.as_secs_f32();
        // scene from sandbox or online game
        let Some((scene, collision, tuning, events)) = self.advance(now, elapsed) else {
            return;
        };
        self.update_looks(dt, &scene, &collision, &events);
        let info = self.game_info(now);
        game_ui::record_kills(
            &mut self.killfeed,
            &events,
            &info.names,
            &info.teams,
            now,
            &self.lang,
        );

        let Some(aspect) = self.gfx.as_ref().map(|g| g.renderer.aspect()) else {
            return;
        };
        let camera = self.game_camera(&scene, info.tick, aspect, dt);
        self.update_weather(dt, &camera);
        self.build_batch(&scene, &tuning, &camera, info.tick);
        let screen = self.build_hud(&scene, &tuning, &info);
        let death_choice = if self.adventure.is_some() && !self.menu.paused {
            let s = hud::scale(screen, self.settings.graphics.ui_scale());
            self.draw_adventure_hud(&camera, screen, s, dt)
        } else {
            None
        };
        let pause_action = if self.menu.paused {
            self.draw_pause(dt, &info)
        } else {
            None
        };
        self.world_look(dt, info.tick);
        if !self.present(&camera, screen, &info) {
            return;
        }
        if let Some(a) = pause_action {
            self.apply_menu(a);
        }
        match death_choice {
            Some(true) => self.adventure_respawn(),
            Some(false) => self.leave_adventure(),
            None => {}
        }
    }

    /// Camera of the frame: adventure camera, screen shake and wobble, snapped to screen pixels.
    fn game_camera(&mut self, scene: &Scene, tick: u64, aspect: f32, dt: f32) -> Camera {
        let center = self.adventure_camera(scene.camera, self.view.view_size(aspect), dt);
        // colourful rush: the world wobbles slightly (E-311)
        let wobble = scene
            .local()
            .filter(|c| c.ch.core.dazed > 0)
            .map_or(Vec2::ZERO, |c| {
                #[allow(clippy::cast_precision_loss)]
                let t = tick as f32 / elora_sim::TICKS_PER_SECOND as f32;
                let fade = (c.ch.core.dazed as f32 / 25.0).min(1.0);
                Vec2::new((t * 2.3).sin() * 7.0, (t * 1.7).cos() * 4.0) * fade
            });
        let mut camera = Camera::new(
            center + self.effects.camera_offset() + wobble,
            &self.view,
            aspect,
        );
        // snap to whole screen pixels: fine lines stay calm while scrolling (E-288)
        if let Some(gfx) = &self.gfx {
            #[allow(clippy::cast_precision_loss)]
            let px = gfx.renderer.size().0 as f32 / camera.size.x;
            if px > 0.0 {
                camera.center = Vec2::new(
                    (camera.center.x * px).round() / px,
                    (camera.center.y * px).round() / px,
                );
            }
        }
        camera
    }

    /// Draw world, HUD and debug panel; `false` if there is no frame to draw into.
    fn present(&mut self, camera: &Camera, screen: Vec2, info: &FrameInfo) -> bool {
        let Some(gfx) = &mut self.gfx else {
            return false;
        };
        let Some(mut frame) = gfx.renderer.begin_frame() else {
            return false;
        };
        gfx.renderer
            .draw_shapes(&mut frame, camera, &self.batch, draw::BACKGROUND);
        let screen_camera = Camera {
            center: screen * 0.5,
            size: screen,
        };
        gfx.renderer
            .draw_overlay(&mut frame, &screen_camera, &self.hud_batch);
        self.draw_debug_panel(&mut frame, info);
        if let Some(gfx) = &mut self.gfx {
            gfx.renderer.end_frame(frame);
        }
        true
    }

    /// Advance the simulation or online client; returns what to draw.
    fn advance(
        &mut self,
        now: Instant,
        elapsed: Duration,
    ) -> Option<(Scene, Collision, Tuning, Vec<elora_sim::Event>)> {
        Some(if self.online.is_some() {
            self.update_online(now);
            let o = self.online.as_mut()?;
            match (
                o.client.scene(now),
                o.client.map.as_ref(),
                o.client.tuning(),
            ) {
                (Some(scene), Some(map), Some(t)) => {
                    (scene, map.collision(), t.clone(), o.client.take_events())
                }
                _ => (
                    Scene::default(),
                    Collision::new(1, 1, vec![elora_sim::Tile::Air]),
                    Tuning::default(),
                    Vec::new(),
                ),
            }
        } else if self.adventure.is_some() {
            self.advance_adventure(elapsed);
            let mut scene = self.sandbox.scene();
            self.adventure_scene(&mut scene);
            (
                scene,
                self.sandbox.world.collision.clone(),
                self.sandbox.world.tuning.clone(),
                self.sandbox.take_events(),
            )
        } else {
            self.sandbox.poll_reload();
            if self
                .sandbox
                .recording
                .as_ref()
                .is_some_and(|r| r.tuning != self.sandbox.world.tuning)
                && let Some(msg) = self.sandbox.stop_recording("tuning changed")
            {
                self.status = msg;
            }
            self.sandbox.advance(elapsed, &mut self.controls);
            (
                self.sandbox.scene(),
                self.sandbox.world.collision.clone(),
                self.sandbox.world.tuning.clone(),
                self.sandbox.take_events(),
            )
        })
    }

    /// Names, teams, game state, own slot, tick, vote and chat of the frame.
    pub(crate) fn game_info(&self, now: Instant) -> FrameInfo {
        match &self.online {
            Some(o) => FrameInfo {
                names: o.client.names.clone(),
                teams: o.client.teams(),
                view: o.client.game(),
                local: o.client.slot,
                tick: o.client.server_tick(now).unwrap_or(0),
                vote: o.client.vote.clone(),
                chat: o.client.chat.iter().cloned().collect(),
                loading: match &o.client.status {
                    Status::Loading {
                        map,
                        received,
                        size,
                    } => Some((map.clone(), *received, *size)),
                    _ => None,
                },
            },
            None => FrameInfo {
                names: self.sandbox.names(),
                teams: self
                    .sandbox
                    .world
                    .players
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| Some((i, p.as_ref()?.team)))
                    .collect(),
                view: self.sandbox.game(),
                local: Some(self.sandbox.player),
                tick: self.sandbox.world.tick,
                vote: None,
                chat: self.sandbox.notices.iter().cloned().collect(),
                loading: None,
            },
        }
    }

    /// Time since the last frame; updates the FPS display.
    fn frame_time(&mut self, now: Instant) -> Duration {
        let elapsed = now - self.last_frame;
        self.last_frame = now;
        if elapsed.as_secs_f32() > 0.0 {
            self.fps = self.fps * 0.95 + (1.0 / elapsed.as_secs_f32()) * 0.05;
        }
        if self.frame_times.len() >= 240 {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(elapsed.as_secs_f32() * 1000.0);
        elapsed
    }

    /// Collect world, characters and effects of the frame in `self.batch`.
    fn build_batch(&mut self, scene: &Scene, tuning: &Tuning, camera: &Camera, tick: u64) {
        self.batch.clear();
        let map = match &self.online {
            Some(o) => o.client.map.as_ref(),
            None => Some(&self.sandbox.map),
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let look_time = map_view::LookTime {
            local_ms: (f64::from(self.figures.time()) * 1000.0) as i64,
            server_ms: (tick * 1000 / u64::from(elora_sim::TICKS_PER_SECOND)) as i64,
            // only in the own game (adventure, training); online without guardians
            hook_wilt: self
                .online
                .is_none()
                .then_some(self.sandbox.world.collision.hook_wilt)
                .flatten(),
            wind: self.weather.wind(),
        };
        draw::scene(
            &mut self.batch,
            scene,
            map_view::MapLayer {
                map,
                view: &mut self.map_view,
                time: look_time,
            },
            tuning,
            camera,
            self.controls.mouse_pos,
            &draw::Looks {
                effects: &self.effects,
                figures: &self.figures,
                art: &self.figure_art,
                items: &self.item_art,
                creatures: &self.creature_art,
                weather: &self.weather,
                emotes: &self.emotes,
                skins: self
                    .online
                    .as_ref()
                    .map_or(&self.no_skins, |o| &o.client.skins),
                own_skin: self.net.skin,
            },
        );
    }

    /// Collect the HUD of the frame in `self.hud_batch`; returns the screen size.
    fn build_hud(&mut self, scene: &Scene, tuning: &Tuning, info: &FrameInfo) -> Vec2 {
        let size = self.gfx.as_ref().map_or((1, 1), |g| g.renderer.size());
        #[allow(clippy::cast_precision_loss)]
        let screen = Vec2::new(size.0 as f32, size.1 as f32);
        self.hud_batch.clear();
        self.hud.draw(
            &mut self.hud_batch,
            &self.item_art,
            screen,
            &hud::HudInfo {
                character: scene.local().map(|c| &c.ch),
                max_health: tuning.max_health,
                view: info.view.as_ref(),
                tick: info.tick,
                local: info.local,
                lang: &self.lang,
                ui_scale: self.settings.graphics.ui_scale(),
            },
        );
        game_ui::draw(
            &mut self.hud_batch,
            self.hud.font(),
            &self.item_art,
            screen,
            hud::scale(screen, self.settings.graphics.ui_scale()),
            &game_ui::GameUi {
                view: info.view.as_ref(),
                names: &info.names,
                teams: &info.teams,
                local: info.local,
                chat: &info.chat,
                input: &self.chat_input,
                scoreboard: self.scoreboard,
                vote: info.vote.as_ref(),
                killfeed: &self.killfeed,
                loading: info.loading.as_ref().map(|(m, r, s)| (m.as_str(), *r, *s)),
                time: self.figures.time(),
                lang: &self.lang,
            },
        );
        if self.emotes.wheel_open {
            let selected = emotes::selection(self.controls.mouse_pos);
            self.emotes.draw_wheel(
                &mut self.hud_batch,
                screen,
                hud::scale(screen, self.settings.graphics.ui_scale()),
                selected,
            );
        }
        screen
    }
}
