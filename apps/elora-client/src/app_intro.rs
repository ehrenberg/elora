//! Intro video before a new adventure (I-3, E-355, E-357): drawing, skipping, narration and
//! music.

use std::time::Instant;

use elora_render::{Align, Camera, Color};
use elora_sim::Vec2;
use winit::keyboard::KeyCode;

use crate::App;
use crate::intro::Intro;

impl App {
    /// Starts the intro if the video exists; the adventure waits underneath.
    pub(crate) fn start_intro(&mut self) {
        self.intro = Intro::start_default(Instant::now());
        if self.intro.is_some() {
            self.controls.release_all();
            let lang = self.lang_code();
            self.sounds.intro(lang);
        }
    }

    /// Esc, Space, Enter or a click skip the intro (not in the first half second).
    pub(crate) fn intro_key(&mut self, code: KeyCode) {
        if matches!(
            code,
            KeyCode::Escape | KeyCode::Space | KeyCode::Enter | KeyCode::NumpadEnter
        ) {
            self.skip_intro();
        }
    }

    pub(crate) fn skip_intro(&mut self) {
        if self
            .intro
            .as_ref()
            .is_some_and(|i| i.can_skip(Instant::now()))
        {
            self.end_intro();
        }
    }

    fn end_intro(&mut self) {
        self.intro = None;
        self.sounds.end_intro();
        if let Some(gfx) = &mut self.gfx {
            gfx.renderer.clear_picture();
        }
        self.controls.release_all();
        // the game starts now, not when the intro started
        self.last_frame = Instant::now();
    }

    /// Draws the current video frame and the skip hint.
    pub(crate) fn redraw_intro(&mut self) {
        let now = Instant::now();
        let (screen, s) = self.menu_ctx_parts();
        let Some(intro) = &mut self.intro else { return };
        if intro.finished(now) {
            self.end_intro();
            return;
        }
        let frame = intro.update(now);
        self.hud_batch.clear();
        let font = self.hud.font();
        if intro.can_skip(now) {
            let key = crate::bindings::Trigger::Key(KeyCode::Space).label(&self.lang);
            font.draw(
                &mut self.hud_batch,
                &self.lang.f("intro.skip", &[("key", &key)]),
                Vec2::new(screen.x - 16.0 * s, 26.0 * s),
                13.0 * s,
                Color::rgba(1.0, 1.0, 1.0, 0.6),
                Align::Right,
            );
        }
        let Some(gfx) = &mut self.gfx else { return };
        if let Some(f) = frame {
            gfx.renderer.set_picture(f.width, f.height, &f.rgba);
        }
        let Some(mut target) = gfx.renderer.begin_frame() else {
            return;
        };
        gfx.renderer.draw_picture(&mut target);
        let camera = Camera {
            center: screen * 0.5,
            size: screen,
        };
        gfx.renderer
            .draw_overlay(&mut target, &camera, &self.hud_batch);
        gfx.renderer.end_frame(target);
    }
}
