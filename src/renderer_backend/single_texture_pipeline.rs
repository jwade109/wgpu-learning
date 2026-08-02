use crate::renderer_backend::*;
use wgpu::*;

pub struct SingleTexturePipeline {
    pipeline: RenderPipeline,
    transforms_ubo: UBO<glm::Mat4>,
}

impl SingleTexturePipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let transforms_ubo_bind_group_layout = {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_ubo();
            builder.build("Transforms")
        };

        let material_bind_group_layout;
        {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("SpriteMaterial Bind Group Layout");
        }

        let mut builder = PipelineBuilder::new(&device);
        let shader = Shader::from_path("src/shaders/single_texture.wgsl");
        builder.add_bind_group_layout(&transforms_ubo_bind_group_layout);
        builder.add_bind_group_layout(&material_bind_group_layout);
        let pipeline = builder.build_pipeline::<FullVertex>(
            "Single Texture Pipeline",
            &shader,
            config.format,
            true,
            true,
        );

        let transforms_ubo = UBO::new(&device, 250, transforms_ubo_bind_group_layout);

        Self {
            pipeline,
            transforms_ubo,
        }
    }

    pub fn draw(
        &mut self,
        rp: &mut RenderPass,
        mesh: &Mesh,
        material: &SpriteMaterial,
        transform: &glm::Mat4,
        queue: &Queue,
        i: u64,
    ) {
        rp.set_pipeline(&self.pipeline);
        self.transforms_ubo.upload(i, transform, queue);
        rp.set_bind_group(0, self.transforms_ubo.bind_group(i as usize), &[]);
        rp.set_bind_group(1, material.bind_group(), &[]);
        draw_mesh(rp, mesh);
    }
}
