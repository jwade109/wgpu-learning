use crate::renderer_backend::*;
use glm::{Mat4, Vec4};
use wgpu::*;

pub struct CirclePipeline {
    pipeline: RenderPipeline,
    colors: BufferResource,
    transforms: BufferResource,
    radius: BufferResource,
    mesh: Mesh,
}

impl CirclePipeline {
    pub const MAX_CHARS_PER_PASS: usize = 480;

    pub fn new(device: &Device, config: &SurfaceConfiguration, queue: &Queue) -> Self {
        let colors = make_array_resource(device, Self::MAX_CHARS_PER_PASS, 16);
        let transforms = make_array_resource(device, Self::MAX_CHARS_PER_PASS, 64);

        // the stride is 16 here because that's the minimum.
        // the element is 4 bytes large
        let radius = make_array_resource(device, Self::MAX_CHARS_PER_PASS, 16);

        let mesh = make_quad(device);

        let mut builder = PipelineBuilder::new(&device);
        let shader = Shader::from_path("src/shaders/circle.wgsl");

        builder.add_bind_group_layout(&colors.layout);
        builder.add_bind_group_layout(&transforms.layout);
        builder.add_bind_group_layout(&radius.layout);

        let pipeline = builder.build_pipeline::<FullVertex>(
            "Circle Pipeline",
            &shader,
            config.format,
            true,
            true,
        );

        for i in 0..Self::MAX_CHARS_PER_PASS {
            let color = [1.0f32, 1.0, 1.0, 1.0];
            queue.write_buffer(&colors.buffer, 16 * i as u64, any_as_u8_slice(&color));
        }

        Self {
            pipeline,
            colors,
            transforms,
            radius,
            mesh,
        }
    }

    pub fn set_color(&self, queue: &Queue, i: usize, color: Vec4) {
        queue.write_buffer(&self.colors.buffer, 16 * i as u64, any_as_u8_slice(&color));
    }

    pub fn set_transform(&self, queue: &Queue, i: usize, transform: &Mat4) {
        queue.write_buffer(
            &self.transforms.buffer,
            64 * i as u64,
            any_as_u8_slice(transform),
        );
    }

    pub fn set_radius(&self, queue: &Queue, i: usize, radius: f32) {
        // the stride is 16 here because that's the minimum.
        // the element is 4 bytes large
        queue.write_buffer(&self.radius.buffer, 16 * i as u64, any_as_u8_slice(&radius));
    }

    pub fn assign_buffer_data(&self, queue: &Queue, commands: &[CircleCommand], sx: f64, sy: f64) {
        for (i, cmd) in commands.iter().enumerate() {
            let ul_x = cmd.x - cmd.radius;
            let ul_y = cmd.y - cmd.radius;
            let transform = screen_space_transform(ul_x, ul_y, cmd.radius * 2.0, cmd.radius * 2.0, sx, sy, 0.0);
            self.set_transform(queue, i, &transform);
            self.set_color(queue, i, cmd.color);
            self.set_radius(queue, i, cmd.radius as f32);
        }
    }

    pub fn draw_circles(&self, rp: &mut RenderPass, n: usize) {
        rp.set_pipeline(&self.pipeline);

        rp.set_bind_group(0, &self.colors.bind_group, &[]);
        rp.set_bind_group(1, &self.transforms.bind_group, &[]);
        rp.set_bind_group(2, &self.radius.bind_group, &[]);

        let n = n.min(Self::MAX_CHARS_PER_PASS);

        self.mesh.set_as_active(rp);
        rp.draw_indexed(0..self.mesh.index_count(), 0, 0..n as u32);
    }
}
