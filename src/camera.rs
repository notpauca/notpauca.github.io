use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, Rad, Vector3};
use wgpu::util::DeviceExt;
use crate::consts;

pub struct Camera {
    pub stats: Stats,
    pub uniform: Uniform,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(aspect: f32, device: &wgpu::Device) -> Self {
        let stats = Stats::new(aspect);
        let uniform = Uniform::new(stats.calc_projection_matrix());

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera uniform buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&consts::CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        Self {
            stats,
            uniform,
            buffer,
            bind_group_layout,
            bind_group
        }
    }

    pub fn update_uniform(&mut self) {
        self.uniform = Uniform::new(self.stats.calc_projection_matrix())
    }

    pub fn resize(&mut self, aspect: f32) {
        self.stats.aspect = aspect;
        //Don't need to update uniform or write anything to the queue, because that's the RenderingStruct::render() function's job
    }
}

#[derive(Debug)]
pub struct Stats {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Stats {
    pub fn new(aspect: f32) -> Self {
         Self {
             position: (0.0, 0.0, 10.0).into(),
             yaw: Deg(-90.0).into(),
             pitch: Deg(0.0).into(),
             aspect,
             fov: 45.0,
             near: 0.1,
             far: 100.0
         }
    }

    pub fn calc_projection_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        let view = Matrix4::look_to_rh(
            self.position,
            Vector3::new(
                cos_pitch * cos_yaw,
                sin_pitch,
                cos_pitch * sin_yaw
            ).normalize(),
            Vector3::unit_y(),
        );

        let proj = perspective(Deg(self.fov), self.aspect, self.near, self.far);

        consts::OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Uniform {
    view_proj: [[f32; 4]; 4]
}

impl Uniform {
    pub fn new(mat: Matrix4<f32>) -> Self {
        Self {
            view_proj: mat.into()
        }
    }
}

