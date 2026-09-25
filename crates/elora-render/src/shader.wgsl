// Farbige 2D-Dreiecke in Weltkoordinaten.

struct View {
    // x, y: obere linke Ecke; z, w: Größe (Welteinheiten)
    rect: vec4<f32>,
};

@group(0) @binding(0) var<uniform> view: View;

struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    let rel = (in.pos - view.rect.xy) / view.rect.zw;
    var out: VertexOut;
    out.clip = vec4<f32>(rel.x * 2.0 - 1.0, 1.0 - rel.y * 2.0, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
