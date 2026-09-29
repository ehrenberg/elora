//! Vektortext (M5.8): Glyphen aus einer TrueType-Schrift, einmal tesselliert und
//! gecacht, beim Zeichnen nur verschoben und skaliert. Für die eigene Spiel-UI
//! (E-031) – HUD jetzt, Menüs später.
//!
//! Einfaches Layout: eine Zeile, Vorschub je Glyphe, keine Unterschneidung und
//! keine Ligaturen. Für Zahlen, Namen und kurze Hinweise reicht das.

use std::cell::RefCell;
use std::collections::HashMap;

use elora_sim::Vec2;
use lyon::math::point;
use lyon::path::Path;
use lyon::path::builder::WithSvg;

use crate::mesh::{Affine, Mesh, MeshBuilder, Paint, Tint};
use crate::{Color, ShapeBatch};

#[derive(Debug, thiserror::Error)]
#[error("Schrift nicht lesbar: {0}")]
pub struct FontError(#[from] ttf_parser::FaceParsingError);

/// Ausrichtung einer Zeile relativ zum Ankerpunkt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// Tessellierte Glyphe: Mesh in em-Einheiten (y nach unten, Grundlinie bei 0) und Vorschub.
#[derive(Debug)]
struct Glyph {
    mesh: Mesh,
    advance: f32,
}

/// Eine Schrift mit Glyphen-Cache.
pub struct Font {
    face: ttf_parser::Face<'static>,
    glyphs: RefCell<HashMap<char, Glyph>>,
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("cached_glyphs", &self.glyphs.borrow().len())
            .finish_non_exhaustive()
    }
}

/// Toleranz der Tessellierung in em (bei 20 px Schriftgröße ≈ 0.04 px).
const TOLERANCE_EM: f32 = 0.002;

impl Font {
    /// # Errors
    /// Wenn `data` keine gültige TrueType/OpenType-Schrift ist.
    pub fn new(data: &'static [u8]) -> Result<Self, FontError> {
        Ok(Self {
            face: ttf_parser::Face::parse(data, 0)?,
            glyphs: RefCell::new(HashMap::new()),
        })
    }

    fn em(&self) -> f32 {
        f32::from(self.face.units_per_em())
    }

    /// Höhe der Großbuchstaben in em (für vertikales Zentrieren).
    pub fn cap_height(&self) -> f32 {
        f32::from(self.face.capital_height().unwrap_or(700)) / self.em()
    }

    fn with_glyph<R>(&self, c: char, f: impl FnOnce(&Glyph) -> R) -> R {
        if let Some(g) = self.glyphs.borrow().get(&c) {
            return f(g);
        }
        let glyph = self.build(c);
        let r = f(&glyph);
        self.glyphs.borrow_mut().insert(c, glyph);
        r
    }

    fn build(&self, c: char) -> Glyph {
        let em = self.em();
        let id = self
            .face
            .glyph_index(c)
            .or_else(|| self.face.glyph_index('?'))
            .unwrap_or_default();
        let advance = f32::from(self.face.glyph_hor_advance(id).unwrap_or(0)) / em;
        let mut outline = Outline {
            builder: Path::svg_builder(),
            scale: 1.0 / em,
        };
        let mesh = if self.face.outline_glyph(id, &mut outline).is_some() {
            MeshBuilder::new(TOLERANCE_EM)
                .fill_path(&outline.builder.build(), Paint::key(1))
                .build()
        } else {
            Mesh::default() // z. B. Leerzeichen
        };
        Glyph { mesh, advance }
    }

    /// Breite von `text` bei Schriftgröße `size`.
    pub fn width(&self, text: &str, size: f32) -> f32 {
        text.chars()
            .map(|c| self.with_glyph(c, |g| g.advance))
            .sum::<f32>()
            * size
    }

    /// Zeichnet eine Zeile; `pos` ist der Ankerpunkt auf der Grundlinie.
    pub fn draw(
        &self,
        batch: &mut ShapeBatch,
        text: &str,
        pos: Vec2,
        size: f32,
        color: Color,
        align: Align,
    ) {
        let width = self.width(text, size);
        let mut x = pos.x
            - match align {
                Align::Left => 0.0,
                Align::Center => width / 2.0,
                Align::Right => width,
            };
        let tint = Tint::new(vec![color]);
        for c in text.chars() {
            self.with_glyph(c, |g| {
                if !g.mesh.is_empty() {
                    let t = Affine::translate(Vec2::new(x, pos.y)).then(Affine::scale(size, size));
                    batch.draw_mesh(&g.mesh, &t, &tint);
                }
                x += g.advance * size;
            });
        }
    }

    /// Wie [`Font::draw`], aber vertikal um `pos.y` zentriert (nach Großbuchstabenhöhe).
    pub fn draw_centered(
        &self,
        batch: &mut ShapeBatch,
        text: &str,
        pos: Vec2,
        size: f32,
        color: Color,
        align: Align,
    ) {
        let baseline = pos.y + self.cap_height() * size / 2.0;
        self.draw(batch, text, Vec2::new(pos.x, baseline), size, color, align);
    }
}

/// Glyphen-Umriss → lyon-Pfad in em, y nach unten.
struct Outline {
    builder: WithSvg<lyon::path::path::BuilderImpl>,
    scale: f32,
}

impl Outline {
    fn p(&self, x: f32, y: f32) -> lyon::math::Point {
        point(x * self.scale, -y * self.scale)
    }
}

impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.builder.move_to(p);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.builder.line_to(p);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (c, p) = (self.p(x1, y1), self.p(x, y));
        self.builder.quadratic_bezier_to(c, p);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (c1, c2, p) = (self.p(x1, y1), self.p(x2, y2), self.p(x, y));
        self.builder.cubic_bezier_to(c1, c2, p);
    }

    fn close(&mut self) {
        self.builder.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTER: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");

    #[test]
    fn glyphs_are_cached_and_measured() {
        let font = Font::new(INTER).unwrap();
        let w = font.width("12", 20.0);
        assert!(w > 15.0 && w < 30.0, "Breite {w}");
        assert!(font.width("", 20.0).abs() < f32::EPSILON);
        let mut batch = ShapeBatch::default();
        font.draw(
            &mut batch,
            "Elora 3:24",
            Vec2::default(),
            20.0,
            Color::rgb(1.0, 1.0, 1.0),
            Align::Center,
        );
        assert!(batch.triangle_count() > 50);
        // 1 2 E l o r a ␣ 3 : 4 – „2“ doppelt; das Leerzeichen hat keine Form, wird aber gecacht
        assert_eq!(font.glyphs.borrow().len(), 11);
        // Glyphen liegen über der Grundlinie (y < 0) und mittig um den Anker
        let (min, max) = batch_bounds(&batch);
        assert!(min.y < -10.0 && max.y <= 0.5, "{min:?} {max:?}");
        assert!((min.x + max.x).abs() < 3.0, "zentriert: {min:?} {max:?}");
    }

    fn batch_bounds(b: &ShapeBatch) -> (Vec2, Vec2) {
        let v = &b.geometry.vertices;
        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);
        for p in v {
            min = Vec2::new(min.x.min(p.pos[0]), min.y.min(p.pos[1]));
            max = Vec2::new(max.x.max(p.pos[0]), max.y.max(p.pos[1]));
        }
        (min, max)
    }
}
