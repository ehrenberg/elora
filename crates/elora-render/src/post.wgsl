// Nachbearbeitung der Welt: Hitzeflimmern (R2-M2.3, E-320) und Sättigung (Farbe kehrt mit
// den Quellen zurück, E-328).
//
// Die Welt liegt als Textur vor; jede Bildzeile wird leicht seitlich verschoben, die Wellen
// steigen langsam auf und sind am unteren Bildrand (heißer Boden) am stärksten.

struct Post {
    // x: Zeit in Sekunden, y: Stärke des Flimmerns 0..1, z: Seitenverhältnis (Breite / Höhe),
    // w: Sättigung (1 = unverändert, 0 = grau)
    params: vec4<f32>,
    // Wetter (R2-W1): Tönung (rgb, Anteil), Nebel (rgb, Dichte), x: Abdunkeln, y: Blitz
    tint: vec4<f32>,
    fog: vec4<f32>,
    extra: vec4<f32>,
};

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var<uniform> post: Post;

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Ein Dreieck, das den ganzen Bildschirm bedeckt.
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOut {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var out: VertexOut;
    out.clip = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let t = post.params.x;
    let strength = post.params.y;
    let aspect = post.params.z;
    let uv = in.uv;
    // oben schwächer, unten (Boden) kräftig
    let falloff = mix(0.3, 1.0, smoothstep(0.0, 1.0, uv.y));
    // zwei überlagerte Wellen, deren Phase mit der Zeit nach oben wandert
    let wobble = sin(uv.x * 7.0 * aspect + t * 0.9) * 1.6;
    let w1 = sin(uv.y * 85.0 + t * 4.2 + wobble);
    let w2 = sin(uv.y * 37.0 + t * 2.6 - uv.x * 5.0 * aspect);
    let dx = (w1 * 0.7 + w2 * 0.3) * 0.0022 / aspect;
    let dy = sin(uv.x * 46.0 * aspect + t * 3.1) * 0.0009;
    let offset = vec2<f32>(dx, dy) * strength * falloff;
    let color = textureSample(scene, scene_sampler, clamp(uv + offset, vec2<f32>(0.0), vec2<f32>(1.0)));
    // ganz leicht warm getönt
    let warm = color.rgb * vec3<f32>(1.03, 1.0, 0.93);
    let tinted = mix(color.rgb, warm, strength * 0.6);
    // Sättigung: Grauwert nach Helligkeit, dann zurück zur Farbe
    let gray = dot(tinted, vec3<f32>(0.299, 0.587, 0.114));
    var rgb = mix(vec3<f32>(gray), tinted, post.params.w);
    // Wetter: Tönung, Abdunkeln, Nebel nach unten dichter, Blitz hellt alles auf
    rgb = mix(rgb, rgb * post.tint.rgb, post.tint.a);
    rgb = rgb * (1.0 - post.extra.x);
    let fog = clamp(post.fog.a * mix(0.35, 1.0, uv.y), 0.0, 1.0);
    rgb = mix(rgb, post.fog.rgb, fog);
    rgb = min(rgb + vec3<f32>(post.extra.y * 0.55), vec3<f32>(1.0));
    return vec4<f32>(rgb, color.a);
}
