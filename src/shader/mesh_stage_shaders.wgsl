@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> time: f32;

@group(2) @binding(0)
var<uniform> mesh_translation: mat4x4<f32>;

@group(3) @binding(0)
var texture: texture_2d<f32>;

@group(3) @binding(1)
var t_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) coords: vec2<f32>,
    @location(2) normals: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>,
    @location(1) normals: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.coords = input.coords;
    out.coords.y = 1-out.coords.y;
    var new_pos = input.position;
    out.clip_position = view_proj * (mesh_translation * vec4<f32>(new_pos, 1.0));
    out.normals = input.normals;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, t_sampler, in.coords);
}