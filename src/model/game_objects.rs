use glm::*;

use crate::{model::{Camera, CameraControls}, renderer_backend::mat4_identity};

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum MeshType {
    Quad,
    Polygon(usize),
    Cube,
    GroundPlane,
}

pub struct Object {
    pub position: Vec3,
    pub angle: f32,
    pub vel: f32,
    pub mesh_type: MeshType,
    pub should_animate: bool,
}

impl Object {
    pub fn get_transform_matrix(&self) -> Matrix4<f32> {
        let eye = mat4_identity();
        let matrix = ext::translate(&eye, self.position)
            * ext::rotate(&eye, self.angle, glm::Vector3::new(0.0, 0.0, 1.0));

        matrix
    }
}

pub struct World {
    pub time: f32,
    pub quads: Vec<Object>,
    pub camera: Camera,
}

impl World {
    pub fn new() -> Self {
        World {
            time: 0.0,
            quads: Vec::new(),
            camera: Camera::new(),
        }
    }

    pub fn update(&mut self, dt: f32, ctrls: &CameraControls) {
        self.time += dt;
        for quad in &mut self.quads {
            quad.angle = quad.angle + quad.vel * dt;
            if quad.angle > 360.0 {
                quad.angle -= 360.0;
            }

            if quad.should_animate {
                let a = quad.position.x * 0.6 + quad.position.z * 0.4 + self.time / 2.0;
                quad.position.y = a.sin() + 1.0;
            }
        }

        self.camera.apply_controls(ctrls);
        self.camera.update();

        // let z = (self.time / 15.0).sin() * 20.0;
        // let y = (self.time / 11.0).sin() * 4.0 + 10.0;
        // let x = (self.time / 15.0).cos() * 20.0;

        // let target = Vec3::new(0.0, 0.0, 0.0);
        // let eye = Vec3::new(x, y, z);

        // self.camera.target = target;
        // self.camera.eye = eye;
    }
}
