// Full-screen picture (adventure intro video, I-2): the image is fitted into the screen
// keeping its aspect ratio; the rest stays black (letterbox).

struct Fit {
    // xy: size of the picture on screen in clip units (1 = whole screen half), zw: unused
    scale: vec4<f32>,
};

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var picture_sampler: sampler;
@group(0) @binding(2) var<uniform> fit: Fit;

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Two triangles forming the fitted rectangle.
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let uv = corners[i];
    var out: VertexOut;
    out.clip = vec4<f32>((uv.x * 2.0 - 1.0) * fit.scale.x, (1.0 - uv.y * 2.0) * fit.scale.y, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(picture, picture_sampler, in.uv).rgb, 1.0);
}
