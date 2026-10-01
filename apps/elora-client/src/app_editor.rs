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
            let mut editor = Editor::new(settings::user_maps_dir(), PathBuf::from("maps"));
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
        let Some(mut frame) = gfx.renderer.begin_frame() else {
            return;
        };
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
        if std::mem::take(&mut editor.leave) {
            // im Editor gespeicherte Karten sofort in Training und „Server erstellen“
            self.maps = app_menu::map_names();
            self.screen = Screen::Menu;
        }
    }
}
