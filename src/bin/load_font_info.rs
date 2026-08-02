use wgpu_learning::renderer_backend::FontInfo;


fn main() {
    let contents = std::fs::read_to_string("img/font_data.json").unwrap();

    let info: FontInfo = serde_json::from_str(&contents).unwrap();

    println!("{:?}", info);
}
