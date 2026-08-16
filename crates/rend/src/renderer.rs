use crate::*;
use enum_iterator::Sequence;
use glm::*;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum ViewSelector {
    Map,
    Lava,
    World3d,
}

pub struct MeshObject {
    pub position: Vec3,
    pub angle: f32,
    pub vel: f32,
    pub mesh_id: usize,
    pub should_animate: bool,
}

impl MeshObject {
    pub fn get_transform_matrix(&self) -> Matrix4<f32> {
        let eye = mat4_identity();
        let matrix = ext::translate(&eye, self.position)
            * ext::rotate(&eye, self.angle, glm::Vector3::new(0.0, 0.0, 1.0));

        matrix
    }
}

pub struct Renderer<'a> {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'a> Renderer<'a> {
    async fn new(window: &mut glfw::Window) -> Self {
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };
        let instance = wgpu::Instance::new(instance_descriptor);
        let surface = instance.create_surface(window.render_context()).unwrap();

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

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

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

        let size = window.get_framebuffer_size();

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

        Renderer {
            instance,
            surface,
            device,
            queue,
            config,
        }
    }
}

struct Pipelines {
    lava_lamp_pipeline: LavaLampPipeline,
    blur_pipeline: BlurPipeline,
    map_pipeline: wgpu::RenderPipeline,
    standard_3d_pipeline: Standard3DPipeline,
    single_color_pipeline: SingleColorPipeline,
    text_pipeline: TextPipeline,
    circle_pipeline: CirclePipeline,
    line_pipeline: LinePipeline,
}

pub struct RenderState<'a> {
    pub renderer: Renderer<'a>,

    pipelines: Pipelines,

    pub window: &'a mut glfw::Window,

    standard_quad: Mesh,
    pub fonts: HashMap<usize, (FontInfo, SpriteMaterial)>,
    meshes: HashMap<usize, Mesh>,
    textures: HashMap<usize, SpriteMaterial>,
    next_resource_id: usize,

    depth_texture: Texture,
    im_tex_1: Texture,
    im_tex_2: Texture,

    common_shader_info: SingleUBO,
}

impl<'a> RenderState<'a> {
    pub async fn new(window: &'a mut glfw::Window) -> Self {
        let renderer = Renderer::new(window).await;

        let bgl = material_bind_group_layout(&renderer.device, "SpriteMaterial Bind Group Layout");

        let ubo_bind_group_layout = {
            let mut builder = BindGroupLayoutBuilder::new(&renderer.device);
            builder.add_ubo();
            builder.build("UBO Bind Group Layout")
        };

        let uniform_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shader Params"),
            size: 40,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let depth_texture = Texture::depth_texture(&renderer, "depth_texture");
        let im_tex_1 = Texture::blank_texture(&renderer, "im_tex_1");
        let im_tex_2 = Texture::blank_texture(&renderer, "im_tex_2");

        let time_etc_data_bind_group =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let single_color_pipeline = SingleColorPipeline::new(&renderer.device, &renderer.config);

        let uniform_bind_group = {
            let mut builder = BindGroupBuilder::new(&renderer.device);
            builder.set_layout(&time_etc_data_bind_group);
            builder.add_buffer(&uniform_buffer, 0);
            builder.build("uniform buffer")
        };

        let standard_quad = make_quad(&renderer.device);

        let text_pipeline = TextPipeline::new(&renderer);
        let blur_pipeline = BlurPipeline::new(&renderer.device, &renderer.config);
        let circle_pipeline = CirclePipeline::new(&renderer);
        let line_pipeline = LinePipeline::new(&renderer.device, &renderer.config);
        let standard_3d_pipeline = Standard3DPipeline::new(
            &renderer.device,
            &ubo_bind_group_layout,
            &bgl,
            &time_etc_data_bind_group,
            &renderer.config,
        );
        let lava_lamp_pipeline = LavaLampPipeline::new(
            &renderer.device,
            &time_etc_data_bind_group,
            &renderer.config,
        );
        let map_pipeline = {
            let mut builder = PipelineBuilder::new(&renderer.device);
            let shader = Shader::from_path("crates/rend/shaders/map.wgsl");
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.build_pipeline::<FullVertex>(
                "Map Pipeline",
                &shader,
                renderer.config.format,
                true,
                true,
            )
        };

        Self {
            renderer,
            window,
            pipelines: Pipelines {
                lava_lamp_pipeline,
                blur_pipeline,
                map_pipeline,
                standard_3d_pipeline,
                single_color_pipeline,
                text_pipeline,
                circle_pipeline,
                line_pipeline,
            },
            fonts: HashMap::new(),
            meshes: HashMap::new(),
            textures: HashMap::new(),
            next_resource_id: 0,
            common_shader_info: SingleUBO {
                buffer: uniform_buffer,
                bind_group: uniform_bind_group,
            },
            standard_quad,
            depth_texture,
            im_tex_1,
            im_tex_2,
        }
    }

    pub fn spawn_mesh(&mut self, mesh: Mesh) -> usize {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.meshes.insert(id, mesh);
        id
    }

    pub fn load_texture(&mut self, path: &str) -> usize {
        let sprite = SpriteMaterial::load(path, &self.renderer);
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.textures.insert(id, sprite);
        id
    }

    pub fn load_font(&mut self, name: &str) -> usize {
        println!("Loading font {name}");
        let data_path = format!("fonts/{name}/font_data.json");
        let texture_path = format!("fonts/{name}/font.png");
        let texture = SpriteMaterial::load(&texture_path, &self.renderer);
        let font = FontInfo::from_file(&data_path).unwrap();
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.fonts.insert(id, (font, texture));
        id
    }

    pub fn spawn_ground_plane(&mut self, x: i32, z: i32, n_quads: u16) -> usize {
        let mesh = make_rough_ground_plane(
            &self.renderer.device,
            Vec2::new(x as f32, z as f32),
            n_quads,
        );
        self.spawn_mesh(mesh)
    }

    pub fn resize(&mut self, new_size: (i32, i32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.renderer.config.width = new_size.0 as u32;
            self.renderer.config.height = new_size.1 as u32;
            self.renderer
                .surface
                .configure(&self.renderer.device, &self.renderer.config);
            self.depth_texture = Texture::depth_texture(&self.renderer, "depth_texture");
            self.im_tex_1 = Texture::blank_texture(&self.renderer, "im_tex_1");
            self.im_tex_2 = Texture::blank_texture(&self.renderer, "im_tex_2");
        }
    }

    pub fn update_surface(&mut self) {
        self.renderer.surface = self
            .renderer
            .instance
            .create_surface(self.window.render_context())
            .unwrap();
    }

    fn draw_lava(&self, view: &wgpu::TextureView) {
        let mut command_encoder = self
            .renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

        let transform = mat4_identity();
        self.pipelines.lava_lamp_pipeline.draw(
            &mut rp,
            &self.standard_quad,
            &transform,
            &self.common_shader_info,
            &self.renderer.queue,
            0,
        );

        drop(rp);
        self.renderer
            .queue
            .submit(std::iter::once(command_encoder.finish()));
    }

    fn draw_map(&self, view: &wgpu::TextureView) {
        let mut command_encoder = self
            .renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

        rp.set_pipeline(&self.pipelines.map_pipeline);
        rp.set_bind_group(0, &self.common_shader_info.bind_group, &[]);

        draw_mesh(&mut rp, &self.standard_quad);

        drop(rp);
        self.renderer
            .queue
            .submit(std::iter::once(command_encoder.finish()));
    }

    fn clear(&self, view: &wgpu::TextureView, color: Color) {
        let mut command_encoder = self
            .renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let rp = self.get_render_pass(&mut command_encoder, Some(color.to_wgpu()), &view);

        drop(rp);

        self.renderer
            .queue
            .submit(std::iter::once(command_encoder.finish()));
    }

    fn draw_3d(&self, meshes: &[MeshObject], view: &wgpu::TextureView) {
        let mut command_encoder = self
            .renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

        rp.set_pipeline(self.pipelines.standard_3d_pipeline.pipeline());
        rp.set_bind_group(2, &self.common_shader_info.bind_group, &[]);
        self.pipelines.standard_3d_pipeline.set_bindings(&mut rp);

        for i in 0..meshes.len() {
            let matrix: Matrix4<f32> = meshes[i].get_transform_matrix();
            self.pipelines.standard_3d_pipeline.upload_transform(
                i as u64,
                &matrix,
                &self.renderer.queue,
            );
        }

        rp.set_bind_group(1, &self.textures.values().next().unwrap().bind_group, &[]);

        for i in 0..meshes.len() {
            let bg = self
                .pipelines
                .standard_3d_pipeline
                .transforms()
                .bind_group(i);

            let Some(mesh) = self.meshes.get(&meshes[i].mesh_id) else {
                println!("Failed to get mesh of type {:?}", meshes[i].mesh_id);
                continue;
            };

            mesh.set_as_active(&mut rp);

            rp.set_bind_group(0, bg, &[]);
            rp.draw_indexed(0..mesh.index_count(), 0, 0..1);
        }

        drop(rp);

        self.renderer
            .queue
            .submit(std::iter::once(command_encoder.finish()));
    }

    fn draw_circles(&self, view: &wgpu::TextureView, commands: &[CircleCommand]) {
        let (sx, sy) = self.window.get_size();
        let screen = Vec2d::new(sx as f64, sy as f64);

        for chunk in commands.chunks(CirclePipeline::MAX_CHARS_PER_PASS) {
            let mut command_encoder = self
                .renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

            self.pipelines
                .circle_pipeline
                .assign_buffer_data(&self.renderer.queue, chunk, screen);

            self.pipelines
                .circle_pipeline
                .draw_circles(&mut rp, chunk.len());

            drop(rp);

            self.renderer
                .queue
                .submit(std::iter::once(command_encoder.finish()));
        }
    }

    fn draw_lines(&self, view: &wgpu::TextureView, commands: &[LineCommand]) {
        let (sx, sy) = self.window.get_size();
        for chunk in commands.chunks(LinePipeline::MAX_LINES_PER_PASS) {
            let mut command_encoder = self
                .renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

            self.pipelines.line_pipeline.assign_buffer_data(
                &self.renderer.queue,
                chunk,
                sx as f64,
                sy as f64,
            );

            self.pipelines
                .line_pipeline
                .draw_lines(&mut rp, chunk.len());

            drop(rp);

            self.renderer
                .queue
                .submit(std::iter::once(command_encoder.finish()));
        }
    }

    fn draw_rectangles(&self, view: &wgpu::TextureView, commands: &[RectCommand]) {
        let (sx, sy) = self.window.get_size();
        let screen = Vec2d::new(sx as f64, sy as f64);

        for cmd in commands {
            let mut command_encoder = self
                .renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

            let tf = screen_space_transform(cmd.pos, cmd.dims, screen, cmd.angle);

            self.pipelines.single_color_pipeline.draw(
                &mut rp,
                &self.standard_quad,
                &tf,
                &cmd.color,
                &self.renderer.queue,
            );

            drop(rp);

            self.renderer
                .queue
                .submit(std::iter::once(command_encoder.finish()));
        }
    }

    fn draw_ui(&self, view: &wgpu::TextureView, commands: &RenderCommands) {
        let (sx, sy) = self.window.get_size();
        let screen = Vec2d::new(sx as f64, sy as f64);

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
                .renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let mut rp = self.get_render_pass(&mut command_encoder, None, &view);

            self.pipelines.text_pipeline.assign_buffer_data(
                &self.renderer.queue,
                chunk,
                font,
                screen,
            );
            self.pipelines.text_pipeline.draw_text(
                &mut rp,
                &self.standard_quad,
                material,
                chunk.len(),
            );

            drop(rp);

            self.renderer
                .queue
                .submit(std::iter::once(command_encoder.finish()));
        }
    }

    pub fn update(&mut self, view_proj: Mat4, time: f32) {
        self.pipelines
            .standard_3d_pipeline
            .upload_camera_matrix(&view_proj, &self.renderer.queue);

        let mouse_pos = self.window.get_cursor_pos();

        let shader_params = ShaderParams {
            mouse: (mouse_pos.0 as f32, mouse_pos.1 as f32),
            time,
            resolution: (
                self.window.get_size().0 as f32,
                self.window.get_size().1 as f32,
            ),
        };

        self.renderer.queue.write_buffer(
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
            .renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rp = self.get_render_pass(&mut command_encoder, None, &outgoing);

        self.pipelines
            .blur_pipeline
            .blur_pass(&mut rp, &self.standard_quad, &incoming.bind_group);

        drop(rp);
        self.renderer
            .queue
            .submit(std::iter::once(command_encoder.finish()));
    }

    pub fn apply_geometry_commands(&self, commands: &RenderCommands, view: &wgpu::TextureView) {
        for cmd in commands.commands() {
            match cmd {
                RenderCommand::Char(_c) => (),
                RenderCommand::Rect(c) => self.draw_rectangles(view, &[*c]),
                RenderCommand::Circle(c) => self.draw_circles(view, &[*c]),
                RenderCommand::Line(c) => self.draw_lines(view, &[*c]),
            }
        }
    }

    pub fn render(
        &mut self,
        view_selector: ViewSelector,
        draw_wireframes: bool,
        meshes: &[MeshObject],
        commands: &RenderCommands,
    ) -> Result<(), wgpu::SurfaceError> {
        let (w, h) = self.window.get_size();

        if w == 0 || h == 0 {
            return Ok(());
        }

        self.renderer.device.poll(wgpu::Maintain::wait());

        {
            let event = self.renderer.queue.submit([]);
            let maintain = wgpu::Maintain::WaitForSubmissionIndex(event);
            self.renderer.device.poll(maintain);
        }

        let drawable = self.renderer.surface.get_current_texture()?;

        let view = drawable
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        match view_selector {
            ViewSelector::Map => {
                self.draw_map(&view);
            }
            ViewSelector::Lava => {
                self.draw_lava(&view);
            }
            ViewSelector::World3d => {
                if draw_wireframes {
                    self.pipelines
                        .standard_3d_pipeline
                        .set_draw_wireframes(true);
                    self.clear(&view, Color::BLACK);
                    self.draw_3d(meshes, &view);
                } else {
                    self.pipelines
                        .standard_3d_pipeline
                        .set_draw_wireframes(false);
                    self.clear(&view, Color::rgb(117, 186, 255, 1.0));
                    self.draw_3d(meshes, &view);
                    self.apply_geometry_commands(commands, &view);
                    // self.blur_pass(&self.im_tex_1, &view);
                    self.draw_ui(&view, commands);
                }
            }
        }

        self.renderer.device.poll(wgpu::Maintain::wait());

        drawable.present();

        Ok(())
    }
}
