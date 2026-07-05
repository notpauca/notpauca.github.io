use std::{borrow::Cow, collections::LinkedList, error::Error};
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;
use crate::{camera, consts, fetch, model, time, Vertex, texture};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex2D {
    coords: [f32; 3],
    color: [f32; 4]
}

impl Vertex2D {
    const ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRS,
        }
    }
}

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    canvas: HtmlCanvasElement,
    pub config: wgpu::SurfaceConfiguration,
    pub pre_postproc_texture: texture::DrawTo,
    pre_postproc_texture_bind_group_layout: wgpu::BindGroupLayout,
    mesh_bind_group_layout: wgpu::BindGroupLayout,
    mesh_render_pipeline: wgpu::RenderPipeline,
    gui_render_pipeline: wgpu::RenderPipeline,
    postproc_render_pipeline: wgpu::RenderPipeline,
    pub texture: texture::Texture,
    pub camera: camera::Camera,
    pub time: time::Time,
    pub depth_texture: texture::Depth,
    skybox_render_pipeline: wgpu::RenderPipeline,
    skybox: texture::Skybox,
    screen_size_bind_group: wgpu::BindGroup,
    screen_size_buffer: wgpu::Buffer,
}

impl Renderer {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, Box<dyn Error>> {
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::BROWSER_WEBGPU,
                flags: Default::default(),
                memory_budget_thresholds: Default::default(),
                backend_options: Default::default(),
                display: None,
            }
        );

        let surface = instance.create_surface(
            wgpu::SurfaceTarget::Canvas(canvas.clone())
        ).expect("Can't get wgpu surface!");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: adapter.features(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: adapter.limits(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let size = canvas.get_bounding_client_rect();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: wgpu::SurfaceColorSpace::Srgb,
            width: size.width() as u32,
            height: size.height() as u32,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };

        let pre_postproc_texture_bind_group_layout = device.create_bind_group_layout(&consts::bind_group_layouts::TEXTURE);

        let pre_postproc_texture = texture::DrawTo::new(&device, config.width, config.height, &pre_postproc_texture_bind_group_layout);

        surface.configure(
            &device,
            &config,
        );

        let mesh_shader_source = fetch::shader_source(consts::MESH_SHADER_PATH).await.expect("Can't get mesh shader source!");
        let mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesh_stage_shaders.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&mesh_shader_source)),
        });

        let gui_shader_source = fetch::shader_source(consts::GUI_SHADER_PATH).await.expect("Can't get GUI shader source!");
        let gui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gui_stage_shaders.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&gui_shader_source)),
        });

        let skybox_shader_source = fetch::shader_source(consts::SKYBOX_SHADER_PATH).await.expect("Can't get skybox shader source!");
        let skybox_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("skybox_stage_shaders.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&skybox_shader_source)),
        });

        let postproc_shader_source = fetch::shader_source(consts::POSTPROC_SHADER_PATH).await.expect("Can't get postproc shader source!");
        let postproc_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("postproc_stage_shaders.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&postproc_shader_source)),
        });

        let depth_texture = {
            let rect = canvas.get_bounding_client_rect();
            texture::Depth::new(&device, rect.width() as u32, rect.height() as u32)
        };

        let aspect = {
            let rect = canvas.get_bounding_client_rect();
            (rect.width() / rect.height()) as f32
        };

        let camera = camera::Camera::new(aspect, &device);

        let time = time::Time::new(&device);

        let mesh_bind_group_layout = device.create_bind_group_layout(&consts::bind_group_layouts::MESH_TRANSFORM);

        let texture = texture::Texture::new(&device).await?;

        let mesh_render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera.bind_group_layout),
                    Some(&time.bind_group_layout),
                    Some(&mesh_bind_group_layout),
                    Some(&texture.bind_group_layout)
                ],
                immediate_size: 0,
            }
        );

        let mesh_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesh_render_pipeline"),
            layout: Some(&mesh_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &mesh_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Some(Vertex::desc())
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &mesh_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let gui_render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Gui Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&time.bind_group_layout),
                    //texture will come later, hopefully
                ],
                immediate_size: 0,
            }
        );

        let gui_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gui_render_pipeline"),
            layout: Some(&gui_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &gui_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &gui_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let skybox = texture::Skybox::new(&device, &queue).await.unwrap();

        let skybox_render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Skybox Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera.skybox_bind_group_layout),
                    Some(&time.bind_group_layout),
                    Some(&skybox.bind_group_layout)
                ],
                immediate_size: 0,
            }
        );

        let skybox_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("skybox_render_pipeline"),
            layout: Some(&skybox_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &skybox_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &skybox_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let screen_size_arr = [size.width() as u32, size.height() as u32];
        let screen_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("screen_size_buffer"),
            contents: bytemuck::cast_slice(&screen_size_arr),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let screen_size_bind_group_layout = device.create_bind_group_layout(&consts::bind_group_layouts::SCREEN_SIZE);

        let screen_size_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &screen_size_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding()
            }],
            label: Some("screen_size_bind_group")
        });

        let postproc_render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Postproc Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&time.bind_group_layout),
                    Some(&pre_postproc_texture_bind_group_layout),
                    Some(&screen_size_bind_group_layout)
                ],
                immediate_size: 0,
            }
        );

        let postproc_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("postproc_render_pipeline"),
            layout: Some(&postproc_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &postproc_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &postproc_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            surface, device, queue, canvas, config,
            camera, time,
            mesh_render_pipeline, texture, depth_texture, mesh_bind_group_layout,
            gui_render_pipeline,
            skybox_render_pipeline, skybox,
            postproc_render_pipeline, pre_postproc_texture, pre_postproc_texture_bind_group_layout,
            screen_size_bind_group, screen_size_buffer
        })
    }

    pub fn resize(&mut self) {
        let (width, height) = {
            let size = self.canvas.get_bounding_client_rect();
            (size.width() as u32, size.height() as u32)
        };

        web_sys::console::log_1(&format!("w: {}, h: {}", width, height).into());

        if width == 0 || height == 0 { return; }
        self.canvas.set_width(width);
        self.canvas.set_height(height);

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        self.pre_postproc_texture = texture::DrawTo::new(&self.device, width, height, &self.pre_postproc_texture_bind_group_layout);

        self.queue.write_buffer(&self.screen_size_buffer, 0, bytemuck::cast_slice(&[width, height]));

        self.camera.resize(width as f32/height as f32);

        self.depth_texture = texture::Depth::new(&self.device, width, height);
    }

    pub fn update_clock(&mut self, dt: f64) {
        self.time.advance(dt as f32);
        self.time.write_itself(&self.queue);
    }

    pub fn render_skybox(&mut self, encoder: &mut wgpu::CommandEncoder) -> Result<(), Box<dyn Error>> {
        self.camera.update_skybox_uniform();
        self.camera.write_rotations(&self.queue);
        let view = &self.pre_postproc_texture.view;
        self.time.write_itself(&self.queue);
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("skybox_render_pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                },
            );
            render_pass.set_pipeline(&self.skybox_render_pipeline);
            render_pass.set_bind_group(0, &self.camera.skybox_bind_group, &[]);
            render_pass.set_bind_group(1, &self.time.bind_group, &[]);
            render_pass.set_bind_group(2, &self.skybox.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        Ok(())
    }

    pub fn render_meshes(&mut self, objects: &LinkedList<model::Finished>, encoder: &mut wgpu::CommandEncoder) -> Result<(), Box<dyn Error>> {
        let view = &self.pre_postproc_texture.view;
        self.camera.update_uniform();
        self.camera.write_itself(&self.queue);
        self.texture.write_itself(&self.queue);
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("mesh_render_pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load, //using skybox now
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                },
            );
            render_pass.set_pipeline(&self.mesh_render_pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.time.bind_group, &[]);
            for mesh in objects {
                render_pass.set_bind_group(2, &mesh.bind_group, &[]);
                render_pass.set_bind_group(3, &self.texture.bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..((mesh.indices.len()*3) as u32), 0, 0..1);
            }
        }
        Ok(())
    }

    pub fn render_gui(&mut self, encoder: &mut wgpu::CommandEncoder) -> Result<(), Box<dyn Error>> {
        let view = &self.pre_postproc_texture.view;
        {
            self.time.write_itself(&self.queue);
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("gui_render_pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                },
            );
            render_pass.set_pipeline(&self.gui_render_pipeline);
            render_pass.set_bind_group(0, &self.time.bind_group, &[]);
            render_pass.draw(0..3, 0..1);

        }
        Ok(())
    }

    pub fn render_postproc(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) -> Result<(), Box<dyn Error>> {
        self.time.write_itself(&self.queue);
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("postproc_render_pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                },
            );
            render_pass.set_pipeline(&self.postproc_render_pipeline);
            render_pass.set_bind_group(0, &self.time.bind_group, &[]);
            render_pass.set_bind_group(1, &self.pre_postproc_texture.bind_group, &[]);
            render_pass.set_bind_group(2, &self.screen_size_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        Ok(())
    }

    pub async fn load_models(&self, model: model::Unfinished) -> LinkedList<model::Finished> {
        model::Finished::from_unfinished(model, &self.device, &self.mesh_bind_group_layout).await.unwrap()
    }
}