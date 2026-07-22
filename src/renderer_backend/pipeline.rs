use std::fs;

#[derive(Default)]
pub struct Shader {
    pub contents: String,
    pub vertex_entry: String,
    pub fragment_entry: String,
}

impl Shader {
    pub fn from_path(path: &str) -> Self {
        let source_code = fs::read_to_string(path).expect("Can't read source code!");
        Self {
            contents: source_code,
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
        }
    }
}

pub struct Builder<'a> {
    shader: Shader,
    pixel_format: wgpu::TextureFormat,
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    device: &'a wgpu::Device,
}

fn make_shader_module(device: &wgpu::Device, shader: &Shader, label: &str) -> wgpu::ShaderModule {
    let desc = wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader.contents.clone().into()),
    };
    device.create_shader_module(desc)
}

impl<'a> Builder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Builder {
            shader: Shader::default(),
            pixel_format: wgpu::TextureFormat::Rgba8Unorm,
            vertex_buffer_layouts: Vec::new(),
            bind_group_layouts: Vec::new(),
            device: device,
        }
    }

    fn reset(&mut self) {
        self.vertex_buffer_layouts.clear();
        self.bind_group_layouts.clear();
    }

    pub fn add_vertex_buffer_layout(&mut self, layout: wgpu::VertexBufferLayout<'static>) {
        self.vertex_buffer_layouts.push(layout);
    }

    pub fn add_bind_group_layout(&mut self, layout: &'a wgpu::BindGroupLayout) {
        self.bind_group_layouts.push(layout);
    }

    pub fn set_shader_module(&mut self, shader: Shader) {
        self.shader = shader;
    }

    pub fn set_pixel_format(&mut self, pixel_format: wgpu::TextureFormat) {
        self.pixel_format = pixel_format;
    }

    pub fn build(&mut self, label: &str) -> wgpu::RenderPipeline {
        let shader = make_shader_module(&self.device, &self.shader, "Shader Module");

        let pipeline_layout = {
            let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &self.bind_group_layouts,
                push_constant_ranges: &[],
            };
            self.device
                .create_pipeline_layout(&pipeline_layout_descriptor)
        };

        let render_targets = [Some(wgpu::ColorTargetState {
            format: self.pixel_format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),

            cache: None,

            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some(&self.shader.vertex_entry),
                buffers: &self.vertex_buffer_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(&self.shader.fragment_entry),
                targets: &render_targets,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),

            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        };

        let pipeline = self
            .device
            .create_render_pipeline(&render_pipeline_descriptor);

        self.reset();

        pipeline
    }
}
