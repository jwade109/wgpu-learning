use crate::renderer_backend::*;
use wgpu::*;

pub struct SingleColorPipeline {
    pipeline: RenderPipeline,
    transforms_ubo: UBO<glm::Mat4>,
    color_ubo: UBO<glm::Vec4>,
}

impl SingleColorPipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let transforms_ubo_bind_group_layout = {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_ubo();
            builder.build("Shader Params")
        };

        let color_ubo_bind_group_layout = {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_ubo();
            builder.build("Shader Params")
        };

        let mut builder = PipelineBuilder::new(&device);
        let shader = Shader::from_path("src/shaders/single_color.wgsl");
        builder.add_bind_group_layout(&transforms_ubo_bind_group_layout);
        builder.add_bind_group_layout(&color_ubo_bind_group_layout);
        let pipeline =
            builder.build_pipeline("Lava Lamp Pipeline", &shader, config.format, true, true);

        let transforms_ubo = UBO::new(&device, 250, transforms_ubo_bind_group_layout);
        let color_ubo = UBO::new(&device, 250, color_ubo_bind_group_layout);

        Self {
            pipeline,
            transforms_ubo,
            color_ubo,
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
        color: &glm::Vec4,
        queue: &Queue,
        i: u64,
    ) {
        rp.set_pipeline(self.pipeline());
        self.transforms_ubo.upload(i, transform, queue);
        self.color_ubo.upload(i, color, queue);
        rp.set_bind_group(0, self.transforms_ubo.bind_group(i as usize), &[]);
        rp.set_bind_group(1, self.color_ubo.bind_group(i as usize), &[]);
        draw_mesh(rp, mesh);
    }
}
