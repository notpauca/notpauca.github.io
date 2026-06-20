struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct TimeUniform {
    time: f32
};
@group(1) @binding(0)
var<uniform> time: TimeUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

fn srgbToLinear(color: vec4<f32>) -> vec4<f32> {
  var res = pow(color, vec4(1.0 / 2.2));
  res.w = color.w;
  return res;
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = srgbToLinear(model.color);
    var new_pos = model.position;
    new_pos.x += time.time/10000.0;
    out.clip_position = camera.view_proj * vec4<f32>(new_pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
