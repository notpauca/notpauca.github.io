mod systems;
mod consts;
mod camera;
mod fetch;
mod time;
mod model;
mod renderer;
mod texture;

use std::{cell::RefCell, rc::Rc, collections::LinkedList, f32::consts::FRAC_PI_2};
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
        rendering_struct.render_skybox(&mut encoder, &view).unwrap();
        rendering_struct.render_meshes(&self.meshes.borrow(), &mut encoder, &view).unwrap();
        rendering_struct.render_gui(&mut encoder, &view).unwrap();

        rendering_struct.queue.submit(Some(encoder.finish()));
        frame.present();
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