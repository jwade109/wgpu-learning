use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct CodePointInfo {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "originX")]
    pub origin_x: i32,
    #[serde(rename = "originY")]
    pub origin_y: i32,
    pub advance: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FontInfo {
    pub name: String,
    pub size: u32,
    pub bold: bool,
    pub italic: bool,
    pub width: u32,
    pub height: u32,
    pub characters: HashMap<char, CodePointInfo>,
}
