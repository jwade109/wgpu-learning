use crate::renderer_backend::*;
use wgpu::*;

pub struct BlurPipeline {
    pipeline: RenderPipeline,
}

impl BlurPipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration, queue: &Queue) -> Self {
        let material_bind_group_layout;
        {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("BlurPipeline Bind Group Layout");
        }

        let mut builder = PipelineBuilder::new(&device);
        let shader = Shader::from_path("src/shaders/blur_shader.wgsl");

        builder.add_bind_group_layout(&material_bind_group_layout);

        let pipeline = builder.build_pipeline::<FullVertex>(
            "Single Texture Pipeline",
            &shader,
            config.format,
            true,
            true,
        );

        Self { pipeline }
    }

    pub fn blur_pass(&self, rp: &mut RenderPass, mesh: &Mesh, material: &BindGroup) {
        rp.set_pipeline(&self.pipeline);
        rp.set_bind_group(0, material, &[]);
        mesh.set_as_active(rp);
        rp.draw_indexed(0..mesh.index_count(), 0, 0..1);
    }
}
