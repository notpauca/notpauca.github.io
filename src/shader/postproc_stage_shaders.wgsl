@group(0) @binding(0)
var<uniform> time: f32;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(1) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var<uniform> screen_size: vec2<u32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2(-1.0, -3.0),
        vec2( 3.0,  1.0),
        vec2(-1.0,  1.0),
    );
    var out : VertexOutput;
    out.position = vec4(positions[id], 0.0, 1.0);
    out.uv.x = out.position.x * 0.5 + 0.5;
    out.uv.y = 1.0 - (out.position.y * 0.5 + 0.5);
    return out;
}

fn barrel_distort(uv: vec2<f32>) -> vec2<f32> {
    let p = uv * 2.0 - 1.0;
    let r2 = dot(p, p);
    let k = 0.3;
    let warped = p * (1.0 + k * r2);
    return warped * 0.5 + 0.5;
}

fn chromatic_aberration(uv: vec2<f32>) -> vec4<f32> {
    let chromatic_offset = vec2(0.015, 0.0);
    let r = textureSample(t_diffuse, s_diffuse, uv + chromatic_offset).r;
    let g = textureSample(t_diffuse, s_diffuse, uv).g;
    let b = textureSample(t_diffuse, s_diffuse, uv - chromatic_offset).b;
    return vec4(r, g, b, 1.0);
}

fn scanline(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let scanline = 0.92 + 0.08 * sin(uv.y * f32(screen_size.y/4) * 3.14159);
    return color*scanline;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//    for testing i can just disable the effects and return the sampled color
//    return textureSample(t_diffuse, s_diffuse, in.uv);
//    for prod
    let uv = barrel_distort(in.uv);
    var color = chromatic_aberration(uv);
    color = scanline(color, uv);
    if (any(uv < vec2(0.0)) || any(uv > vec2(1.0))) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    return color;
}