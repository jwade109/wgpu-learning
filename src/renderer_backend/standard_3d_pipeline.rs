use wgpu::{
    util::DeviceExt, BindGroupLayout, Device, Queue, RenderPass, RenderPipeline,
    SurfaceConfiguration,
};

use crate::renderer_backend::{
    bind_group_layout,
    ubo::{SingleUBO, UBO},
    Shader, Texture,
};

pub struct Standard3DPipeline {
    standard: RenderPipeline,
    wireframe: RenderPipeline,
    draw_wireframes: bool,
    camera_ubo: UBO,
    lighting_ubo: SingleUBO,
    pub depth_texture: Texture,
}

impl Standard3DPipeline {
    pub fn new(
        device: &Device,
        ubo_bind_group_layout: &BindGroupLayout,
        material_bind_group_layout: &BindGroupLayout,
        time_etc_data_bind_group: &BindGroupLayout,
        config: &SurfaceConfiguration,
    ) -> Self {
        let depth_texture = Texture::create_depth_texture(&device, config, "depth_texture");

        let camera_projection_bind_group_layout = {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_ubo();
            builder.build("Camera Projection UBO")
        };

        let lighting_bind_group_layout = {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_ubo();
            builder.build("Lighting UBO")
        };

        let mut builder = super::pipeline::Builder::new(&device);
        let shader = Shader::from_path("src/shaders/texture.wgsl");
        builder.add_bind_group_layout(ubo_bind_group_layout);
        builder.add_bind_group_layout(material_bind_group_layout);
        builder.add_bind_group_layout(time_etc_data_bind_group);
        builder.add_bind_group_layout(&camera_projection_bind_group_layout);
        builder.add_bind_group_layout(&lighting_bind_group_layout);
        let standard = builder.build_pipeline("Standard 3D Pipeline", &shader, config.format, true);

        let wireframe = {
            let mut builder = super::pipeline::Builder::new(&device);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.add_bind_group_layout(&material_bind_group_layout);
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.add_bind_group_layout(&camera_projection_bind_group_layout);
            builder.add_bind_group_layout(&lighting_bind_group_layout);
            builder.wireframes();
            builder.build_pipeline("Texture Pipeline", &shader, config.format, true)
        };

        let camera_ubo = UBO::new(&device, 1, camera_projection_bind_group_layout);

        let light_source: [f32; 3] = [4.0, 3.0, 5.0];

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader Params"),
            contents: &bytemuck::bytes_of(&light_source),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let lighting_bind_group = {
            let mut builder = super::bind_group::Builder::new(&device);
            builder.set_layout(&lighting_bind_group_layout);
            builder.add_buffer(&buffer, 0);
            builder.build("uniform buffer")
        };

        Self {
            standard,
            wireframe,
            draw_wireframes: false,
            camera_ubo,
            lighting_ubo: SingleUBO {
                buffer,
                bind_group: lighting_bind_group,
            },
            depth_texture,
        }
    }

    pub fn set_draw_wireframes(&mut self, wireframes: bool) {
        self.draw_wireframes = wireframes;
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        if self.draw_wireframes {
            &self.wireframe
        } else {
            &self.standard
        }
    }

    pub fn redraw_depth_texture(&mut self, device: &Device, config: &SurfaceConfiguration) {
        self.depth_texture = Texture::create_depth_texture(device, config, "depth_texture");
    }

    pub fn set_bindings(&self, rp: &mut RenderPass) {
        rp.set_bind_group(3, self.camera_ubo.bind_group(0), &[]);
        rp.set_bind_group(4, &self.lighting_ubo.bind_group, &[]);
    }

    pub fn upload_camera_matrix(&mut self, view_proj: &glm::Mat4, queue: &Queue) {
        self.camera_ubo.upload(0, view_proj, queue);
    }
}
