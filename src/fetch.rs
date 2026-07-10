use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Request, RequestInit};
use crate::consts;

pub async fn shader_source(path: &str) -> Result<String, JsValue> {
    let shader_source_request = RequestInit::new();
    shader_source_request.set_method("GET");
    shader_source_request.set_mode(web_sys::RequestMode::Cors);

    let request = Request::new_with_str_and_init(path, &shader_source_request)?;

    request.headers()
        .set("Accept", "text/wgsl")?;

    let resp_value = web_sys::window().unwrap().fetch_with_request(&request).await?;
    assert!(resp_value.is_instance_of::<web_sys::Response>());
    let resp_value = resp_value.dyn_into::<web_sys::Response>()?;
    let res = resp_value.text()?.await?;
    Ok(res.as_string().unwrap())
}

pub async fn binary_data(path: &str) -> Result<Vec<u8>, JsValue> {
    let shader_source_request = RequestInit::new();
    shader_source_request.set_method("GET");
    shader_source_request.set_mode(web_sys::RequestMode::Cors);

    let path = format!("{}{path}", consts::ASSETS_DIRECTORY);

    let request = Request::new_with_str_and_init(&path, &shader_source_request)?;

    request.headers()
        .set("Accept", "*")?;

    let resp_value = web_sys::window().unwrap().fetch_with_request(&request).await?;
    assert!(resp_value.is_instance_of::<web_sys::Response>());
    let resp_value = resp_value.dyn_into::<web_sys::Response>()?;
    let res = web_sys::js_sys::Uint8Array::new(&resp_value.array_buffer()?.await?);
    Ok(res.to_vec())
}

pub async fn font(name: &str) -> fontdue::Font {
    let font_file_bytes = binary_data(&format!("fonts/{name}")).await.unwrap();
    fontdue::Font::from_bytes(font_file_bytes.as_slice(), Default::default()).unwrap()
}