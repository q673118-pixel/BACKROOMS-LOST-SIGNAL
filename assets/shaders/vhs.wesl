import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct VhsPostProcessSettings {
    intensity: f32,
    scanlines: f32,
    noise: f32,
    chromatic: f32,
    distortion: f32,
    flicker: f32,
    time: f32,
    enabled: f32,
}
@group(0) @binding(2) var<uniform> settings: VhsPostProcessSettings;

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    let amount = clamp(settings.intensity, 0.0, 1.0) * settings.enabled;

    let line_wave = sin(uv.y * 95.0 + settings.time * 2.1);
    uv.x += line_wave * settings.distortion * amount * 0.004;

    let shift = settings.chromatic * amount * 0.003;
    let red = textureSample(screen_texture, texture_sampler, uv + vec2<f32>(shift, 0.0)).r;
    let green = textureSample(screen_texture, texture_sampler, uv).g;
    let blue = textureSample(screen_texture, texture_sampler, uv - vec2<f32>(shift, 0.0)).b;
    var color = vec3<f32>(red, green, blue);

    let scan = 0.5 + 0.5 * sin(uv.y * 1100.0);
    color *= 1.0 - scan * settings.scanlines * amount * 0.24;

    let cell = floor(uv * vec2<f32>(900.0, 500.0) + vec2<f32>(settings.time * 37.0, settings.time * 19.0));
    let grain = hash21(cell) - 0.5;
    color += grain * settings.noise * amount * 0.13;

    let flick = abs(sin(settings.time * 17.0)) * settings.flicker * amount * 0.08;
    color *= 1.0 - flick;

    let vignette = smoothstep(0.95, 0.25, distance(uv, vec2<f32>(0.5, 0.5)));
    color *= 0.88 + vignette * 0.12;

    return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
