use crate::{model::*, renderer_backend::*};
use enum_iterator::Sequence;
use glm::Vec3;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

#[derive(PartialEq, Eq, Sequence)]
pub enum PipelineSelector {
    Lava,
    Loading,
    Map,
    World3d,
}

pub struct Renderer<'a> {
    pub paused: bool,
    pub pipeline_selector: PipelineSelector,
    pub draw_wireframes: bool,
    mouse_pos_smoothed: [f32; 2],

    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: (i32, i32),
    pub window: &'a mut glfw::Window,
    lava_lamp_pipeline: LavaLampPipeline,
    loading_animation_pipeline: wgpu::RenderPipeline,
    map_pipeline: wgpu::RenderPipeline,
    standard_3d_pipeline: Standard3DPipeline,

    meshes: HashMap<MeshType, Mesh>,

    fun_quad_material: SpriteMaterial,
    depth_texture: Texture,

    ubo: UBO,

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
                | wgpu::Features::POLYGON_MODE_POINT,
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

        let mut meshes = HashMap::new();

        meshes.insert(MeshType::Quad, make_quad(&device, 1.0));
        meshes.insert(
            MeshType::Cube,
            make_cube(&device, glm::Vec4::new(1.0, 0.6, 0.6, 0.4)),
        );
        meshes.insert(MeshType::GroundPlane, make_rough_ground_plane(&device));

        for n_sides in 3..=70 {
            let mesh = make_n_gon(&device, n_sides);
            meshes.insert(MeshType::Polygon(n_sides), mesh);
        }

        let material_bind_group_layout;
        {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("SpriteMaterial Bind Group Layout");
        }

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

        let loading_animation_pipeline = {
            let mut builder = PipelineBuilder::new(&device);
            let shader = Shader::from_path("src/shaders/loading.wgsl");
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.build_pipeline("Lava Lamp Pipeline", &shader, config.format, true, true)
        };

        let map_pipeline = {
            let mut builder = PipelineBuilder::new(&device);
            let shader = Shader::from_path("src/shaders/map.wgsl");
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.build_pipeline("Map Pipeline", &shader, config.format, true, true)
        };

        let standard_3d_pipeline = Standard3DPipeline::new(
            &device,
            &ubo_bind_group_layout,
            &material_bind_group_layout,
            &time_etc_data_bind_group,
            &config,
        );

        let fun_quad_material = SpriteMaterial::new(
            "img/invincible.jpg",
            &device,
            &queue,
            "Quad Material",
            &material_bind_group_layout,
        );

        let uniform_bind_group = {
            let mut builder = BindGroupBuilder::new(&device);
            builder.set_layout(&time_etc_data_bind_group);
            builder.add_buffer(&uniform_buffer, 0);
            builder.build("uniform buffer")
        };

        let ubo_bind_group_layout = {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_ubo();
            builder.build("UBO Bind Group Layout")
        };

        let ubo = UBO::new(&device, 250, ubo_bind_group_layout);

        Self {
            paused: false,
            mouse_pos_smoothed: [0.0, 0.0],
            instance,
            window,
            surface,
            device,
            queue,
            config,
            size,
            lava_lamp_pipeline,
            loading_animation_pipeline,
            map_pipeline,
            standard_3d_pipeline,
            meshes,
            fun_quad_material,
            ubo,
            common_shader_info: SingleUBO {
                buffer: uniform_buffer,
                bind_group: uniform_bind_group,
            },

            pipeline_selector: PipelineSelector::World3d,
            draw_wireframes: false,
            depth_texture,
        }
    }

    pub fn resize(&mut self, new_size: (i32, i32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0 as u32;
            self.config.height = new_size.1 as u32;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn update_surface(&mut self) {
        self.surface = self
            .instance
            .create_surface(self.window.render_context())
            .unwrap();
    }

    fn draw_loading(&self, rp: &mut wgpu::RenderPass) {
        rp.set_pipeline(&self.loading_animation_pipeline);
        let bg = self.ubo.bind_group(0);
        rp.set_bind_group(0, &self.common_shader_info.bind_group, &[]);
        rp.set_bind_group(1, bg, &[]);
        let mesh = self.meshes.get(&MeshType::Quad).unwrap();
        draw_mesh(rp, mesh);
    }

    fn draw_map(&self, rp: &mut wgpu::RenderPass) {
        rp.set_pipeline(&self.map_pipeline);
        rp.set_bind_group(0, &self.common_shader_info.bind_group, &[]);
        let mesh = self.meshes.get(&MeshType::Quad).unwrap();
        draw_mesh(rp, mesh);
    }

    fn draw_texture(&mut self, rp: &mut wgpu::RenderPass, world: &World) {
        self.standard_3d_pipeline
            .set_draw_wireframes(self.draw_wireframes);
        rp.set_pipeline(self.standard_3d_pipeline.pipeline());
        rp.set_bind_group(2, &self.common_shader_info.bind_group, &[]);
        self.standard_3d_pipeline.set_bindings(rp);

        {
            for i in 0..world.quads.len() {
                let matrix = world.quads[i].get_transform_matrix();
                self.ubo.upload(i as u64, &matrix, &self.queue);
            }
        }

        rp.set_bind_group(1, self.fun_quad_material.bind_group(), &[]);

        for i in 0..world.quads.len() {
            let bg = self.ubo.bind_group(i);
            let mesh = self.meshes.get(&world.quads[i].mesh_type).unwrap();

            mesh.set_as_active(rp);

            rp.set_bind_group(0, bg, &[]);
            rp.draw_indexed(0..mesh.index_count(), 0, 0..1);
        }
    }

    fn update_projection(&mut self, world: &World) {
        let view_proj = world.camera.to_projection_matrix(self.window);
        self.standard_3d_pipeline
            .upload_camera_matrix(&view_proj, &self.queue);
    }

    pub fn update(&mut self, world: &mut World) {
        self.update_projection(world);

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

    pub fn render(&mut self, world: &mut World) -> Result<(), wgpu::SurfaceError> {
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

        let depth_stencil_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
            view: self.depth_texture.view(),
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        });

        let clear_color = if self.draw_wireframes {
            wgpu::Color::default()
        } else {
            wgpu::Color {
                r: 0.4,
                g: 0.4,
                b: 0.9,
                a: 1.0,
            }
        };

        let drawable = self.surface.get_current_texture()?;
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &drawable
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut renderpass = command_encoder.begin_render_pass(&render_pass_descriptor);

            match self.pipeline_selector {
                PipelineSelector::Lava => {}
                PipelineSelector::Loading => {
                    self.draw_loading(&mut renderpass);
                }
                PipelineSelector::Map => {
                    self.draw_map(&mut renderpass);
                }
                PipelineSelector::World3d => {
                    self.draw_texture(&mut renderpass, &world);

                    let mesh = self.meshes.get(&MeshType::Quad).unwrap();

                    let (sx, sy) = self.window.get_size();
                    let aspect = sx as f32 / sy as f32;

                    let pixels = 300;
                    let padding = 70;
                    let size = (pixels - padding) as f32 / sx as f32;

                    let mut i = 0;

                    let camera_proj = world.camera.to_projection_matrix(self.window);
                    let t = (world.time / 3.0).sin() * 0.5 + 0.5;
                    let eye = mat4_identity();
                    let proj = mat4_lerp(&camera_proj, &eye, t);

                    'outer: for x in (-1000..=1000).step_by(pixels) {
                        for y in (-500..=500).step_by(pixels) {
                            let xoff = x as f32 / sx as f32;
                            let yoff = y as f32 / sy as f32;
                            let tf = proj
                                * translation_matrix(Vec3::new(xoff, yoff, 0.0))
                                * mat4_diagonal(size / aspect, size, 1.0, 1.0);
                            self.lava_lamp_pipeline.draw(
                                &mut renderpass,
                                mesh,
                                &tf,
                                &self.common_shader_info,
                                &self.queue,
                                i,
                            );

                            i += 1;
                            if i >= 250 {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        self.device.poll(wgpu::Maintain::wait());

        drawable.present();

        Ok(())
    }
}
