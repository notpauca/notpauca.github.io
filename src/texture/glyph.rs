use std::error::Error;
use crate::{consts, fetch};

pub struct Glyph {
    bytes: Vec<u8>,
    metrics: fontdue::Metrics,
    image_texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Glyph {
    pub fn new(device: &wgpu::Device, metrics: fontdue::Metrics, bitmap: Vec<u8>, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let size = (metrics.width as u32, metrics.height as u32);
        let image_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph_texture"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        } );

        // let texture_view = image_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let view = image_texture.create_view(&Default::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
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

        Self {bytes: bitmap, metrics, image_texture, view, sampler, bind_group}
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