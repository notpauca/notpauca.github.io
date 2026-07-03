use std::error::Error;
use crate::{consts, fetch};

pub struct Texture {
    image: image::RgbaImage,
    image_texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout
}

impl Texture {
    pub async fn new(device: &wgpu::Device) -> Result<Self, Box<dyn Error>> {
        let image_bytes = fetch::binary_data("pfp.png").await.unwrap();
        // let image_bytes = include_bytes!("../img/pfp.png");
        let image = image::load_from_memory(image_bytes.as_slice())?.to_rgba8();
        let size = image.dimensions();
        let image_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("pfp_image_texture"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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

        let bind_group_layout = device.create_bind_group_layout(&consts::bind_group_layouts::TEXTURE);

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

        Ok(Self {image, image_texture, view, sampler, bind_group, bind_group_layout})
    }

    pub fn write_itself(&self, queue: &wgpu::Queue) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.image_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.image.as_raw().as_slice(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4*self.image.width()),
                rows_per_image: Some(self.image.height())
            },
            self.image_texture.size()
        );
    }
}