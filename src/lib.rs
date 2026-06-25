mod systems;
mod consts;
mod camera;
mod fetch;
mod time;
mod mesh;

use std::{error::Error, cell::RefCell, rc::Rc, collections::LinkedList, borrow::Cow};
use cgmath::{Deg, Rad, Vector3};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, js_sys::Date};
use crate::mesh::{Mesh, UnfinishedMesh};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl Vertex {
    const ATTRIBS: &[wgpu::VertexAttribute; 2] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}

struct PortfolioApp {
    rendering_struct: Rc<RefCell<RenderingStruct>>,
    keyboard_input: Rc<RefCell<systems::KeyboardInput>>,
    mouse_input: Rc<RefCell<systems::MouseInput>>,
}

impl PortfolioApp {
    async fn new(canvas: HtmlCanvasElement) -> Self {
        let mut unfinished_meshes = LinkedList::new();
        unfinished_meshes.push_back(
            (
                vec![
                    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
                    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.0, 1.0, 0.0, 1.0] },
                    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.0, 0.0, 1.0, 1.0] },
                    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.0, 1.0, 1.0, 1.0] },
                    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [1.0, 1.0, 0.0, 1.0] },
                ],
                vec![
                    [0, 1, 4],
                    [1, 2, 4],
                    [2, 3, 4],
                ],
                Vector3::new(2.0,0.0,0.0),
                Vector3::new(Rad(0.0), Deg(90.0).into(), Rad(0.0)),
                Vector3::new(1.0,1.0,1.0)
            )
        );
        unfinished_meshes.push_back(
            (
                vec![
                    Vertex { position: [-0.0868241, 1.49240386, 1.0], color: [1.0, 0.0, 0.0, 0.5] },
                    Vertex { position: [-0.49513406, 1.06958647, 1.0], color: [0.0, 1.0, 0.0, 0.5] },
                    Vertex { position: [-0.21918549, -1.44939706, 1.0], color: [0.0, 0.0, 1.0, 0.5] },
                    Vertex { position: [0.35966998, -1.3473291, 1.0], color: [0.0, 1.0, 1.0, 0.5] },
                    Vertex { position: [0.44147372, 1.2347359, 1.0], color: [1.0, 1.0, 0.0, 0.5] },
                ],
                vec![
                    [1, 4, 0],
                    [2, 4, 1],
                    [3, 4, 2],
                ],
                Vector3::new(0.0,0.0,0.0),
                Vector3::new(Rad(0.0),Rad(0.0),Rad(0.0)),
                Vector3::new(1.0,1.0,1.0)
            )
        );

        Self {
            rendering_struct: Rc::new(RefCell::new(RenderingStruct::new(canvas, unfinished_meshes).await)),
            keyboard_input: Rc::new(RefCell::new(systems::KeyboardInput::default())),
            mouse_input: Rc::new(RefCell::new(systems::MouseInput::default())),
        }
    }

    fn update(&self, dt: f64) {
        // web_sys::console::debug_1(&format!("dt: {}", dt).into()); //for frame times
        self.keyboard_input.borrow_mut().update(&self, dt);
        self.mouse_input.borrow_mut().update(&self);
    }
}

struct DepthTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl DepthTexture {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width, height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::Less),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );
        Self {texture, view, sampler}
    }
}

struct RenderingStruct {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    canvas: HtmlCanvasElement,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    meshes: LinkedList<Mesh>,
    camera: camera::Camera,
    time: time::Time,
    depth_texture: DepthTexture
}

impl RenderingStruct {
    async fn new(canvas: HtmlCanvasElement, unfinished_meshes: LinkedList<UnfinishedMesh>) -> Self {
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
            })
            .await.unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                //apparently webgl doesn't support everything that webgpu has to offer
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await.unwrap();

        let size = canvas.get_bounding_client_rect();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width() as u32,
            height: size.height() as u32,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };

        surface.configure(
            &device,
            &config,
        );

        let shader_source = fetch::shader_source().await.expect("Can't get shader source!");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shaders.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_source)),
        });

        let depth_texture = {
            let rect = canvas.get_bounding_client_rect();
            DepthTexture::new(&device, rect.width() as u32, rect.height() as u32)
        };

        let aspect = {
            let rect = canvas.get_bounding_client_rect();
            (rect.width() / rect.height()) as f32
        };

        let camera = camera::Camera::new(aspect, &device);

        let time = time::Time::new(&device);

        let mesh_bind_group_layout = device.create_bind_group_layout(&consts::MESH_TRANSFORM_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let meshes = unfinished_meshes.into_iter().map(|unfinished_mesh| {
            Mesh::new(unfinished_mesh, &device, &mesh_bind_group_layout)
        }).collect();

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera.bind_group_layout),
                    Some(&time.bind_group_layout),
                    Some(&mesh_bind_group_layout),
                ],
                immediate_size: 0,
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
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

        Self {
            surface, device, queue, canvas, config, render_pipeline, meshes, camera, time, depth_texture
        }
    }

    fn resize(&mut self) {
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

        self.camera.resize(width as f32/height as f32);

        self.depth_texture = DepthTexture::new(&self.device, width, height);
    }

    fn render(&mut self, dt: f64) -> Result<(), Box<dyn Error>> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            _ => panic!("Can't get surface texture!")
        };

        self.camera.update_uniform();
        self.camera.write_itself(&self.queue);

        self.time.advance(dt as f32);
        self.time.write_itself(&self.queue);

        for mesh in &mut self.meshes {
            mesh.update_uniform();
            mesh.write_itself(&self.queue)
        }

        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 0.2,
                                        g: 0.4,
                                        b: 0.8,
                                        a: 1.0,
                                    }
                                ),
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.time.bind_group, &[]);
            for mesh in &mut self.meshes {
                render_pass.set_bind_group(2, &mesh.bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..((mesh.indices.len()*3) as u32), 0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    let canvas = web_sys::window().unwrap()
        .document().unwrap()
        .get_element_by_id(consts::CANVAS_ID).expect("Can't get canvas, maybe change CANVAS_ID?");
    let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;

    let app = PortfolioApp::new(canvas).await;

    {
        let app_for_callback = app.rendering_struct.clone();
        let closure = Closure::wrap(Box::new(move || {
            app_for_callback.borrow_mut().resize();
            // app_for_callback.borrow_mut().render().unwrap();
        }) as Box<dyn FnMut()>);

        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback(
                "resize",
                closure.as_ref().unchecked_ref(),
            )?;

        closure.forget();
    }

    //callback for when key's pressed
    {
        let watcher_for_callback = app.keyboard_input.clone();
        let closure = Closure::wrap(Box::new(move |event| {
            watcher_for_callback.borrow_mut().keyboard_down(event);
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        web_sys::window()
            .unwrap()
            .set_onkeydown(Some(closure.as_ref().unchecked_ref()));

        closure.forget();
    }

    //callback for when key's released, as using just "keydown" is as if the player's using a textbox and holding a letter key
    {
        let watcher_for_callback = app.keyboard_input.clone();
        let closure = Closure::wrap(Box::new(move |event| {
            watcher_for_callback.borrow_mut().keyboard_up(event);
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        web_sys::window()
            .unwrap()
            .set_onkeyup(Some(closure.as_ref().unchecked_ref()));

        closure.forget();
    }

    //callback for mouse movement actions
    {
        let watcher_for_callback = app.mouse_input.clone();
        let closure = Closure::wrap(Box::new(move |event| {
            watcher_for_callback.borrow_mut().mouse_moved(event);
        }) as Box<dyn FnMut(web_sys::PointerEvent)>);

        web_sys::window()
            .unwrap()
            .set_onpointermove(Some(closure.as_ref().unchecked_ref()));

        closure.forget();
    }

    //callback for mouse movement actions
    {
        let watcher_for_callback = app.mouse_input.clone();
        let closure = Closure::wrap(Box::new(move |event| {
            watcher_for_callback.borrow_mut().mouse_clicked(event);
        }) as Box<dyn FnMut(web_sys::PointerEvent)>);

        web_sys::window()
            .unwrap()
            .set_onpointerdown(Some(closure.as_ref().unchecked_ref()));

        closure.forget();
    }

    //rendering loop, a big, "recursive" rendering callback thing
    {
        let mut time = Date::now();
        let f = Rc::new(RefCell::new(None::<ScopedClosure<dyn FnMut()>>));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let current_time = Date::now();
            let dt = current_time-time;
            time = current_time;
            app.update(dt);
            app.rendering_struct.borrow_mut().render(dt).unwrap();
            web_sys::window().unwrap()
                .request_animation_frame(
                    f.borrow()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                )
                .unwrap();
        }) as Box<dyn FnMut()>));

        web_sys::window().unwrap()
            .request_animation_frame(
                g.borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )?;
    }
    Ok(())
}