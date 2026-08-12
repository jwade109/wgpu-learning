use crate::{model::*, renderer_backend::*};
use enum_iterator::Sequence;
use glm::*;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

#[derive(PartialEq, Eq, Sequence)]
pub enum ViewSelector {
    Map,
    Lava,
    World3d,
}

pub struct Renderer<'a> {
    pub paused: bool,
    pub view_selector: ViewSelector,
    pub draw_wireframes: bool,
    mouse_pos_smoothed: [f32; 2],

    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    pub window: &'a mut glfw::Window,
    _lava_lamp_pipeline: LavaLampPipeline,
    blur_pipeline: BlurPipeline,
    map_pipeline: wgpu::RenderPipeline,
    standard_3d_pipeline: Standard3DPipeline,
    single_color_pipeline: SingleColorPipeline,
    text_pipeline: TextPipeline,
    circle_pipeline: CirclePipeline,

    standard_quad: Mesh,
    pub fonts: HashMap<usize, (FontInfo, SpriteMaterial)>,
    meshes: HashMap<usize, Mesh>,
    textures: HashMap<usize, SpriteMaterial>,
    next_resource_id: usize,

    depth_texture: Texture,
    intermediate_texture: Texture,
    intermediate_texture_2: Texture,

    common_shader_info: SingleUBO,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a mut glfw::Window) -> Self {
        let size = window.get_framebuffer_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };
        let instance = wgpu::Instance::new(instance_descriptor);
        let surface = instance.create_surface(window.render_context()).unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::POLYGON_MODE_LINE
                | wgpu::Features::POLYGON_MODE_POINT
                | wgpu::Features::BUFFER_BINDING_ARRAY,
            required_limits: wgpu::Limits {
                max_bind_groups: 8,
                ..Default::default()
            },
            memory_hints: wgpu::MemoryHints::Performance,
            label: Some("Device"),
        };
        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let bgl = material_bind_group_layout(&device, "SpriteMaterial Bind Group Layout");

        let ubo_bind_group_layout;
        {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_ubo();
            ubo_bind_group_layout = builder.build("UBO Bind Group Layout");
        }

        let shader_params = ShaderParams {
            mouse: (450.0, 360.0),
            time: 0.0,
            resolution: (100.0, 100.0),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader Params"),
            contents: &shader_params.to_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
        let intermediate_texture =
            Texture::create_intermediate_texture(&device, &config, "intermediate_texture");
        let intermediate_texture_2 =
            Texture::create_intermediate_texture(&device, &config, "intermediate_texture_2");

        let time_etc_data_bind_group =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_data_bind_group"),
            });

        let lava_lamp_pipeline = LavaLampPipeline::new(&device, &time_etc_data_bind_group, &config);

        let map_pipeline = {
            let mut builder = PipelineBuilder::new(&device);
            let shader = Shader::from_path("src/shaders/map.wgsl");
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.build_pipeline::<FullVertex>("Map Pipeline", &shader, config.format, true, true)
        };

        let standard_3d_pipeline = Standard3DPipeline::new(
            &device,
            &ubo_bind_group_layout,
            &bgl,
            &time_etc_data_bind_group,
            &config,
        );

        let single_color_pipeline = SingleColorPipeline::new(&device, &config);

        let uniform_bind_group = {
            let mut builder = BindGroupBuilder::new(&device);
            builder.set_layout(&time_etc_data_bind_group);
            builder.add_buffer(&uniform_buffer, 0);
            builder.build("uniform buffer")
        };

        let text_pipeline = TextPipeline::new(&device, &config, &queue);

        let blur_pipeline = BlurPipeline::new(&device, &config);

        let standard_quad = make_quad(&device);

        let circle_pipeline = CirclePipeline::new(&device, &config, &queue);

        Self {
            paused: false,
            mouse_pos_smoothed: [0.0, 0.0],
            instance,
            window,
            surface,
            device,
            queue,
            config,
            _lava_lamp_pipeline: lava_lamp_pipeline,
            blur_pipeline,
            map_pipeline,
            standard_3d_pipeline,
            single_color_pipeline,
            text_pipeline,
            circle_pipeline,
            fonts: HashMap::new(),
            meshes: HashMap::new(),
            textures: HashMap::new(),
            next_resource_id: 0,
            common_shader_info: SingleUBO {
                buffer: uniform_buffer,
                bind_group: uniform_bind_group,
            },
            standard_quad,
            view_selector: ViewSelector::World3d,
            draw_wireframes: false,
            depth_texture,
            intermediate_texture,
            intermediate_texture_2,
        }
    }

    pub fn spawn_mesh(&mut self, mesh: Mesh) -> usize {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.meshes.insert(id, mesh);
        id
    }

    pub fn load_texture(&mut self, path: &str) -> usize {
        let sprite = SpriteMaterial::load(path, &self.device, &self.queue);
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.textures.insert(id, sprite);
        id
    }

    pub fn load_font(&mut self, name: &str) -> usize {
        println!("Loading font {name}");
        let data_path = format!("fonts/{name}/font_data.json");
        let texture_path = format!("fonts/{name}/font.png");
        let texture = SpriteMaterial::load(&texture_path, &self.device, &self.queue);
        let font = FontInfo::from_file(&data_path).unwrap();
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.fonts.insert(id, (font, texture));
        id
    }

    pub fn spawn_ground_plane(&mut self, x: i32, z: i32, n_quads: u16) -> usize {
        let mesh = make_rough_ground_plane(&self.device, Vec2::new(x as f32, z as f32), n_quads);
        self.spawn_mesh(mesh)
    }

    pub fn resize(&mut self, new_size: (i32, i32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.config.width = new_size.0 as u32;
            self.config.height = new_size.1 as u32;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.intermediate_texture = Texture::create_intermediate_texture(
                &self.device,
                &self.config,
                "intermediate_texture",
            );
            self.intermediate_texture_2 = Texture::create_intermediate_texture(
                &self.device,
                &self.config,
                "intermediate_texture_2",
            );
        }
    }

    pub fn update_surface(&mut self) {
        self.surface = self
            .instance
            .create_surface(self.window.render_context())
            .unwrap();
    }

    fn draw_lava(&self, view: &wgpu::TextureView) {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

        let transform = mat4_identity();
        self._lava_lamp_pipeline.draw(
            &mut rp,
            &self.standard_quad,
            &transform,
            &self.common_shader_info,
            &self.queue,
            0,
        );

        drop(rp);
        self.queue.submit(std::iter::once(command_encoder.finish()));
    }

    fn draw_map(&self, view: &wgpu::TextureView) {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

        rp.set_pipeline(&self.map_pipeline);
        rp.set_bind_group(0, &self.common_shader_info.bind_group, &[]);

        draw_mesh(&mut rp, &self.standard_quad);

        drop(rp);
        self.queue.submit(std::iter::once(command_encoder.finish()));
    }

    fn draw_3d(&self, world: &World, view: &wgpu::TextureView) {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rp = self.get_render_pass(&mut command_encoder, Some(wgpu::Color::BLACK), &view);

        rp.set_pipeline(self.standard_3d_pipeline.pipeline());
        rp.set_bind_group(2, &self.common_shader_info.bind_group, &[]);
        self.standard_3d_pipeline.set_bindings(&mut rp);

        for i in 0..world.quads.len() {
            let matrix = world.quads[i].get_transform_matrix();
            self.standard_3d_pipeline
                .upload_transform(i as u64, &matrix, &self.queue);
        }

        rp.set_bind_group(1, &self.textures.values().next().unwrap().bind_group, &[]);

        for i in 0..world.quads.len() {
            let bg = self.standard_3d_pipeline.transforms().bind_group(i);

            let Some(mesh) = self.meshes.get(&world.quads[i].mesh_id) else {
                println!("Failed to get mesh of type {:?}", world.quads[i].mesh_id);
                continue;
            };

            mesh.set_as_active(&mut rp);

            rp.set_bind_group(0, bg, &[]);
            rp.draw_indexed(0..mesh.index_count(), 0, 0..1);
        }

        drop(rp);

        self.queue.submit(std::iter::once(command_encoder.finish()));
    }

    fn draw_circles(&self, view: &wgpu::TextureView, commands: &RenderCommands) {
        let (sx, sy) = self.window.get_size();
        let commands: Vec<CircleCommand> = commands
            .commands()
            .filter_map(|e: &RenderCommand| match e {
                RenderCommand::Circle(c) => Some(*c),
                _ => None,
            })
            .collect();

        for chunk in commands.chunks(CirclePipeline::MAX_CHARS_PER_PASS) {
            let mut command_encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

            self.circle_pipeline
                .assign_buffer_data(&self.queue, chunk, sx as f64, sy as f64);

            self.circle_pipeline.draw_circles(&mut rp, chunk.len());

            drop(rp);

            self.queue.submit(std::iter::once(command_encoder.finish()));
        }
    }

    fn draw_rectangles(&self, view: &wgpu::TextureView, commands: &RenderCommands) {
        let (sx, sy) = self.window.get_size();

        let commands: Vec<RectCommand> = commands
            .commands()
            .filter_map(|e: &RenderCommand| match e {
                RenderCommand::Rect(c) => Some(*c),
                _ => None,
            })
            .collect();

        for cmd in commands {
            let mut command_encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

            let tf = screen_space_transform(
                cmd.x, cmd.y, cmd.width, cmd.height, sx as f64, sy as f64, cmd.angle,
            );

            self.single_color_pipeline.draw(
                &mut rp,
                &self.standard_quad,
                &tf,
                &cmd.color,
                &self.queue,
            );

            drop(rp);

            self.queue.submit(std::iter::once(command_encoder.finish()));
        }
    }

    fn draw_ui(&self, view: &wgpu::TextureView, commands: &RenderCommands) {
        let (sx, sy) = self.window.get_size();

        let commands: Vec<CharCommand> = commands
            .commands()
            .filter_map(|e: &RenderCommand| match e {
                RenderCommand::Char(c) => Some(*c),
                _ => None,
            })
            .collect();

        let obj = commands.iter().next().unwrap();
        let (font, material) = self.fonts.get(&obj.font).unwrap();

        for chunk in commands.chunks(TextPipeline::MAX_CHARS_PER_PASS) {
            let mut command_encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

            self.text_pipeline
                .assign_buffer_data(&self.queue, chunk, font, sx as f64, sy as f64);
            self.text_pipeline
                .draw_text(&mut rp, &self.standard_quad, material, chunk.len());

            drop(rp);

            self.queue.submit(std::iter::once(command_encoder.finish()));
        }
    }

    pub fn update(&mut self, world: &World) {
        let view_proj = world.camera.to_projection_matrix(self.window);
        self.standard_3d_pipeline
            .upload_camera_matrix(&view_proj, &self.queue);

        let mouse_pos = self.window.get_cursor_pos();

        self.mouse_pos_smoothed[0] += (mouse_pos.0 as f32 - self.mouse_pos_smoothed[0]) * 0.06;
        self.mouse_pos_smoothed[1] += (mouse_pos.1 as f32 - self.mouse_pos_smoothed[1]) * 0.06;

        let shader_params = ShaderParams {
            mouse: (self.mouse_pos_smoothed[0], self.mouse_pos_smoothed[1]),
            time: world.time,
            resolution: (
                self.window.get_size().0 as f32,
                self.window.get_size().1 as f32,
            ),
        };

        self.queue.write_buffer(
            &self.common_shader_info.buffer,
            0,
            &shader_params.to_bytes(),
        );
    }

    fn get_render_pass<'b>(
        &self,
        command_encoder: &'b mut wgpu::CommandEncoder,
        clear_color: Option<wgpu::Color>,
        view: &wgpu::TextureView,
    ) -> wgpu::RenderPass<'b> {
        let depth_stencil_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        });

        let load = clear_color.map_or(wgpu::LoadOp::Load, |c| wgpu::LoadOp::Clear(c));

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        command_encoder.begin_render_pass(&render_pass_descriptor)
    }

    fn blur_pass(&self, incoming: &Texture, outgoing: &wgpu::TextureView) {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rp = self.get_render_pass(&mut command_encoder, None, &outgoing);

        self.blur_pipeline
            .blur_pass(&mut rp, &self.standard_quad, &incoming.bind_group);

        drop(rp);
        self.queue.submit(std::iter::once(command_encoder.finish()));
    }

    pub fn render(
        &mut self,
        world: &mut World,
        commands: &RenderCommands,
    ) -> Result<(), wgpu::SurfaceError> {
        let (w, h) = self.window.get_size();

        if w == 0 || h == 0 {
            return Ok(());
        }

        self.device.poll(wgpu::Maintain::wait());

        {
            let event = self.queue.submit([]);
            let maintain = wgpu::Maintain::WaitForSubmissionIndex(event);
            self.device.poll(maintain);
        }

        let drawable = self.surface.get_current_texture()?;

        let view = drawable
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.standard_3d_pipeline
            .set_draw_wireframes(self.draw_wireframes);

        match self.view_selector {
            ViewSelector::Map => {
                self.draw_map(&view);
            }
            ViewSelector::Lava => {
                self.draw_lava(&view);
            }
            ViewSelector::World3d => {
                if self.draw_wireframes {
                    self.draw_3d(&world, &view);
                    // self.blur_pass(&self.intermediate_texture_2, &view);
                } else {
                    self.draw_3d(&world, &self.intermediate_texture.view);
                    // self.draw_rectangles(&view, commands);
                    self.draw_circles(&view, commands);
                    // self.blur_pass(&self.intermediate_texture, &view);
                    self.draw_ui(&view, commands);
                }
            }
        }

        self.device.poll(wgpu::Maintain::wait());

        drawable.present();

        Ok(())
    }
}
