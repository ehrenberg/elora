//! Benchmark for cached meshes (M5.1): 64 figures + 2000 particles per frame.
//!
//! `cargo run --release -p elora-render --example bench_meshes`

use std::time::Instant;

use elora_render::{Affine, Color, MeshBuilder, Paint, ShapeBatch, Tint, ellipse};
use elora_sim::Vec2;

fn main() {
    // Figure similar to draft B: body, belly, eyes, feet with strokes
    let outline = (5.0, Paint::Solid(Color::hex(0x2b2b2b)));
    let t0 = Instant::now();
    let figure = MeshBuilder::new(0.25)
        .fill_outlined(
            &ellipse(Vec2::new(-20.0, 59.0), Vec2::new(14.0, 8.0)),
            Paint::key(3),
            outline,
        )
        .fill_outlined(
            &ellipse(Vec2::new(22.0, 59.0), Vec2::new(14.0, 8.0)),
            Paint::key(3),
            outline,
        )
        .fill_outlined(
            &ellipse(Vec2::new(0.0, 0.0), Vec2::new(50.0, 58.0)),
            Paint::key(2),
            outline,
        )
        .fill_ellipse(
            Vec2::new(4.0, 40.0),
            Vec2::new(36.0, 12.0),
            Paint::key_shaded(2, 0.5),
        )
        .fill_ellipse(Vec2::new(2.0, 4.0), Vec2::new(6.5, 11.0), Paint::key(1))
        .fill_ellipse(Vec2::new(26.0, 4.0), Vec2::new(6.5, 11.0), Paint::key(1))
        .build();
    let particle = MeshBuilder::new(0.25)
        .fill_ellipse(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            Paint::Solid(Color::hex(0xcccccc)),
        )
        .build();
    println!(
        "Figur: {} Dreiecke, Partikel: {} Dreiecke, tesselliert in {:.2} ms",
        figure.triangle_count(),
        particle.triangle_count(),
        t0.elapsed().as_secs_f64() * 1000.0
    );

    let tint = Tint::new(vec![
        Color::hex(0x2b2b2b),
        Color::hex(0xf2c14e),
        Color::hex(0xd9a43a),
    ]);
    let mut batch = ShapeBatch::default();
    let frames = 1000;
    let t0 = Instant::now();
    for f in 0..frames {
        batch.clear();
        #[allow(clippy::cast_precision_loss)]
        let phase = f as f32 * 0.1;
        for i in 0..64 {
            #[allow(clippy::cast_precision_loss)]
            let x = i as f32 * 40.0;
            let squash = 1.0 + 0.2 * (phase + x).sin();
            let t = Affine::translate(Vec2::new(x, 100.0))
                .then(Affine::rotate(0.1 * phase.sin()))
                .then(Affine::scale(0.36 / squash, 0.36 * squash));
            batch.draw_mesh(&figure, &t, &tint);
        }
        for i in 0..2000 {
            #[allow(clippy::cast_precision_loss)]
            let p = Vec2::new((i % 50) as f32 * 30.0, (i / 50) as f32 * 20.0);
            batch.draw_mesh(
                &particle,
                &Affine::translate(p).then(Affine::scale(4.0, 4.0)),
                &Tint::default(),
            );
        }
    }
    let per_frame = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(frames);
    println!(
        "{} Dreiecke pro Frame, CPU-Aufbau {per_frame:.3} ms/Frame",
        batch.triangle_count()
    );
}
