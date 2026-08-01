use crate::renderer_backend::*;
use wgpu::*;

pub struct LavaLampPipeline {
    pipeline: RenderPipeline,
    camera_ubo: UBO<glm::Mat4>,
}

impl LavaLampPipeline {
    pub fn new(
        device: &Device,
        time_etc_data_bind_group: &BindGroupLayout,
        config: &SurfaceConfiguration,
    ) -> Self {
        let camera_projection_bind_group_layout = {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_ubo();
            builder.build("Camera Projection UBO")
        };

        let mut builder = PipelineBuilder::new(&device);
        let shader = Shader::from_path("src/shaders/cells.wgsl");
        builder.add_bind_group_layout(time_etc_data_bind_group);
        builder.add_bind_group_layout(&camera_projection_bind_group_layout);
        let pipeline =
            builder.build_pipeline("Lava Lamp Pipeline", &shader, config.format, true, true);

        let camera_ubo = UBO::new(&device, 250, camera_projection_bind_group_layout);

        Self {
            pipeline,
            camera_ubo,
        }
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn draw(
        &mut self,
        rp: &mut RenderPass,
        mesh: &Mesh,
        transform: &glm::Mat4,
        shader_params: &SingleUBO,
        queue: &Queue,
        i: u64,
    ) {
        rp.set_pipeline(self.pipeline());
        self.camera_ubo.upload(i, transform, queue);
        rp.set_bind_group(0, &shader_params.bind_group, &[]);
        rp.set_bind_group(1, self.camera_ubo.bind_group(i as usize), &[]);
        draw_mesh(rp, mesh);
    }
}
