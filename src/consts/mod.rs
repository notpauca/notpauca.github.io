pub mod bind_group_layouts;

pub use bind_group_layouts::CAMERA as CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR;
pub use bind_group_layouts::TIME as TIME_BIND_GROUP_LAYOUT_DESCRIPTOR;
pub use bind_group_layouts::MESH_TRANSFORM as MESH_TRANSFORM_BIND_GROUP_LAYOUT_DESCRIPTOR;
pub use bind_group_layouts::TEXTURE as TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR;


pub const CANVAS_ID: &'static str = "canvas";
pub const SHADER_FILE_PATH: &'static str = "src/shaders.wgsl";

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

pub const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

pub const ASSETS_DIRECTORY: &'static str = "assets/";