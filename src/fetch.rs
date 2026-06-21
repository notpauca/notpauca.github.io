use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Request, RequestInit};
use crate::consts;

pub async fn shader_source() -> Result<String, JsValue> {
    let shader_source_request = RequestInit::new();
    shader_source_request.set_method("GET");
    shader_source_request.set_mode(web_sys::RequestMode::Cors);

    let request = Request::new_with_str_and_init(consts::SHADER_FILE_PATH, &shader_source_request)?;

    request.headers()
        .set("Accept", "text/wgsl")?;

    let resp_value = web_sys::window().unwrap().fetch_with_request(&request).await?;
    assert!(resp_value.is_instance_of::<web_sys::Response>());
    let resp_value = resp_value.dyn_into::<web_sys::Response>()?;
    let res = resp_value.text()?.await?;
    Ok(res.as_string().unwrap())
}