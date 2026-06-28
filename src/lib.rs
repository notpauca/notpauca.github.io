mod systems;
mod consts;
mod camera;
mod fetch;
mod time;
mod model;
mod renderer;

use std::{error::Error, cell::RefCell, rc::Rc, collections::LinkedList, f32::consts::FRAC_PI_2};
use cgmath::{Rad, Vector3};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, js_sys::Date};

const SCENE: &[model::Unfinished] = &[
    (
        "monkey.obj",
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(Rad(0.0), Rad(FRAC_PI_2), Rad(0.0)),
        Vector3::new(1.0, 1.0, 1.0)
    ),
    (
        "monkey.obj",
        Vector3::new(5.0, 0.0, 0.0),
        Vector3::new(Rad(0.0), Rad(0.0), Rad(0.0)),
        Vector3::new(1.0, 1.0, 1.0)
    )
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    texture_coords: [f32; 2],
    normal: [f32; 3]
}

impl Vertex {
    const ATTRIBS: &[wgpu::VertexAttribute; 3] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}

struct PortfolioApp {
    rendering_struct: Rc<RefCell<renderer::Renderer>>,
    keyboard_input: Rc<RefCell<systems::KeyboardInput>>,
    mouse_input: Rc<RefCell<systems::MouseInput>>,
    meshes: Rc<RefCell<LinkedList<model::Finished>>>,
}

impl PortfolioApp {
    async fn new(canvas: HtmlCanvasElement) -> Self {
        Self {
            rendering_struct: Rc::new(RefCell::new(renderer::Renderer::new(canvas).await.expect("Can't get the WebGPU instance!"))),
            keyboard_input: Rc::new(RefCell::new(systems::KeyboardInput::default())),
            mouse_input: Rc::new(RefCell::new(systems::MouseInput::default())),
            meshes: Rc::new(RefCell::new(LinkedList::new()))
        }
    }

    async fn initialize_models(&mut self, unfinished_meshes: &[model::Unfinished]) {
        for mesh in unfinished_meshes {
            self.meshes.borrow_mut().append(&mut self.rendering_struct.borrow_mut().load_models(*mesh).await);
        }

    }

    fn update(&self, dt: f64) {
        for mesh in &mut self.meshes.borrow_mut().iter_mut() {
            mesh.scale += Vector3::new(
                (self.rendering_struct.borrow().time.uniform.time/1000.0).sin()*0.01,
                (self.rendering_struct.borrow().time.uniform.time/1000.0).sin()*0.01,
                (self.rendering_struct.borrow().time.uniform.time/1000.0).sin()*0.01,);
            mesh.update_uniform();
            mesh.write_itself(&self.rendering_struct.borrow().queue)
        }

        // web_sys::console::debug_1(&format!("dt: {}", dt).into()); //for frame times
        self.keyboard_input.borrow_mut().update(&self, dt);
        self.mouse_input.borrow_mut().update(&self);
    }

    fn render(&self, dt: f64) {
        let mut rendering_struct = self.rendering_struct.borrow_mut();

        let frame = match rendering_struct.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            _ => panic!("Can't get surface texture!")
        };

        let view = frame.texture.create_view(&Default::default());
        let mut encoder = rendering_struct.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        rendering_struct.update_clock(dt);

        rendering_struct.render_meshes(&self.meshes.borrow(), &mut encoder, &view).unwrap();
        rendering_struct.render_gui(&mut encoder, &view).unwrap();

        rendering_struct.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

struct Texture {
    image: image::RgbaImage,
    image_texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout
}

impl Texture {
    fn new(device: &wgpu::Device) -> Result<Self, Box<dyn Error>> {
        let image_bytes = include_bytes!("../img/pfp.png");
        let image = image::load_from_memory(image_bytes)?.to_rgba8();
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
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let view = image_texture.create_view(&Default::default());

        let bind_group_layout = device.create_bind_group_layout(&consts::TEXTURE_BIND_GROUP_LAYOUT_DESCRIPTOR);

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

    fn write_itself(&self, queue: &wgpu::Queue) {
        queue.write_texture(wgpu::TexelCopyTextureInfo {
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
        self.image_texture.size());
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

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    let canvas = web_sys::window().unwrap()
        .document().unwrap()
        .get_element_by_id(consts::CANVAS_ID).expect("Can't get canvas, maybe change CANVAS_ID?");
    let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;

    let mut app = PortfolioApp::new(canvas).await;

    app.initialize_models(SCENE).await;

    {
        let app_for_callback = app.rendering_struct.clone();
        let closure = Closure::wrap(Box::new(move || {
            app_for_callback.borrow_mut().resize();
        }) as Box<dyn FnMut()>);

        web_sys::window()
            .unwrap()
            .set_onresize(
                Some(closure.as_ref().unchecked_ref()),
            );

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
            app.render(dt);
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