//! Gecachte Formen (M5.1): einmal tesselliert, oft gezeichnet.
//!
//! Ein [`Mesh`] liegt in lokalen Koordinaten vor. Beim Zeichnen wird es per
//! [`Affine`] verschoben, gedreht, skaliert oder verformt (Squash & Stretch) und
//! über eine [`Tint`] eingefärbt: Ecken mit Farbschlüssel bekommen die Farbe des
//! Schlüssels, optional aufgehellt oder abgedunkelt (z. B. Bauchfleck, E-097).

use elora_sim::Vec2;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, LineCap, LineJoin, StrokeOptions,
    StrokeTessellator, StrokeVertex, VertexBuffers,
};

use crate::Color;

/// Affine 2D-Transformation: `p' = x_axis * p.x + y_axis * p.y + offset`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    pub x_axis: Vec2,
    pub y_axis: Vec2,
    pub offset: Vec2,
}

impl Default for Affine {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Affine {
    pub const IDENTITY: Self = Self {
        x_axis: Vec2::new(1.0, 0.0),
        y_axis: Vec2::new(0.0, 1.0),
        offset: Vec2::new(0.0, 0.0),
    };

    pub const fn translate(offset: Vec2) -> Self {
        Self {
            offset,
            ..Self::IDENTITY
        }
    }

    pub const fn scale(sx: f32, sy: f32) -> Self {
        Self {
            x_axis: Vec2::new(sx, 0.0),
            y_axis: Vec2::new(0.0, sy),
            offset: Vec2::new(0.0, 0.0),
        }
    }

    /// Drehung um den Ursprung (Bogenmaß; positiv = im Uhrzeigersinn, da y nach unten zeigt).
    pub fn rotate(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            x_axis: Vec2::new(c, s),
            y_axis: Vec2::new(-s, c),
            offset: Vec2::new(0.0, 0.0),
        }
    }

    /// Erst `inner`, dann `self` anwenden.
    #[must_use]
    pub fn then(self, inner: Self) -> Self {
        Self {
            x_axis: self.apply_vector(inner.x_axis),
            y_axis: self.apply_vector(inner.y_axis),
            offset: self.apply(inner.offset),
        }
    }

    pub fn apply(&self, p: Vec2) -> Vec2 {
        self.apply_vector(p) + self.offset
    }

    pub fn apply_vector(&self, v: Vec2) -> Vec2 {
        self.x_axis * v.x + self.y_axis * v.y
    }
}

/// Wie eine Form gefüllt wird.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paint {
    /// Feste Farbe.
    Solid(Color),
    /// Linearer Verlauf von `from` (Farbe `a`) nach `to` (Farbe `b`), lokale Koordinaten.
    Linear {
        from: Vec2,
        to: Vec2,
        a: Color,
        b: Color,
    },
    /// Farbe aus der [`Tint`] beim Zeichnen; `shade` > 0 hellt auf, < 0 dunkelt ab (−1..=1).
    Key { slot: u8, shade: f32, alpha: f32 },
}

impl Paint {
    pub const fn key(slot: u8) -> Self {
        Self::Key {
            slot,
            shade: 0.0,
            alpha: 1.0,
        }
    }

    pub const fn key_shaded(slot: u8, shade: f32) -> Self {
        Self::Key {
            slot,
            shade,
            alpha: 1.0,
        }
    }

    fn vertex(&self, pos: [f32; 2]) -> MeshVertex {
        let (color, slot, shade) = match *self {
            Self::Solid(c) => (c.0, 0, 0.0),
            Self::Linear { from, to, a, b } => {
                let d = to - from;
                let len2 = d.x * d.x + d.y * d.y;
                let t = if len2 > 0.0 {
                    (((pos[0] - from.x) * d.x + (pos[1] - from.y) * d.y) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (lerp_color(a, b, t).0, 0, 0.0)
            }
            Self::Key { slot, shade, alpha } => ([0.0, 0.0, 0.0, alpha], slot, shade),
        };
        MeshVertex {
            pos,
            color,
            slot,
            shade,
        }
    }
}

/// Farben für die Schlüssel eines Meshes. Schlüssel 0 bleibt unbenutzt (feste Farbe);
/// fehlende Schlüssel werden magenta gezeichnet, damit sie auffallen.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tint {
    pub colors: Vec<Color>,
    /// Deckkraft für das ganze Mesh (Ausblenden, Unsichtbarkeit).
    pub alpha: Option<f32>,
}

impl Tint {
    pub fn new(colors: impl Into<Vec<Color>>) -> Self {
        Self {
            colors: colors.into(),
            alpha: None,
        }
    }

    fn resolve(&self, v: &MeshVertex) -> [f32; 4] {
        let mut c = if v.slot == 0 {
            v.color
        } else {
            let base = self
                .colors
                .get(usize::from(v.slot - 1))
                .copied()
                .unwrap_or(Color::rgb(1.0, 0.0, 1.0));
            let mut c = shade(base, v.shade).0;
            c[3] *= v.color[3];
            c
        };
        if let Some(a) = self.alpha {
            c[3] *= a;
        }
        c
    }
}

/// Hellt (`amount` > 0, Richtung Weiß) oder dunkelt (< 0, Richtung Schwarz) eine Farbe ab.
pub fn shade(c: Color, amount: f32) -> Color {
    if amount >= 0.0 {
        lerp_color(c, Color::rgba(1.0, 1.0, 1.0, c.0[3]), amount.min(1.0))
    } else {
        lerp_color(c, Color::rgba(0.0, 0.0, 0.0, c.0[3]), (-amount).min(1.0))
    }
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let mut out = [0.0; 4];
    for (i, o) in out.iter_mut().enumerate() {
        *o = a.0[i] + (b.0[i] - a.0[i]) * t;
    }
    Color(out)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MeshVertex {
    pub pos: [f32; 2],
    /// Feste Farbe, bei Schlüssel nur die Deckkraft.
    pub color: [f32; 4],
    /// 0 = feste Farbe, sonst Schlüssel (1-basiert in [`Tint::colors`]).
    pub slot: u8,
    pub shade: f32,
}

/// Fertig tessellierte Form in lokalen Koordinaten.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub(crate) vertices: Vec<MeshVertex>,
    pub(crate) indices: Vec<u32>,
}

impl Mesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Umschließendes Rechteck (min, max) in lokalen Koordinaten.
    pub fn bounds(&self) -> Option<(Vec2, Vec2)> {
        let first = self.vertices.first()?;
        let mut min = Vec2::new(first.pos[0], first.pos[1]);
        let mut max = min;
        for v in &self.vertices {
            min = Vec2::new(min.x.min(v.pos[0]), min.y.min(v.pos[1]));
            max = Vec2::new(max.x.max(v.pos[0]), max.y.max(v.pos[1]));
        }
        Some((min, max))
    }

    /// Ecken transformiert und eingefärbt an `out` anhängen.
    pub(crate) fn emit(
        &self,
        transform: &Affine,
        tint: &Tint,
        out: &mut VertexBuffers<crate::shapes::Vertex, u32>,
    ) {
        let base = u32::try_from(out.vertices.len()).expect("zu viele Vertices");
        out.vertices.extend(self.vertices.iter().map(|v| {
            let p = transform.apply(Vec2::new(v.pos[0], v.pos[1]));
            crate::shapes::Vertex {
                pos: [p.x, p.y],
                color: tint.resolve(v),
            }
        }));
        out.indices.extend(self.indices.iter().map(|i| base + i));
    }
}

/// Baut ein [`Mesh`] aus Pfaden auf; spätere Formen liegen über früheren.
pub struct MeshBuilder {
    geometry: VertexBuffers<MeshVertex, u32>,
    fill: FillTessellator,
    stroke: StrokeTessellator,
    tolerance: f32,
}

impl std::fmt::Debug for MeshBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshBuilder")
            .field("vertices", &self.geometry.vertices.len())
            .field("tolerance", &self.tolerance)
            .finish_non_exhaustive()
    }
}

impl MeshBuilder {
    /// `tolerance`: maximale Abweichung von der Kurve in lokalen Einheiten.
    pub fn new(tolerance: f32) -> Self {
        Self {
            geometry: VertexBuffers::new(),
            fill: FillTessellator::new(),
            stroke: StrokeTessellator::new(),
            tolerance,
        }
    }

    pub fn fill_path(&mut self, path: &Path, paint: Paint) -> &mut Self {
        let _ = self.fill.tessellate_path(
            path,
            &FillOptions::tolerance(self.tolerance),
            &mut BuffersBuilder::new(&mut self.geometry, |v: FillVertex| {
                paint.vertex(v.position().to_array())
            }),
        );
        self
    }

    /// Kontur mit runden Ecken und Enden.
    pub fn stroke_path(&mut self, path: &Path, width: f32, paint: Paint) -> &mut Self {
        let options = StrokeOptions::tolerance(self.tolerance)
            .with_line_width(width)
            .with_line_join(LineJoin::Round)
            .with_line_cap(LineCap::Round);
        let _ = self.stroke.tessellate_path(
            path,
            &options,
            &mut BuffersBuilder::new(&mut self.geometry, |v: StrokeVertex| {
                paint.vertex(v.position().to_array())
            }),
        );
        self
    }

    /// Gefüllt mit Kontur darüber (`outline` = Breite, Farbe).
    pub fn fill_outlined(&mut self, path: &Path, fill: Paint, outline: (f32, Paint)) -> &mut Self {
        self.fill_path(path, fill)
            .stroke_path(path, outline.0, outline.1)
    }

    pub fn fill_ellipse(&mut self, center: Vec2, radii: Vec2, paint: Paint) -> &mut Self {
        self.fill_path(&ellipse(center, radii), paint)
    }

    pub fn build(&mut self) -> Mesh {
        let g = std::mem::replace(&mut self.geometry, VertexBuffers::new());
        Mesh {
            vertices: g.vertices,
            indices: g.indices,
        }
    }
}

/// Ellipse als Pfad.
pub fn ellipse(center: Vec2, radii: Vec2) -> Path {
    let mut b = Path::builder();
    b.add_ellipse(
        point(center.x, center.y),
        lyon::math::vector(radii.x, radii.y),
        lyon::math::Angle::zero(),
        lyon::path::Winding::Positive,
    );
    b.build()
}

/// Rechteck mit abgerundeten Ecken als Pfad.
pub fn rounded_rect(min: Vec2, max: Vec2, radius: f32) -> Path {
    let mut b = Path::builder();
    b.add_rounded_rectangle(
        &lyon::math::Box2D::new(point(min.x, min.y), point(max.x, max.y)),
        &lyon::path::builder::BorderRadii::new(radius),
        lyon::path::Winding::Positive,
    );
    b.build()
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // exakte Werte bei 0, 0.5 und 1
mod tests {
    use super::*;

    fn close(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 1e-4
    }

    #[test]
    fn affine_composition() {
        let t = Affine::translate(Vec2::new(10.0, 0.0))
            .then(Affine::rotate(std::f32::consts::FRAC_PI_2))
            .then(Affine::scale(2.0, 1.0));
        // (1,0) → skaliert (2,0) → gedreht (0,2) → verschoben (10,2)
        assert!(close(t.apply(Vec2::new(1.0, 0.0)), Vec2::new(10.0, 2.0)));
        assert!(close(
            Affine::IDENTITY.apply(Vec2::new(3.0, 4.0)),
            Vec2::new(3.0, 4.0)
        ));
    }

    #[test]
    fn tint_and_shade() {
        let mut b = MeshBuilder::new(0.5);
        b.fill_ellipse(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0), Paint::key(1));
        b.fill_ellipse(
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Paint::key_shaded(1, 0.5),
        );
        let mesh = b.build();
        assert!(mesh.triangle_count() > 10);
        let tint = Tint::new(vec![Color::rgb(0.0, 0.0, 1.0)]);
        let mut out = VertexBuffers::new();
        mesh.emit(&Affine::IDENTITY, &tint, &mut out);
        assert!(out.vertices.iter().any(|v| v.color == [0.0, 0.0, 1.0, 1.0]));
        assert!(out.vertices.iter().any(|v| v.color == [0.5, 0.5, 1.0, 1.0]));
        // fehlender Schlüssel fällt auf
        let mut out = VertexBuffers::new();
        mesh.emit(&Affine::IDENTITY, &Tint::default(), &mut out);
        assert_eq!(out.vertices[0].color, [1.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn linear_gradient_endpoints() {
        let p = Paint::Linear {
            from: Vec2::new(0.0, 0.0),
            to: Vec2::new(0.0, 10.0),
            a: Color::rgb(0.0, 0.0, 0.0),
            b: Color::rgb(1.0, 1.0, 1.0),
        };
        assert_eq!(p.vertex([0.0, -5.0]).color, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(p.vertex([3.0, 5.0]).color, [0.5, 0.5, 0.5, 1.0]);
        assert_eq!(p.vertex([0.0, 20.0]).color, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn transform_moves_bounds() {
        let mesh = MeshBuilder::new(0.25)
            .fill_path(
                &rounded_rect(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0), 0.2),
                Paint::Solid(Color::rgb(1.0, 1.0, 1.0)),
            )
            .build();
        let mut out = VertexBuffers::new();
        let t = Affine::translate(Vec2::new(100.0, 50.0)).then(Affine::scale(3.0, 0.5));
        mesh.emit(&t, &Tint::default(), &mut out);
        let xs = out.vertices.iter().map(|v| v.pos[0]);
        let (lo, hi) = xs.fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        assert!((lo - 97.0).abs() < 1e-3 && (hi - 103.0).abs() < 1e-3);
        assert_eq!(out.indices.len(), mesh.indices.len());
    }
}
