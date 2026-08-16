use wgpu_learning::FontInfo;


fn main() {
    let contents = std::fs::read_to_string("fonts/consolas/font_data.json").unwrap();

    let info: FontInfo = serde_json::from_str(&contents).unwrap();

    println!("{:?}", info);
}
