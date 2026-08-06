#[derive(Debug, Clone, Copy)]
pub enum RenderCommand {
    Char(CharCommand),
}

#[derive(Debug, Clone, Copy)]
pub struct CharCommand {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub c: char,
    pub font: usize,
}

pub struct RenderCommands {
    commands: Vec<RenderCommand>,
}

impl RenderCommands {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn commands(&self) -> impl Iterator<Item = &RenderCommand> {
        self.commands.iter()
    }

    pub fn char(&mut self, x: i32, y: i32, w: i32, h: i32, c: char, font: usize) {
        self.commands.push(RenderCommand::Char(CharCommand {
            x,
            y,
            width: w,
            height: h,
            c,
            font,
        }));
    }
}
