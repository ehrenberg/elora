//! Vektorformen, zur Laufzeit per lyon tesselliert (E-033).

use elora_sim::Vec2;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

/// RGBA-Farbe, Komponenten 0..=1 (sRGB).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub [f32; 4]);

impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self([r, g, b, 1.0])
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }

    /// Aus 0xRRGGBB.
    pub const fn hex(v: u32) -> Self {
        Self::rgb(
            ((v >> 16) & 0xff) as f32 / 255.0,
            ((v >> 8) & 0xff) as f32 / 255.0,
            (v & 0xff) as f32 / 255.0,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
}

/// Sammelt Formen eines Frames in Weltkoordinaten.
pub struct ShapeBatch {
    pub(crate) geometry: VertexBuffers<Vertex, u32>,
    fill: FillTessellator,
    stroke: StrokeTessellator,
    /// Toleranz der Tessellierung in Welteinheiten (kleiner = feiner).
    pub tolerance: f32,
}

impl std::fmt::Debug for ShapeBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShapeBatch")
            .field("vertices", &self.geometry.vertices.len())
            .field("indices", &self.geometry.indices.len())
            .finish_non_exhaustive()
    }
}

impl Default for ShapeBatch {
    fn default() -> Self {
        Self {
            geometry: VertexBuffers::new(),
            fill: FillTessellator::new(),
            stroke: StrokeTessellator::new(),
            tolerance: 0.25,
        }
    }
}

impl ShapeBatch {
    pub fn clear(&mut self) {
        self.geometry.vertices.clear();
        self.geometry.indices.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.geometry.indices.is_empty()
    }

    /// Achsenparalleles Rechteck ohne Tessellierung (für viele Tiles).
    ///
    /// # Panics
    /// Bei mehr als `u32::MAX` Vertices in einem Batch.
    pub fn fill_rect(&mut self, min: Vec2, max: Vec2, color: Color) {
        let base = u32::try_from(self.geometry.vertices.len()).expect("zu viele Vertices");
        let c = color.0;
        self.geometry.vertices.extend_from_slice(&[
            Vertex {
                pos: [min.x, min.y],
                color: c,
            },
            Vertex {
                pos: [max.x, min.y],
                color: c,
            },
            Vertex {
                pos: [max.x, max.y],
                color: c,
            },
            Vertex {
                pos: [min.x, max.y],
                color: c,
            },
        ]);
        self.geometry.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }

    pub fn fill_circle(&mut self, center: Vec2, radius: f32, color: Color) {
        let options = FillOptions::tolerance(self.tolerance);
        let c = color.0;
        // Tessellierung einfacher Formen kann nicht fehlschlagen
        let _ = self.fill.tessellate_circle(
            point(center.x, center.y),
            radius,
            &options,
            &mut BuffersBuilder::new(&mut self.geometry, |v: FillVertex| Vertex {
                pos: v.position().to_array(),
                color: c,
            }),
        );
    }

    /// Geschlossenes, gefülltes Polygon.
    pub fn fill_polygon(&mut self, points: &[Vec2], color: Color) {
        let Some(path) = polyline(points, true) else {
            return;
        };
        let c = color.0;
        let _ = self.fill.tessellate_path(
            &path,
            &FillOptions::tolerance(self.tolerance),
            &mut BuffersBuilder::new(&mut self.geometry, |v: FillVertex| Vertex {
                pos: v.position().to_array(),
                color: c,
            }),
        );
    }

    /// Linienzug mit runden Enden.
    pub fn stroke_polyline(&mut self, points: &[Vec2], width: f32, color: Color) {
        let Some(path) = polyline(points, false) else {
            return;
        };
        let c = color.0;
        let options = StrokeOptions::tolerance(self.tolerance)
            .with_line_width(width)
            .with_line_cap(lyon::tessellation::LineCap::Round);
        let _ = self.stroke.tessellate_path(
            &path,
            &options,
            &mut BuffersBuilder::new(&mut self.geometry, |v: StrokeVertex| Vertex {
                pos: v.position().to_array(),
                color: c,
            }),
        );
    }

    /// Gecachtes Mesh transformiert und eingefärbt anhängen.
    ///
    /// # Panics
    /// Bei mehr als `u32::MAX` Vertices in einem Batch.
    pub fn draw_mesh(&mut self, mesh: &crate::Mesh, transform: &crate::Affine, tint: &crate::Tint) {
        mesh.emit(transform, tint, &mut self.geometry);
    }

    /// Dreiecke als SVG im Weltausschnitt `min..max` – zur Sichtprüfung ohne Fenster
    /// (z. B. mit `cargo xtask svg-preview` rastern).
    pub fn debug_svg(&self, min: Vec2, max: Vec2, background: Color) -> String {
        use std::fmt::Write as _;
        let size = max - min;
        let hex = |c: [f32; 4]| {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            // auf 0..=255 begrenzt
            let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            format!("#{:02x}{:02x}{:02x}", b(c[0]), b(c[1]), b(c[2]))
        };
        let mut out = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" shape-rendering="crispEdges">"#,
            min.x, min.y, size.x, size.y
        );
        let _ = write!(
            out,
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
            min.x,
            min.y,
            size.x,
            size.y,
            hex(background.0)
        );
        let v = &self.geometry.vertices;
        for tri in self.geometry.indices.as_chunks::<3>().0 {
            let [a, b, c] = tri.map(|i| v[i as usize]);
            let _ = write!(
                out,
                r#"<path d="M{},{}L{},{}L{},{}Z" fill="{}" fill-opacity="{:.3}" stroke="{}" stroke-opacity="{:.3}" stroke-width="0.15"/>"#,
                a.pos[0],
                a.pos[1],
                b.pos[0],
                b.pos[1],
                c.pos[0],
                c.pos[1],
                hex(a.color),
                a.color[3],
                hex(a.color),
                a.color[3]
            );
        }
        out.push_str("</svg>");
        out
    }

    /// Anzahl Dreiecke im Batch (Statistik).
    pub fn triangle_count(&self) -> usize {
        self.geometry.indices.len() / 3
    }

    pub fn stroke_line(&mut self, from: Vec2, to: Vec2, width: f32, color: Color) {
        self.stroke_polyline(&[from, to], width, color);
    }

    pub fn stroke_circle(&mut self, center: Vec2, radius: f32, width: f32, color: Color) {
        let c = color.0;
        let _ = self.stroke.tessellate_circle(
            point(center.x, center.y),
            radius,
            &StrokeOptions::tolerance(self.tolerance).with_line_width(width),
            &mut BuffersBuilder::new(&mut self.geometry, |v: StrokeVertex| Vertex {
                pos: v.position().to_array(),
                color: c,
            }),
        );
    }
}

fn polyline(points: &[Vec2], close: bool) -> Option<Path> {
    let (first, rest) = points.split_first()?;
    let mut b = Path::builder();
    b.begin(point(first.x, first.y));
    for p in rest {
        b.line_to(point(p.x, p.y));
    }
    b.end(close);
    Some(b.build())
}
