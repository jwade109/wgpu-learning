use glm::*;

pub enum RenderCommand {
    Mesh(MeshCommand),
}

pub struct MeshCommand {
    id: usize,
    position: Vec3,
}

pub struct RenderCommands {
    commands: Vec<RenderCommand>,
}

impl RenderCommands {
    pub fn add(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }
}
