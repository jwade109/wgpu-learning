use crate::renderer_backend::FontInfo;
use glm::Vec4;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub enum RenderCommand {
    Char(CharCommand),
    Rect(RectCommand),
}

#[derive(Debug, Clone, Copy)]
pub struct RectCommand {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Vec4,
}

#[derive(Debug, Clone, Copy)]
pub struct CharCommand {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub c: char,
    pub font: usize,
    pub color: Vec4,
}

pub struct RenderCommands {
    pub fonts: BTreeMap<usize, FontInfo>,
    commands: Vec<RenderCommand>,
}

impl RenderCommands {
    pub fn new(fonts: BTreeMap<usize, FontInfo>) -> Self {
        Self {
            fonts,
            commands: Vec::new(),
        }
    }

    pub fn commands(&self) -> impl Iterator<Item = &RenderCommand> {
        self.commands.iter()
    }

    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: Vec4) {
        self.commands.push(RenderCommand::Rect(RectCommand {
            x,
            y,
            width: w,
            height: h,
            color,
        }));
    }

    pub fn char(&mut self, x: f64, y: f64, w: f64, h: f64, c: char, font: usize, color: Vec4) {
        self.commands.push(RenderCommand::Char(CharCommand {
            x,
            y,
            width: w,
            height: h,
            c,
            font,
            color,
        }));
    }

    pub fn paragraph(
        &mut self,
        font_id: usize,
        font_size: f64,
        x_origin: f64,
        y_origin: f64,
        text: &str,
        layout_width: Option<f64>,
    ) {
        // TODO this is terrible
        let font = self.fonts.get(&font_id).unwrap().clone();

        let font_size = font_size / font.size as f64;

        let mut col_offset = 0;

        let mut x = x_origin;
        let mut y = y_origin;

        for ch in text.chars() {
            if ch == '\n' {
                y += font.size as f64 * font_size;
                x = x_origin;
                col_offset = 0;
                continue;
            }

            let Some(data) = font.characters.get(&ch) else {
                continue;
            };

            if ch == ' ' && col_offset == 0 {
                continue;
            }

            let w = data.width as f64 * font_size;
            let h = data.height as f64 * font_size;

            let xt = x - data.origin_x as f64 * font_size;
            let yt = y - data.origin_y as f64 * font_size + font.size as f64 * font_size;

            let color = Vec4::new(1.0, 1.0, 1.0, 1.0);

            if ch != ' ' {
                self.char(xt, yt, w, h, ch, font_id, color);
            }

            col_offset += 1;

            x += data.advance as f64 * font_size;

            if let Some(layout_width) = layout_width {
                if ch == ' ' && x + w > x_origin + layout_width {
                    y += font.size as f64 * font_size;
                    x = x_origin;
                    col_offset = 0;
                }
            }
        }
    }
}
