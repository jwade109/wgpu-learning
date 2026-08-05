use glm::*;

use crate::{
    model::{Camera, CameraControls},
    renderer_backend::mat4_identity,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum TextureOrChar {
    Texture(usize),
    Char(usize, char),
    Color,
}

impl TextureOrChar {
    pub fn id(&self) -> Option<usize> {
        match self {
            TextureOrChar::Char(id, _) => Some(*id),
            TextureOrChar::Texture(id) => Some(*id),
            _ => None,
        }
    }

    pub fn char(&self) -> Option<char> {
        match self {
            TextureOrChar::Char(_, c) => Some(*c),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum EntityKind {
    Mesh,
    ScreenRect {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        tex_or_char: TextureOrChar,
    },
}

pub struct Object {
    pub position: Vec3,
    pub angle: f32,
    pub vel: f32,
    pub kind: EntityKind,
    pub mesh_id: usize,
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

pub struct ScreenText {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub c: char,
    pub font: usize,
}

pub struct World {
    pub time: f32,
    pub quads: Vec<Object>,
    pub text: Vec<ScreenText>,
    pub camera: Camera,
}

impl World {
    pub fn new() -> Self {
        World {
            time: 0.0,
            quads: Vec::new(),
            text: Vec::new(),
            camera: Camera::new(),
        }
    }

    pub fn ground_plane(&mut self, x: i32, z: i32, mesh_id: usize) {
        self.quads.push(Object {
            position: Vec3::new(x as f32, 0.0, z as f32),
            angle: 0.0,
            vel: 0.0,
            kind: EntityKind::Mesh,
            should_animate: false,
            mesh_id,
        });
    }

    pub fn ui(&mut self, x: i32, y: i32, w: i32, h: i32, c: char, font: usize) {
        self.text.push(ScreenText {
            x,
            y,
            width: w,
            height: h,
            c,
            font,
        });
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
