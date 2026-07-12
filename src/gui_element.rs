use std::collections::LinkedList;
use cgmath::Vector2;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    coords: [f32; 2],
    uv: [f32; 2]
}

impl Vertex2D {
    const ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRS,
        }
    }
}

//Text to draw, position (0,0 at bottom-left), font size, font name (for looking up the font, will skip if no font is found)
pub type Unfinished = (&'static str, Vector2<f32>, f32, &'static str);

pub struct Finished {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    bytes: Vec<u8>,
    metrics: fontdue::Metrics,
    image_texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Finished {
    pub async fn from_unfinished(unfinished: Unfinished, device: &wgpu::Device, font: &fontdue::Font, glyph_bind_group_layout: &wgpu::BindGroupLayout) -> LinkedList<Self> {
        let mut x_pos = unfinished.1.x;
        let y_pos = unfinished.1.y;
        let size = unfinished.2;
        let mut ret = LinkedList::new();
        for character in unfinished.0.chars() {
            if character.is_whitespace() {
               x_pos+=size/2.0;
                continue;
            }
            let (metrics, bytes) = font.rasterize(character, size);
            let relative_xpos = x_pos + metrics.xmin as f32;
            let relative_ypos = y_pos + metrics.ymin as f32;

            let vertices = [
                Vertex2D {coords: [relative_xpos, relative_ypos], uv: [0.0, 0.0]},
                Vertex2D {coords: [relative_xpos + metrics.width as f32, relative_ypos], uv: [1.0, 0.0]},
                Vertex2D {coords: [relative_xpos + metrics.width as f32, relative_ypos + metrics.height as f32], uv: [1.0, 1.0]},
                Vertex2D {coords: [relative_xpos, relative_ypos + metrics.height as f32], uv: [0.0, 1.0]}
            ];

            x_pos+= metrics.advance_width;

            let indices: [[u16; 3]; 2] = [
                [0, 1, 2],
                [0, 2, 3]
            ];

            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );

            let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );

            let image_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("glyph_texture"),
                size: wgpu::Extent3d {
                    width: metrics.width as u32,
                    height: metrics.height as u32,
                    depth_or_array_layers: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            // let texture_view = image_texture.create_view(&Default::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                ..Default::default()
            });

            let view = image_texture.create_view(&Default::default());

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &glyph_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some("texture_bind_group")
            });

            ret.push_back(
                Self {
                    vertex_buffer,
                    index_buffer,
                    bytes,
                    metrics,
                    image_texture,
                    bind_group
                }
            );
        }
        ret
    }

    pub fn write_itself(&self, queue: &wgpu::Queue) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.image_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.metrics.width as u32),
                rows_per_image: Some(self.metrics.height as u32)
            },
            self.image_texture.size()
        );
    }
}