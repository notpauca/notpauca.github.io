use itertools::Itertools;
use crate::{consts, fetch};

pub struct Skybox {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    images: [image::RgbaImage; 6],
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout
}

impl Skybox {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self, wasm_bindgen::JsValue> {
        let skybox_image_nx = fetch::binary_data("sky_97_2k/sky_97_cubemap_2k/nx.png").await?;
        let skybox_image_ny = fetch::binary_data("sky_97_2k/sky_97_cubemap_2k/ny.png").await?;
        let skybox_image_nz = fetch::binary_data("sky_97_2k/sky_97_cubemap_2k/nz.png").await?;
        let skybox_image_px = fetch::binary_data("sky_97_2k/sky_97_cubemap_2k/px.png").await?;
        let skybox_image_py = fetch::binary_data("sky_97_2k/sky_97_cubemap_2k/py.png").await?;
        let skybox_image_pz = fetch::binary_data("sky_97_2k/sky_97_cubemap_2k/pz.png").await?;

        let images = [
            image::load_from_memory(skybox_image_px.as_slice()).unwrap().to_rgba8(),
            image::load_from_memory(skybox_image_nx.as_slice()).unwrap().to_rgba8(),
            image::load_from_memory(skybox_image_py.as_slice()).unwrap().to_rgba8(),
            image::load_from_memory(skybox_image_ny.as_slice()).unwrap().to_rgba8(),
            image::load_from_memory(skybox_image_pz.as_slice()).unwrap().to_rgba8(),
            image::load_from_memory(skybox_image_nz.as_slice()).unwrap().to_rgba8(),
        ];

        let size = wgpu::Extent3d {
            width: images[0].width(),
            height: images[0].height(),
            depth_or_array_layers: 6,
        };

        assert!(images.clone().into_iter().map(|a| a.dimensions()).all_equal());

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Skybox texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for (num, image) in images.iter().enumerate() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: num as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                image.as_raw().as_slice(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4*size.width),
                    rows_per_image: Some(size.height)
                },
                wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1
                }
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let skybox_bind_group_layout = device.create_bind_group_layout(&consts::bind_group_layouts::SKYBOX_TEXTURE);

        let skybox_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &skybox_bind_group_layout,
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
            label: Some("skybox_texture_bind_group")
        });

        Ok(Self {
            texture, view, sampler, images,
            bind_group_layout: skybox_bind_group_layout,
            bind_group: skybox_bind_group
        })
    }
}