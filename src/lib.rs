use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

const CANVAS_ID: &'static str = "canvas";

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("Can't get window!");
    let document = window.document().expect("Can't get window.document!");
    let canvas = document.get_element_by_id(CANVAS_ID).expect("Can't get canvas, maybe change CANVAS_ID?"); //gotta draw to it now!
    let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;

    let canvas_bounding_rect = canvas.get_bounding_client_rect();
    let canvas_height = canvas_bounding_rect.height() as u32;
    let canvas_width = canvas_bounding_rect.width() as u32;

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
        wgpu::SurfaceTarget::Canvas(canvas)
    ).expect("Can't get wgpu surface!");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
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


    let surface_caps = surface.get_capabilities(&adapter);

    let surface_format = surface_caps.formats.iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: canvas_width,
        height: canvas_height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: Vec::new(),
        desired_maximum_frame_latency: 2,
    };

    surface.configure(
        &device,
        &config,
    );

    let frame = match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(texture) => texture,
        _ => panic!("Can't get surface texture!")
    };

    let view = frame.texture.create_view(&Default::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    {
        let _pass = encoder.begin_render_pass(
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
    }
    queue.submit(Some(encoder.finish()));
    frame.present();

    web_sys::console::log_1(&format!("w: {canvas_width}, h: {canvas_height}").into());

    Ok(())

}
