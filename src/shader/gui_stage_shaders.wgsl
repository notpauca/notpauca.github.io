@group(0) @binding(0)
var<uniform> time: f32;

@group(1) @binding(0)
var<uniform> screen_size: vec2<u32>;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(2) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) coords: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let coords = in.coords/vec2<f32>(screen_size)-1.0;
    out.clip_position = vec4(coords, 0.0, 1.0);
    out.uv = in.uv;
    out.uv.y = 1 - out.uv.y;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = vec3((sin(time/1000.0)+1.0)/2);
    let alpha = textureSample(t_diffuse, s_diffuse, in.uv).r;
    return vec4(color, alpha);
//    return vec4(1.0, 1.0, 1.0, 1.0);
}