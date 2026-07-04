pub mod bind_group_layouts;

pub const CANVAS_ID: &'static str = "canvas";
pub const MESH_SHADER_PATH: &'static str = "src/shader/mesh_stage_shaders.wgsl";
pub const GUI_SHADER_PATH: &'static str = "src/shader/gui_stage_shaders.wgsl";
pub const SKYBOX_SHADER_PATH: &'static str = "src/shader/skybox_stage_shaders.wgsl";
pub const POSTPROC_SHADER_PATH: &'static str = "src/shader/postproc_stage_shaders.wgsl";

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

pub const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

pub const ASSETS_DIRECTORY: &'static str = "assets/";