use glm::*;

pub struct Object {
    pub position: Vec3,
    pub angle: f32,
    pub vel: f32,
    pub n_sides: usize,
}

pub struct Camera {
    pub position: Vec3,
    pub target_position: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        let position = Vec3::new(0.5, 0.7, 0.0);

        Self {
            position,
            target_position: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn to_projection_mat() -> Mat4 {
        num_traits::identities::one()
    }
}
