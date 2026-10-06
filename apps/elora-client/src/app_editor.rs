//! Editor im Programm (M6.6): Wechsel aus dem Hauptmenü, Zeichnen und egui-Oberfläche je Frame.

use std::path::PathBuf;
use std::time::Instant;

use elora_sim::Vec2;

use crate::editor::{Editor, panel, view};
use crate::{App, Screen, app_menu, settings};

impl App {
    /// Aus dem Hauptmenü in den Editor; der letzte Stand bleibt erhalten.
    pub(crate) fn enter_editor(&mut self) {
        if self.editor.is_none() {
            let mut editor = Editor::new(
                settings::user_maps_dir(),
                elora_server::paths::resolve(&PathBuf::from("maps")),
            );
            editor.solid_materials = self.map_view.art.solid_material_names();
            if let Some(first) = editor.solid_materials.first() {
                editor.solid_material.clone_from(first);
            }
            self.editor = Some(editor);
        }
        self.sounds.menu_music(false);
        self.screen = Screen::Editor;
        self.set_cursor_grab(false);
    }

    /// Karte des Editors in einer Trainingsrunde spielen (M6.9), ohne zu speichern.
    fn start_editor_test(&mut self) {
        let Some(editor) = &self.editor else { return };
        let map = editor.map.clone();
        self.online = None;
        let previous = self.sandbox.play_map(map);
        // nur die erste Karte merken (nicht die eines vorigen Testspiels)
        self.editor_test.get_or_insert(previous);
        self.sandbox.notice(elora_protocol::Message::Text(
            self.lang.t("editor.test_notice").to_owned(),
        ));
        self.enter_game();
    }

    /// Ist gerade ein Testspiel aus dem Editor aktiv?
    pub(crate) fn testing_map(&self) -> bool {
        self.editor_test.is_some()
    }

    /// Testspiel beenden: vorige Trainingskarte zurück, Editor wie zuvor.
    pub(crate) fn leave_editor_test(&mut self) {
        if let Some(previous) = self.editor_test.take() {
            self.sandbox.play_map(previous);
        }
        self.menu.paused = false;
        self.screen = Screen::Editor;
        self.set_cursor_grab(false);
    }

    pub(crate) fn redraw_editor(&mut self) {
        let now = Instant::now();
        let Some(editor) = &mut self.editor else {
            self.screen = Screen::Menu;
            return;
        };
        let Some(gfx) = &mut self.gfx else { return };
        let (w, h) = gfx.renderer.size();
        #[allow(clippy::cast_precision_loss)]
        let window = Vec2::new(w as f32, h as f32);
        let area_center = if self.editor_area.center_px == Vec2::ZERO {
            window * 0.5
        } else {
            self.editor_area.center_px
        };
        let camera = view::camera(editor, window, area_center);
        self.batch.clear();
        view::draw(
            &mut self.batch,
            editor,
            &mut self.map_view,
            &self.item_art,
            &camera,
            self.figures.time(),
            self.editor_area.preview,
        );
        if editor.visible.entities {
            crate::editor::panel_adventure::draw(
                &mut self.batch,
                editor,
                &self.creature_art,
                self.figures.time(),
            );
        }
        let Some(mut frame) = gfx.renderer.begin_frame() else {
            return;
        };
        gfx.renderer.set_heat_haze(0.0, 0.0);
        gfx.renderer.set_saturation(1.0);
        gfx.renderer.set_grade(elora_render::Grade::NONE);
        gfx.renderer
            .draw_shapes(&mut frame, &camera, &self.batch, view::OUTSIDE);
        let lang = &self.lang;
        let map_view = &mut self.map_view;
        #[allow(clippy::cast_possible_truncation)]
        let time_ms = (f64::from(self.figures.time()) * 1000.0) as i64;
        let mut area = self.editor_area;
        gfx.gui.draw(&gfx.window, &gfx.renderer, &mut frame, |ui| {
            let ppp = ui.ctx().pixels_per_point();
            area = panel::ui(ui, editor, map_view, lang, window, ppp, time_ms, now);
        });
        gfx.renderer.end_frame(frame);
        self.editor_area = area;
        match editor.request.take() {
            Some(crate::editor::Request::Test) => self.start_editor_test(),
            Some(crate::editor::Request::TestAdventure) => self.start_adventure_test(),
            Some(crate::editor::Request::Leave) => self.leave_editor(),
            None => {}
        }
    }

    fn leave_editor(&mut self) {
        // im Editor gespeicherte Karten sofort in Training und „Server erstellen“
        self.maps = app_menu::map_names();
        self.screen = Screen::Menu;
    }
}
