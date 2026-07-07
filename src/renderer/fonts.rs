pub(crate) async fn load_font(name: &str) -> fontdue::Font {
    let font_file_bytes = crate::fetch::binary_data(&format!("fonts/{name}")).await.unwrap();
    fontdue::Font::from_bytes(font_file_bytes.as_slice(), Default::default()).unwrap()
}