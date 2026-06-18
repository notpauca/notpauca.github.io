use std::{error::Error, cell::RefCell, rc::Rc, collections::HashSet};
use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, Rad, Vector3};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, js_sys::Date};
use wgpu::util::DeviceExt;

const CANVAS_ID: &'static str = "canvas";
const SHADER_FILE_PATH: &'static str = "./src/shaders.wgsl";

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: &[wgpu::VertexAttribute; 2] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4]
}

impl CameraUniform {
    fn new(mat: Matrix4<f32>) -> Self {
        Self {
            view_proj: mat.into()
        }
    }
}


struct PortfolioApp {
    rendering_struct: Rc<RefCell<RenderingStruct>>,
    keyboard_input: Rc<RefCell<KeyboardInputSystem>>,
    vertices: Vec<Vertex>, //TODO: maybe make a proper mesh class, let that do the GPU memory buffer stuff?
    indices: Vec<[u16; 3]>
}

impl PortfolioApp {
    async fn new(canvas: HtmlCanvasElement) -> Self {
        let vertices = vec![
            Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [1.0, 0.0, 0.0] }, // A
            Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.0, 1.0, 0.0] }, // B
            Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.0, 0.0, 1.0] }, // C
            Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.0, 1.0, 1.0] }, // D
            Vertex { position: [0.44147372, 0.2347359, 0.0], color: [1.0, 1.0, 0.0] }, // E
        ];

        let indices = vec![
            [ 0, 1, 4 ],
            [ 1, 2, 4 ],
            [ 2, 3, 4 ],
        ];

        Self {
            rendering_struct: Rc::new(RefCell::new(RenderingStruct::new(canvas, &vertices, &indices).await)),
            keyboard_input: Rc::new(RefCell::new(KeyboardInputSystem::default())),
            vertices, indices
        }
    }

    fn update(&self, dt: f64) {
        // web_sys::console::debug_1(&format!("dt: {}", dt).into()); //for frame times
        self.keyboard_input.borrow_mut().update(&mut self.rendering_struct.borrow_mut().camera, dt);
    }
}

#[derive(Debug)]
struct Camera {
    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    aspect: f32,
    fov: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: (0.0, 5.0, 10.0).into(),
            yaw: Deg(-90.0).into(),
            pitch: Deg(-20.0).into(),
            aspect,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
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

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}


#[derive(Default)]
#[repr(transparent)]
struct KeyboardInputSystem {
    keys: HashSet<String>
}

impl KeyboardInputSystem {
    fn keyboard_down(&mut self, event: web_sys::KeyboardEvent) {
        if self.keys.insert(event.code()) {
            web_sys::console::log_1(&format!("pressed key: {}", event.code()).into());
        }
    }

    fn keyboard_up(&mut self, event: web_sys::KeyboardEvent) {
        if self.keys.remove(&event.code()) {
            web_sys::console::log_1(&format!("released key: {}", event.code()).into());
        }
    }

    fn update(&mut self, camera: &mut Camera, dt: f64) {
        let mut movement_vector: Vector3<f32> = (0.0, 0.0, 0.0).into();

        if self.keys.contains("KeyD") {
            movement_vector.x+=1.0;
        }
        if self.keys.contains("KeyA") {
            movement_vector.x-=1.0;
        }
        if self.keys.contains("KeyW") {
            movement_vector.z-=1.0;
        }
        if self.keys.contains("KeyS") {
            movement_vector.z+=1.0;
        }
        if self.keys.contains("Space") {
            movement_vector.y+=1.0;
        }
        if self.keys.contains("ShiftLeft") || self.keys.contains("ShiftRight") {
            movement_vector.y-=1.0;
        }

        if self.keys.contains("Enter") {
            web_sys::console::info_1(&format!("{camera:?}").into())
        }

        // let movement_vector: Point3<f32> = movement_vector.normalize();

        camera.position += movement_vector * dt as f32 * 0.01;

    }
}

struct RenderingStruct {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    canvas: HtmlCanvasElement,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup
}

impl RenderingStruct {
    async fn new(canvas: HtmlCanvasElement, vertices: &Vec<Vertex>, indices: &Vec<[u16; 3]>) -> Self {
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

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders.wgsl")); //TODO: dynamically load this file?

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let aspect = {
            let rect = canvas.get_bounding_client_rect();
            (rect.width()/rect.height()) as f32
        };

        let camera = Camera::new(aspect);

        let camera_uniform = CameraUniform::new(camera.calc_projection_matrix());
        // let camera_uniform = CameraUniform::new(Matrix4::from_value(1.0));

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera uniform buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera_bind_group_layout),
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
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // cull_mode: None,
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

        Self {
            surface, device, queue, canvas, config, render_pipeline, vertex_buffer, index_buffer, camera, camera_uniform, camera_buffer, camera_bind_group,
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

        self.camera.aspect = width as f32/height as f32;
        self.camera_uniform = CameraUniform::new(self.camera.calc_projection_matrix());
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    fn render(&mut self) -> Result<(), Box<dyn Error>> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            _ => panic!("Can't get surface texture!")
        };

        self.camera_uniform = CameraUniform::new(self.camera.calc_projection_matrix());
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

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
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                },
            );
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..9, 0, 0..1);
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
        .get_element_by_id(CANVAS_ID).expect("Can't get canvas, maybe change CANVAS_ID?"); //gotta draw to it now!
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

    //callback for when key's released, as "keydown" acts as if the player's using a textbox and holding a button
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

    //rendering loop, a big, "recursive" rendering callback thing
    {
        let mut time = Date::now();
        let f = Rc::new(RefCell::new(None::<ScopedClosure<dyn FnMut()>>));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let current_time = Date::now();
            app.update(current_time-time);
            time = current_time;
            app.rendering_struct.borrow_mut().render().unwrap();
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