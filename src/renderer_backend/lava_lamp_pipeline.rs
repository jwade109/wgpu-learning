use crate::renderer_backend::*;
use wgpu::*;

pub struct LavaLampPipeline {
    pipeline: RenderPipeline,
}

impl LavaLampPipeline {
    pub fn new(
        device: &Device,
        time_etc_data_bind_group: &BindGroupLayout,
        config: &SurfaceConfiguration,
    ) -> Self {
        let mut builder = PipelineBuilder::new(&device);
        let shader = Shader::from_path("src/shaders/cells.wgsl");
        builder.add_bind_group_layout(time_etc_data_bind_group);
        let pipeline =
            builder.build_pipeline("Lava Lamp Pipeline", &shader, config.format, true, true);

        Self { pipeline }
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn draw(&self, rp: &mut RenderPass, mesh: &Mesh, shader_params: &SingleUBO) {
        rp.set_pipeline(self.pipeline());
        rp.set_bind_group(0, &shader_params.bind_group, &[]);
        draw_mesh(rp, mesh);
    }
}
