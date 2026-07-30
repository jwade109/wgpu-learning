use glm::*;

pub enum MeshType {
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
        let c0 = glm::Vec4::new(1.0, 0.0, 0.0, 0.0);
        let c1 = glm::Vec4::new(0.0, 1.0, 0.0, 0.0);
        let c2 = glm::Vec4::new(0.0, 0.0, 1.0, 0.0);
        let c3 = glm::Vec4::new(0.0, 0.0, 0.0, 1.0);
        let m1 = glm::Matrix4::new(c0, c1, c2, c3);
        let m2 = glm::Matrix4::new(c0, c1, c2, c3);

        let matrix = ext::translate(&m1, self.position)
            * ext::rotate(&m2, self.angle, glm::Vector3::new(0.0, 0.0, 1.0));

        matrix
    }
}

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            target: Vec3::new(0.0, 0.0, 0.0),
            eye: Vec3::new(8.0, 19.0, 5.0),
        }
    }

    pub fn to_projection_matrix(&self, window: &glfw::Window) -> Mat4 {
        let up = normalize(Vec3::new(0.0, 1.0, 0.0));

        let zaxis = normalize(self.eye - self.target); // forward vector
        let xaxis = normalize(cross(up, zaxis)); // The "right" vector.
        let yaxis = normalize(cross(zaxis, xaxis)); // The "up" vector.

        let orientation = Matrix4::new(
            Vec4::new(xaxis.x, yaxis.x, zaxis.x, 0.0),
            Vec4::new(xaxis.y, yaxis.y, zaxis.y, 0.0),
            Vec4::new(xaxis.z, yaxis.z, zaxis.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        let translation = Matrix4::new(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(-self.eye.x, -self.eye.y, -self.eye.z, 1.0),
        );

        let view = orientation * translation;

        let fov_y: f32 = radians(50.0);
        let (sx, sy) = window.get_size();
        let aspect = sx as f32 / sy as f32;
        let z_near = 0.1;
        let z_far = 100.0;
        let projection = ext::perspective(fov_y, aspect, z_near, z_far);

        projection * view
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

    pub fn update(&mut self, dt: f32) {
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

        let z = (self.time / 15.0).sin() * 20.0;
        let y = (self.time / 11.0).sin() * 4.0 + 10.0;
        let x = (self.time / 15.0).cos() * 20.0;

        let target = Vec3::new(0.0, 0.0, 0.0);
        let eye = Vec3::new(x, y, z);

        self.camera.target = target;
        self.camera.eye = eye;
    }
}
