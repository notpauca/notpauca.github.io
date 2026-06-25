use wgpu::util::DeviceExt;
use crate::consts;

pub struct Time {
    pub uniform: Uniform,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup
}

impl Time {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = Uniform {time: 0f32};
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&consts::TIME_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("time_bind_group"),
        });

        Self {
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn advance(&mut self, dt: f32) {
        self.uniform.time+=dt;
    }

    pub fn write_itself(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Uniform {
    pub time: f32
}