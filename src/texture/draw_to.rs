use wgpu::BindGroupLayout;

pub struct DrawTo {
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}
impl DrawTo {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, bind_group_layout: &BindGroupLayout, color_format: wgpu::TextureFormat) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("prepostproc_texture"),
            size: wgpu::Extent3d {
                width, height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: color_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: Default::default(),
        });

        let view = texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

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
            label: Some("pre_postproc_bind_group"),
        });

        Self {
            texture,
            sampler,
            view,
            bind_group,
        }
    }
}