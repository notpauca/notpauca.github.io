struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct TimeUniform {
    time: f32,
};
@group(1) @binding(0)
var<uniform> time: TimeUniform;

struct MeshTranslationUniform {
    translation_matrix: mat4x4<f32>,
};
@group(2) @binding(0)
var<uniform> mesh_translation: MeshTranslationUniform;

@group(3) @binding(0)
var text: texture_2d<f32>;

@group(3) @binding(1)
var sampl: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.coords = model.coords;
    out.coords.y = 1-out.coords.y;
    var new_pos = model.position;
    out.clip_position = camera.view_proj * (mesh_translation.translation_matrix * vec4<f32>(new_pos, 1.0));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(text, sampl, in.coords);
}