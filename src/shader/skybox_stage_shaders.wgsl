// essentially stolen from https://github.com/sotrh/learn-wgpu/blob/master/code/intermediate/tutorial13-hdr/src/sky.wgsl
// i'm too lazy to mess around with the math used here
// TODO: have a normal camera struct and stuff, not a different implementation for each render pass.

struct Camera {
    inv_view: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> time: f32;

@group(2) @binding(0)
var sky_texture: texture_cube<f32>;

@group(2) @binding(1)
var sky_sampler: sampler;

struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    @location(0) clip_position: vec4<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u,
    ));
    var out: VertexOutput;
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        let view_pos_homogeneous = camera.inv_proj * in.clip_position;
        let view_ray_direction = view_pos_homogeneous.xyz / view_pos_homogeneous.w;
        var ray_direction = normalize((camera.inv_view * vec4(view_ray_direction, 0.0)).xyz);

        let sample = textureSample(sky_texture, sky_sampler, ray_direction);
        return sample;

}