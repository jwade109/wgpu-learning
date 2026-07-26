use std::collections::HashMap;

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
#[derive(Debug, Copy, Clone)]
struct UniformData {
    mouse: (f32, f32),
    resolution: (f32, f32),
    time: f32,
}

impl UniformData {
    fn to_bytes(&self) -> Vec<u8> {
        [
            self.mouse.0.to_le_bytes(),
            self.mouse.1.to_le_bytes(),
            self.resolution.0.to_le_bytes(),
            self.resolution.1.to_le_bytes(),
            self.time.to_le_bytes(),
            [0; 4],
            [0; 4],
            [0; 4],
            [0; 4],
            [0; 4],
        ]
        .concat()
    }
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
        for i in 0..self.quads.len() {
            self.quads[i].angle = self.quads[i].angle + self.quads[i].vel * dt;
            if self.quads[i].angle > 360.0 {
                self.quads[i].angle -= 360.0;
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
    loading_animation_pipeline: wgpu::RenderPipeline,
    map_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
    full_screen_mesh: Mesh,

    fun_quad_meshes: HashMap<usize, Mesh>,
    fun_quad_material: Material,

    ubo: Option<UBO>,

    common_shader_info: SingleUBO,

    pipeline_selector: PipelineSelector,
}

#[derive(PartialEq, Eq)]
enum PipelineSelector {
    Lava,
    Loading,
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

        let full_screen_quad_mesh = mesh_builder::make_n_gon(&device, 3);

        let mut fun_quad_meshes = HashMap::new();

        for n_sides in 3..=20 {
            let mesh = mesh_builder::make_n_gon(&device, n_sides);
            fun_quad_meshes.insert(n_sides, mesh);
        }

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
            mouse: (450.0, 360.0),
            time: 0.0,
            resolution: (100.0, 100.0),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: &uniform_data.to_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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

        let lava_lamp_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            let shader = Shader::from_path("src/shaders/cells.wgsl");
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.build("Lava Lamp Pipeline", &shader, config.format)
        };

        let loading_animation_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            let shader = Shader::from_path("src/shaders/loading.wgsl");
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.build("Lava Lamp Pipeline", &shader, config.format)
        };

        let map_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            let shader = Shader::from_path("src/shaders/map.wgsl");
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.build("Map Pipeline", &shader, config.format)
        };

        let texture_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            let shader = Shader::from_path("src/shaders/texture.wgsl");
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.add_bind_group_layout(&material_bind_group_layout);
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.build("Texture Pipeline", &shader, config.format)
        };

        let fun_quad_material = Material::new(
            "img/invincible.jpg",
            &device,
            &queue,
            "Quad Material",
            &material_bind_group_layout,
        );

        let uniform_bind_group = {
            let mut builder = Builder::new(&device);
            builder.set_layout(&time_etc_data_bind_group);
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
            loading_animation_pipeline,
            map_pipeline,
            texture_pipeline,
            full_screen_mesh: full_screen_quad_mesh,
            fun_quad_meshes,
            fun_quad_material,
            ubo: None,

            common_shader_info: SingleUBO {
                buffer: uniform_buffer,
                bind_group: uniform_bind_group,
            },

            pipeline_selector: PipelineSelector::Lava,
        }
    }

    fn get_current_pipeline(&self) -> &wgpu::RenderPipeline {
        match self.pipeline_selector {
            PipelineSelector::Lava => &self.lava_lamp_pipeline,
            PipelineSelector::Loading => &self.loading_animation_pipeline,
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

    fn draw_lava_lamp(&self, rp: &mut wgpu::RenderPass) {
        rp.set_bind_group(0, &self.common_shader_info.bind_group, &[]);
        self.full_screen_mesh.set_as_active(rp);
        let bg = self
            .ubo
            .as_ref()
            .map(|e| e.bind_group(0))
            .flatten()
            .unwrap();
        rp.set_bind_group(1, bg, &[]);
        rp.draw_indexed(0..self.full_screen_mesh.index_count(), 0, 0..1);
    }

    fn draw_loading(&self, rp: &mut wgpu::RenderPass) {
        rp.set_bind_group(0, &self.common_shader_info.bind_group, &[]);
        self.full_screen_mesh.set_as_active(rp);
        let bg = self
            .ubo
            .as_ref()
            .map(|e| e.bind_group(0))
            .flatten()
            .unwrap();
        rp.set_bind_group(1, bg, &[]);
        rp.draw_indexed(0..self.full_screen_mesh.index_count(), 0, 0..1);
    }

    fn draw_map(&self, rp: &mut wgpu::RenderPass) {
        rp.set_bind_group(0, &self.common_shader_info.bind_group, &[]);
        self.full_screen_mesh.set_as_active(rp);
        let bg = self
            .ubo
            .as_ref()
            .map(|e| e.bind_group(0))
            .flatten()
            .unwrap();
        rp.set_bind_group(1, bg, &[]);
        rp.draw_indexed(0..self.full_screen_mesh.index_count(), 0, 0..1);
    }

    fn draw_texture(&mut self, rp: &mut wgpu::RenderPass, quads: &Vec<Object>, tris: &Vec<Object>) {
        rp.set_bind_group(2, &self.common_shader_info.bind_group, &[]);

        // upload transforms to UBO
        let c0 = glm::Vec4::new(1.0, 0.0, 0.0, 0.0);
        let c1 = glm::Vec4::new(0.0, 1.0, 0.0, 0.0);
        let c2 = glm::Vec4::new(0.0, 0.0, 1.0, 0.0);
        let c3 = glm::Vec4::new(0.0, 0.0, 0.0, 1.0);
        let m1 = glm::Matrix4::new(c0, c1, c2, c3);
        let m2 = glm::Matrix4::new(c0, c1, c2, c3);

        {
            let mut offset: u64 = 0;
            for i in 0..quads.len() {
                let matrix = ext::rotate(&m2, quads[i].angle, glm::Vector3::new(0.0, 0.0, 1.0))
                    * ext::translate(&m1, quads[i].position);
                self.ubo
                    .as_mut()
                    .unwrap()
                    .upload(offset + i as u64, &matrix, &self.queue);
            }

            offset = quads.len() as u64;
            for i in 0..tris.len() {
                let matrix = ext::rotate(&m2, tris[i].angle, glm::Vector3::new(0.0, 0.0, 1.0))
                    * ext::translate(&m1, tris[i].position);
                self.ubo
                    .as_mut()
                    .unwrap()
                    .upload(offset + i as u64, &matrix, &self.queue);
            }
        }

        // rp.set_bind_group(0, &self.common_shader_info.bind_group, &[]);
        rp.set_bind_group(1, &self.fun_quad_material.bind_group, &[]);
        // Quads

        for i in 0..quads.len() {
            let matrix = ext::rotate(&m2, quads[i].angle, glm::Vector3::new(0.0, 0.0, 1.0))
                * ext::translate(&m1, quads[i].position);
            self.ubo
                .as_mut()
                .unwrap()
                .upload(i as u64, &matrix, &self.queue);

            let bg = self
                .ubo
                .as_ref()
                .map(|e| e.bind_group(i))
                .flatten()
                .unwrap();

            let n_sides = quads[i].n_sides;
            let mesh = self.fun_quad_meshes.get(&n_sides).unwrap();

            mesh.set_as_active(rp);

            rp.set_bind_group(0, bg, &[]);
            rp.draw_indexed(0..mesh.index_count(), 0, 0..1);
        }
    }

    fn render(
        &mut self,
        quads: &mut Vec<Object>,
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
            mouse: (self.mouse_pos_smoothed[0], self.mouse_pos_smoothed[1]),
            time: self.time,
            resolution: (
                self.window.get_size().0 as f32,
                self.window.get_size().1 as f32,
            ),
        };

        if !self.paused {
            self.time += 0.005;

            for quad in quads.iter_mut() {
                let t = (self.time * 2.0).floor() as usize;
                let n = t % 18 + 3;
                quad.n_sides = n;
            }
        }

        self.queue
            .write_buffer(&self.common_shader_info.buffer, 0, &uniform_data.to_bytes());

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
                    self.draw_lava_lamp(&mut renderpass);
                }
                PipelineSelector::Loading => {
                    self.draw_loading(&mut renderpass);
                }
                PipelineSelector::Map => {
                    self.draw_map(&mut renderpass);
                }
                PipelineSelector::Texture => {
                    self.draw_texture(&mut renderpass, quads, tris);
                }
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
    world.quads.push(Object {
        position: glm::Vec3::new(0.0, 0.0, 0.0),
        angle: 0.0,
        vel: 0.00002,
        n_sides: 9,
    });

    state.build_ubos_for_objects(world.quads.len());

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
                        PipelineSelector::Lava => PipelineSelector::Loading,
                        PipelineSelector::Loading => PipelineSelector::Map,
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

        match state.render(&mut world.quads, &world.tris) {
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
