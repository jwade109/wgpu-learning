use glfw::{fail_on_errors, Action, ClientApiHint, Key, Window, WindowHint};
mod renderer_backend;
use renderer_backend::{bind_group_layout, material::Material, mesh_builder, pipeline, ubo::UBO};
mod model;
use clap::Parser;
use glm::ext;
use model::game_objects::Object;
use wgpu::util::DeviceExt;

use crate::{
    model::game_objects::Camera,
    renderer_backend::{bind_group::Builder, mesh_builder::Mesh, pipeline::Shader, ubo::SingleUBO},
};

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformData {
    mouse_pos_and_time: [f32; 3],
    _dummy: [f32; 12],
}

struct World {
    quads: Vec<Object>,
    tris: Vec<Object>,
    camera: Camera,
}

impl World {
    fn new() -> Self {
        World {
            quads: Vec::new(),
            tris: Vec::new(),
            camera: Camera::new(),
        }
    }

    fn update(&mut self, dt: f32, window: &mut glfw::Window) {
        for i in 0..self.tris.len() {
            self.tris[i].angle = self.tris[i].angle + self.tris[i].vel * dt;
            if self.tris[i].angle > 360.0 {
                self.tris[i].angle -= 360.0;
            }
        }

        // let pos = window.get_cursor_pos();
        // window.set_cursor_pos(400.0, 400.0);
        // let dx = (-40.0 * (pos.0 - 400.0) / 400.0) as f32;
        // let dy = (-40.0 * (pos.1 - 400.0) / 400.0) as f32;

        // self.camera.spin(dx, dy);
    }
}

struct State<'a> {
    time: f32,
    paused: bool,
    mouse_pos_smoothed: [f32; 2],

    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (i32, i32),
    window: &'a mut Window,
    lava_lamp_pipeline: wgpu::RenderPipeline,
    map_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
    triangle: MeshWithMaterial,
    quad: MeshWithMaterial,
    ubo: Option<UBO>,

    uniform_data_ubo: SingleUBO,

    pipeline_selector: PipelineSelector,
}

struct MeshWithMaterial {
    mesh: Mesh,
    material: Material,
}

#[derive(PartialEq, Eq)]
enum PipelineSelector {
    Lava,
    Map,
    Texture,
}

impl<'a> State<'a> {
    async fn new(window: &'a mut Window, shader_path: &str) -> Self {
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
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
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

        let triangle_mesh = mesh_builder::make_octagon(&device);

        let quad_mesh = mesh_builder::make_quad(&device);

        let material_bind_group_layout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("Material Bind Group Layout");
        }

        let ubo_bind_group_layout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_ubo();
            ubo_bind_group_layout = builder.build("UBO Bind Group Layout");
        }

        let uniform_data = UniformData {
            mouse_pos_and_time: [450.0, 360.0, 0.0],
            _dummy: Default::default(),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
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

        let lava_lamp_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            let shader = Shader::from_path("src/shaders/cells.wgsl");
            builder.add_vertex_buffer_layout(mesh_builder::Vertex::get_layout());
            builder.add_bind_group_layout(&uniform_bind_group_layout);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.build("Lava Lamp Pipeline", &shader, config.format)
        };

        let map_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            let shader = Shader::from_path("src/shaders/map.wgsl");
            builder.add_vertex_buffer_layout(mesh_builder::Vertex::get_layout());
            builder.add_bind_group_layout(&uniform_bind_group_layout);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.build("Map Pipeline", &shader, config.format)
        };

        let texture_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            let shader = Shader::from_path("src/shaders/texture.wgsl");
            builder.add_vertex_buffer_layout(mesh_builder::Vertex::get_layout());
            builder.add_bind_group_layout(&uniform_bind_group_layout);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.add_bind_group_layout(&material_bind_group_layout);
            builder.build("Texture Pipeline", &shader, config.format)
        };

        let triangle_material = Material::new(
            "img/pod.jpg",
            &device,
            &queue,
            "Triangle Material",
            &material_bind_group_layout,
        );

        let quad_material = Material::new(
            "img/invincible.jpg",
            &device,
            &queue,
            "Quad Material",
            &material_bind_group_layout,
        );

        let uniform_bind_group = {
            let mut builder = Builder::new(&device);
            builder.set_layout(&uniform_bind_group_layout);
            builder.add_buffer(&uniform_buffer, 0);
            builder.build("uniform buffer")
        };

        Self {
            time: 0.0,
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
            map_pipeline,
            texture_pipeline,
            triangle: MeshWithMaterial {
                mesh: triangle_mesh,
                material: triangle_material,
            },
            quad: MeshWithMaterial {
                mesh: quad_mesh,
                material: quad_material,
            },
            ubo: None,

            uniform_data_ubo: SingleUBO {
                buffer: uniform_buffer,
                bind_group: uniform_bind_group,
            },

            pipeline_selector: PipelineSelector::Lava,
        }
    }

    fn get_current_pipeline(&self) -> &wgpu::RenderPipeline {
        match self.pipeline_selector {
            PipelineSelector::Lava => &self.lava_lamp_pipeline,
            PipelineSelector::Map => &self.map_pipeline,
            PipelineSelector::Texture => &self.texture_pipeline,
        }
    }

    fn resize(&mut self, new_size: (i32, i32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0 as u32;
            self.config.height = new_size.1 as u32;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update_surface(&mut self) {
        self.surface = self
            .instance
            .create_surface(self.window.render_context())
            .unwrap();
    }

    pub fn build_ubos_for_objects(&mut self, object_count: usize) {
        let ubo_bind_group_layout = {
            let mut builder = bind_group_layout::Builder::new(&self.device);
            builder.add_ubo();
            builder.build("UBO Bind Group Layout")
        };
        self.ubo = Some(UBO::new(&self.device, object_count, ubo_bind_group_layout));
    }

    fn render(
        &mut self,
        quads: &Vec<Object>,
        tris: &Vec<Object>,
    ) -> Result<(), wgpu::SurfaceError> {
        let (w, h) = self.window.get_size();

        if w == 0 || h == 0 {
            return Ok(());
        }

        self.device.poll(wgpu::Maintain::wait());

        let mouse_pos = self.window.get_cursor_pos();

        self.mouse_pos_smoothed[0] += (mouse_pos.0 as f32 - self.mouse_pos_smoothed[0]) * 0.06;
        self.mouse_pos_smoothed[1] += (mouse_pos.1 as f32 - self.mouse_pos_smoothed[1]) * 0.06;

        let uniform_data = UniformData {
            mouse_pos_and_time: [
                self.mouse_pos_smoothed[0],
                self.mouse_pos_smoothed[1],
                self.time,
            ],
            _dummy: Default::default(),
        };

        if !self.paused {
            self.time += 0.005;
        }

        self.queue.write_buffer(
            &self.uniform_data_ubo.buffer,
            0,
            bytemuck::cast_slice(&[uniform_data]),
        );

        // upload transforms to UBO
        {
            let mut offset: u64 = 0;
            for i in 0..quads.len() {
                let c0 = glm::Vec4::new(1.0, 0.0, 0.0, 0.0);
                let c1 = glm::Vec4::new(0.0, 1.0, 0.0, 0.0);
                let c2 = glm::Vec4::new(0.0, 0.0, 1.0, 0.0);
                let c3 = glm::Vec4::new(0.0, 0.0, 0.0, 1.0);
                let m1 = glm::Matrix4::new(c0, c1, c2, c3);
                let m2 = glm::Matrix4::new(c0, c1, c2, c3);
                let matrix = ext::rotate(&m2, quads[i].angle, glm::Vector3::new(0.0, 0.0, 1.0))
                    * ext::translate(&m1, quads[i].position);
                self.ubo
                    .as_mut()
                    .unwrap()
                    .upload(offset + i as u64, &matrix, &self.queue);
            }

            offset = quads.len() as u64;
            for i in 0..tris.len() {
                let c0 = glm::Vec4::new(1.0, 0.0, 0.0, 0.0);
                let c1 = glm::Vec4::new(0.0, 1.0, 0.0, 0.0);
                let c2 = glm::Vec4::new(0.0, 0.0, 1.0, 0.0);
                let c3 = glm::Vec4::new(0.0, 0.0, 0.0, 1.0);
                let m1 = glm::Matrix4::new(c0, c1, c2, c3);
                let m2 = glm::Matrix4::new(c0, c1, c2, c3);
                let matrix = ext::rotate(&m2, tris[i].angle, glm::Vector3::new(0.0, 0.0, 1.0))
                    * ext::translate(&m1, tris[i].position);
                self.ubo
                    .as_mut()
                    .unwrap()
                    .upload(offset + i as u64, &matrix, &self.queue);
            }
        }

        {
            let event = self.queue.submit([]);
            let maintain = wgpu::Maintain::WaitForSubmissionIndex(event);
            self.device.poll(maintain);
        }

        let drawable = self.surface.get_current_texture()?;
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &drawable
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.75,
                        g: 0.5,
                        b: 0.25,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
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
            renderpass.set_pipeline(self.get_current_pipeline());

            match self.pipeline_selector {
                PipelineSelector::Lava => {
                    renderpass.set_bind_group(0, &self.uniform_data_ubo.bind_group, &[]);
                }
                PipelineSelector::Map => {
                    renderpass.set_bind_group(0, &self.uniform_data_ubo.bind_group, &[]);
                }
                PipelineSelector::Texture => {
                    renderpass.set_bind_group(0, &self.uniform_data_ubo.bind_group, &[]);
                    renderpass.set_bind_group(2, &self.quad.material.bind_group, &[]);
                }
            }

            // Quads
            self.quad.mesh.apply_to_pass(&mut renderpass);

            let n_indices = self.quad.mesh.index_count();

            let mut offset: usize = 0;
            for i in 0..quads.len() {
                let bg = self
                    .ubo
                    .as_ref()
                    .map(|e| e.bind_group(offset + i))
                    .flatten()
                    .unwrap();
                renderpass.set_bind_group(1, bg, &[]);
                renderpass.draw_indexed(0..n_indices, 0, 0..1);
            }

            if self.pipeline_selector == PipelineSelector::Texture {
                renderpass.set_bind_group(2, &self.triangle.material.bind_group, &[]);
            }

            self.triangle.mesh.apply_to_pass(&mut renderpass);
            let n_indices = self.triangle.mesh.index_count();

            offset = quads.len();
            for i in 0..tris.len() {
                let bg = self
                    .ubo
                    .as_ref()
                    .map(|e| e.bind_group(offset + i))
                    .flatten()
                    .unwrap();
                renderpass.set_bind_group(1, bg, &[]);
                // renderpass.draw(0..3, 0..1);
                renderpass.draw_indexed(0..n_indices, 0, 0..1);
            }
        }
        self.queue.submit(std::iter::once(command_encoder.finish()));
        self.device.poll(wgpu::Maintain::wait());

        drawable.present();

        Ok(())
    }
}

#[derive(Parser)]
struct Args {
    shader_path: String,
}

async fn run() {
    let args = Args::parse();

    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    let (mut window, events) = glfw
        .create_window(800, 600, "It's WGPU time.", glfw::WindowMode::Windowed)
        .unwrap();

    let mut state = State::new(&mut window, &args.shader_path).await;

    state.window.set_framebuffer_size_polling(true);
    state.window.set_key_polling(true);
    state.window.set_mouse_button_polling(true);
    state.window.set_pos_polling(true);

    // state.window.set_cursor_mode(glfw::CursorMode::Hidden);

    // Build world
    let mut world = World::new();
    world.tris.push(Object {
        position: glm::Vec3::new(0.0, 0.0, 0.0),
        angle: 0.0,
        vel: 0.000,
    });
    // world.tris.push(Object {
    //     position: glm::Vec3::new(0.0, 0.0, 0.0),
    //     angle: 0.8,
    //     vel: 0.002,
    // });
    world.quads.push(Object {
        position: glm::Vec3::new(0.0, 0.0, 0.0),
        angle: 0.0,
        vel: 0.0,
    });

    state.build_ubos_for_objects(3);

    while !state.window.should_close() {
        glfw.poll_events();

        world.update(16.67, state.window);

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                //Hit escape
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    state.window.set_should_close(true)
                }
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    state.paused ^= true;
                }
                glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) => {
                    state.pipeline_selector = match state.pipeline_selector {
                        PipelineSelector::Lava => PipelineSelector::Map,
                        PipelineSelector::Map => PipelineSelector::Texture,
                        PipelineSelector::Texture => PipelineSelector::Lava,
                    };
                }

                //Window was moved
                glfw::WindowEvent::Pos(..) => {
                    state.update_surface();
                    state.resize(state.size);
                }

                //Window was resized
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    state.update_surface();
                    state.resize((width, height));
                }
                _ => {}
            }
        }

        match state.render(&world.quads, &world.tris) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                state.update_surface();
                state.resize(state.size);
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

fn main() {
    pollster::block_on(run());
}
