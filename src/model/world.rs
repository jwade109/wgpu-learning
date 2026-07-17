use super::game_object::Object;

pub struct World {
    objects: Vec<Object>,
}

impl World {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        for obj in &mut self.objects {
            obj.angle += dt * 0.01;
            if obj.angle > 360.0 {
                obj.angle -= 360.0;
            }
        }
    }
}
