@group(0) @binding(0)
var<uniform> time: f32;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u
    ));
    var out: VertexOutput;
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.uv = uv * 4.0 - 1.0;
    out.uv.y = 1 - out.uv.y;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//    var res = in.color;
//    res.w = (sin(time/500)/2.0+0.5)/2.0+0.25;
//    return res;
    let alpha = textureSample(t_diffuse, s_diffuse, in.uv).r;
    return vec4(1.0, 1.0, 1.0, alpha);
}