use std::{error::Error, cell::RefCell, rc::Rc, collections::HashSet, borrow::Cow};
use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, Rad, Vector3};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, js_sys::Date, Request, RequestInit};
use wgpu::util::DeviceExt;

const CANVAS_ID: &'static str = "canvas";
const SHADER_FILE_PATH: &'static str = "src/shaders.wgsl";

async fn get_shader_source() -> Result<String, JsValue> {
    let shader_source_request = RequestInit::new();
    shader_source_request.set_method("GET");
    shader_source_request.set_mode(web_sys::RequestMode::Cors);

    let request = Request::new_with_str_and_init(SHADER_FILE_PATH, &shader_source_request)?;

    request.headers()
        .set("Accept", "text/wgsl")?;

    let resp_value = web_sys::window().unwrap().fetch_with_request(&request).await?;
    assert!(resp_value.is_instance_of::<web_sys::Response>());
    let resp_value = resp_value.dyn_into::<web_sys::Response>()?;
    let res = resp_value.text()?.await?;
    Ok(res.as_string().unwrap())
}


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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct TimeUniform {
    time: f32
}

impl TimeUniform {
    fn new() -> Self {
        Self {
            time: 0f32
        }
    }
}

//TODO: handle mouse input, as angle is only controlled with arrow keys for now.
struct PortfolioApp {
    rendering_struct: Rc<RefCell<RenderingStruct>>,
    keyboard_input: Rc<RefCell<KeyboardInputSystem>>,
    mouse_input: Rc<RefCell<MouseInputSystem>>,
    vertices: Vec<Vertex>, //TODO: maybe make a proper mesh class, let that do the GPU memory buffer stuff?
    indices: Vec<[u16; 3]>
}

impl PortfolioApp {
    async fn new(canvas: HtmlCanvasElement) -> Self {
        let vertices = vec![
            Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [1.0, 0.0, 0.0] },
            Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.0, 0.0, 1.0] },
            Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.0, 1.0, 1.0] },
            Vertex { position: [0.44147372, 0.2347359, 0.0], color: [1.0, 1.0, 0.0] },
        ];

        let indices = vec![
            [ 0, 1, 4 ],
            [ 1, 2, 4 ],
            [ 2, 3, 4 ],
        ];

        Self {
            rendering_struct: Rc::new(RefCell::new(RenderingStruct::new(canvas, &vertices, &indices).await)),
            keyboard_input: Rc::new(RefCell::new(KeyboardInputSystem::default())),
            mouse_input: Rc::new(RefCell::new(MouseInputSystem::default())),
            vertices, indices
        }
    }

    fn update(&self, dt: f64) {
        // web_sys::console::debug_1(&format!("dt: {}", dt).into()); //for frame times
        self.keyboard_input.borrow_mut().update(&mut self.rendering_struct.borrow_mut().camera, dt);
        self.mouse_input.borrow_mut().update(&mut self.rendering_struct.borrow_mut().camera, dt);
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
            position: (0.0, 0.0, 10.0).into(),
            yaw: Deg(-90.0).into(),
            pitch: Deg(0.0).into(),
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
        let (mut right, mut up, mut forward) = (0.0, 0.0, 0.0);

        //camera eye
        if self.keys.contains("KeyD") {
            right +=1.0;
        }
        if self.keys.contains("KeyA") {
            right -=1.0;
        }
        if self.keys.contains("KeyW") {
            forward +=1.0;
        }
        if self.keys.contains("KeyS") {
            forward -=1.0;
        }
        if self.keys.contains("Space") {
            up+=1.0;
        }
        if self.keys.contains("ShiftLeft") || self.keys.contains("ShiftRight") {
            up-=1.0;
        }

        //camera angle
        if self.keys.contains("ArrowRight") {
            camera.yaw+=Rad(0.001*dt as f32);
        }
        if self.keys.contains("ArrowLeft") {
            camera.yaw-=Rad(0.001*dt as f32);
        }
        if self.keys.contains("ArrowUp") {
            camera.pitch+=Rad(0.001*dt as f32);
        }
        if self.keys.contains("ArrowDown") {
            camera.pitch-=Rad(0.001*dt as f32);
        }


        if self.keys.contains("Enter") {
            web_sys::console::info_1(&format!("{camera:?}").into())
        }

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();

        //https://sotrh.github.io/learn-wgpu/intermediate/tutorial12-camera/#the-camera-controller
        //too lazy to remember the right math, so just stole it.
        let forward_vector = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right_vector = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward_vector * forward * 0.01 * dt as f32;
        camera.position += right_vector * right * 0.01 * dt as f32;
        camera.position.y +=up*0.01*dt as f32;
    }
}

#[derive(Default)]
struct MouseInputSystem {
    mouse_movement_delta: (f32, f32),
    mouse_locked: bool
}

impl MouseInputSystem {
    fn mouse_moved(&mut self, event: web_sys::PointerEvent) {
        if self.mouse_locked {
            self.mouse_movement_delta = (event.movement_x() as f32, event.movement_y() as f32);
        }
    }

    fn update(&mut self, camera: &mut Camera, dt: f64) {
        camera.pitch+=Deg(-self.mouse_movement_delta.1).into();
        camera.yaw+=Deg(self.mouse_movement_delta.0).into();
        self.mouse_movement_delta = (0.0, 0.0);
    }

    fn mouse_clicked(&mut self, event: web_sys::PointerEvent) {
        self.mouse_locked = !self.mouse_locked;
        if self.mouse_locked {
            web_sys::window().unwrap()
                .document().unwrap()
                .get_element_by_id(CANVAS_ID).unwrap()
                .request_pointer_lock();
        } else {
            web_sys::window().unwrap()
                .document().unwrap().exit_pointer_lock()
        }
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
    camera_bind_group: wgpu::BindGroup,
    time_uniform: TimeUniform,
    time_buffer: wgpu::Buffer,
    time_bind_group: wgpu::BindGroup
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

        let shader_source = get_shader_source().await.expect("Can't get shader source!");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shaders.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_source)),
        });

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

        let time_uniform = TimeUniform::new();

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time buffer"),
            contents: bytemuck::cast_slice(&[time_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let time_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("time_bind_group_layout"),
        });

        let time_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &time_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: time_buffer.as_entire_binding(),
                }
            ],
            label: Some("time_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera_bind_group_layout),
                    Some(&time_bind_group_layout),
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
            surface, device, queue, canvas, config, render_pipeline, vertex_buffer, index_buffer, camera, camera_uniform, camera_buffer, camera_bind_group, time_uniform, time_buffer, time_bind_group
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

    fn render(&mut self, dt: f64) -> Result<(), Box<dyn Error>> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            _ => panic!("Can't get surface texture!")
        };

        self.camera_uniform = CameraUniform::new(self.camera.calc_projection_matrix());
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        self.time_uniform.time+=dt as f32;
        self.queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[self.time_uniform]));

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
            render_pass.set_bind_group(1, &self.time_bind_group, &[]);
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
        .get_element_by_id(CANVAS_ID).expect("Can't get canvas, maybe change CANVAS_ID?");
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