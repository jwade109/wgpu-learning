use crate::{Color, FontInfo};
use std::collections::BTreeMap;

pub type Vec2d = glm::Vector2<f64>;

#[derive(Debug, Clone, Copy)]
pub enum RenderCommand {
    Char(CharCommand),
    Rect(RectCommand),
    Circle(CircleCommand),
    Line(LineCommand),
}

#[derive(Debug, Clone, Copy)]
pub struct RectCommand {
    pub pos: Vec2d,
    pub dims: Vec2d,
    pub angle: f64,
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct CharCommand {
    pub pos: Vec2d,
    pub dims: Vec2d,
    pub c: char,
    pub font: usize,
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct CircleCommand {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct LineCommand {
    pub start: Vec2d,
    pub end: Vec2d,
    pub thickness: f64,
    pub color: Color,
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

    pub fn rect(&mut self, pos: Vec2d, dims: Vec2d, angle: f64, color: Color) {
        self.commands.push(RenderCommand::Rect(RectCommand {
            pos,
            dims,
            angle,
            color,
        }));
    }

    pub fn circle(&mut self, x: f64, y: f64) -> CircleBuilder<'_> {
        let builder = CircleBuilder::new(self, x, y);
        builder
    }

    pub fn line(&mut self, start: Vec2d, end: Vec2d) -> LineBuilder<'_> {
        let builder = LineBuilder::new(self, start, end);
        builder
    }

    pub fn char(&mut self, pos: Vec2d, dims: Vec2d, c: char, font: usize, color: Color) {
        self.commands.push(RenderCommand::Char(CharCommand {
            pos,
            dims,
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

            let color = Color::WHITE;

            if ch != ' ' {
                self.char(Vec2d::new(xt, yt), Vec2d::new(w, h), ch, font_id, color);
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

pub struct CircleBuilder<'a> {
    commands: &'a mut RenderCommands,
    x: f64,
    y: f64,
    r: f64,
    color: Color,
}

impl<'a> CircleBuilder<'a> {
    fn new(commands: &'a mut RenderCommands, x: f64, y: f64) -> Self {
        Self {
            commands,
            x,
            y,
            r: 50.0,
            color: Color::new(0.0, 0.3, 1.0, 0.8),
        }
    }

    pub fn radius(&mut self, radius: f64) -> &mut Self {
        self.r = radius;
        self
    }

    pub fn diameter(&mut self, diameter: f64) -> &mut Self {
        self.r = diameter / 2.0;
        self
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
}

impl<'a> Drop for CircleBuilder<'a> {
    fn drop(&mut self) {
        let circle = CircleCommand {
            x: self.x,
            y: self.y,
            radius: self.r,
            color: self.color,
        };
        self.commands.commands.push(RenderCommand::Circle(circle));
    }
}

pub struct LineBuilder<'a> {
    commands: &'a mut RenderCommands,
    start: Vec2d,
    end: Vec2d,
    thickness: f64,
    color: Color,
}

impl<'a> LineBuilder<'a> {
    fn new(commands: &'a mut RenderCommands, start: Vec2d, end: Vec2d) -> Self {
        Self {
            commands,
            start,
            end,
            thickness: 10.0,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn thickness(&mut self, thickness: f64) -> &mut Self {
        self.thickness = thickness;
        self
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
}

impl<'a> Drop for LineBuilder<'a> {
    fn drop(&mut self) {
        let line = LineCommand {
            start: self.start,
            end: self.end,
            thickness: self.thickness,
            color: self.color,
        };
        self.commands.commands.push(RenderCommand::Line(line));
    }
}
