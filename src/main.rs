use glm::*;
use std::collections::HashMap;

use enum_iterator::Sequence;
use glfw::{fail_on_errors, Action, ClientApiHint, Key, Window, WindowHint};
mod renderer_backend;
use renderer_backend::{bind_group_layout, material::SpriteMaterial, mesh, pipeline, ubo::UBO};
mod model;
use clap::Parser;
use model::game_objects::Object;
use wgpu::util::DeviceExt;

use crate::{
    model::game_objects::{Camera, MeshType},
    renderer_backend::{bind_group::Builder, mesh::Mesh, pipeline::Shader, ubo::SingleUBO},
};

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone)]
struct ShaderParams {
    mouse: (f32, f32),
    resolution: (f32, f32),
    camera_offset: (f32, f32, f32),
    time: f32,
}

impl ShaderParams {
    fn to_bytes(&self) -> Vec<u8> {
        [
            self.mouse.0.to_le_bytes(),
            self.mouse.1.to_le_bytes(),
            self.resolution.0.to_le_bytes(),
            self.resolution.1.to_le_bytes(),
            self.camera_offset.0.to_le_bytes(),
            self.camera_offset.1.to_le_bytes(),
            self.camera_offset.2.to_le_bytes(),
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
    camera: Camera,
}

impl World {
    fn new() -> Self {
        World {
            quads: Vec::new(),
            camera: Camera::new(),
        }
    }

    fn update(&mut self, dt: f32, window: &mut glfw::Window) {
        for i in 0..self.quads.len() {
            self.quads[i].angle = self.quads[i].angle + self.quads[i].vel * dt;
            if self.quads[i].angle > 360.0 {
                self.quads[i].angle -= 360.0;
            }
        }

        // let (sx, sy) = window.get_size();
        // let mouse_pos = window.get_cursor_pos();
        // window.set_cursor_pos(sx as f64, sy as f64);
        // let dx = (-40.0 * mouse_pos.0 as f32 - sx as f32) / sx as f32;
        // let dy = (-40.0 * mouse_pos.1 as f32 - sy as f32) / sy as f32;
        // self.camera.spin(dx / 100.0, dy / 100.0);
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
    wireframe_texture_pipeline: wgpu::RenderPipeline,
    full_screen_mesh: Mesh,
    fun_quad_meshes: HashMap<usize, Mesh>,
    cube_mesh: Mesh,
    fun_quad_material: SpriteMaterial,

    ubo: Option<UBO>,

    projection_ubo: UBO,

    common_shader_info: SingleUBO,

    pipeline_selector: PipelineSelector,
    draw_wireframes: bool,
}

#[derive(PartialEq, Eq, Sequence)]
enum PipelineSelector {
    Lava,
    Loading,
    Map,
    Texture,
}

impl<'a> State<'a> {
    async fn new(window: &'a mut Window) -> Self {
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

        let full_screen_quad_mesh = mesh::make_quad(&device, 1.0);

        let mut fun_quad_meshes = HashMap::new();

        let cube_mesh = mesh::make_cube(&device, Vec3::new(1.0, 0.6, 0.6));

        for n_sides in 3..=70 {
            let mesh = mesh::make_n_gon(&device, n_sides);
            fun_quad_meshes.insert(n_sides, mesh);
        }

        let material_bind_group_layout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("SpriteMaterial Bind Group Layout");
        }

        let ubo_bind_group_layout;
        {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_ubo();
            ubo_bind_group_layout = builder.build("UBO Bind Group Layout");
        }

        let shader_params = ShaderParams {
            mouse: (450.0, 360.0),
            time: 0.0,
            resolution: (100.0, 100.0),
            camera_offset: (0.0, 0.0, 0.0),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader Params"),
            contents: &shader_params.to_bytes(),
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

        let camera_projection_bind_group_layout = {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_ubo();
            builder.build("Camera Projection UBO")
        };

        let texture_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            let shader = Shader::from_path("src/shaders/texture.wgsl");
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.add_bind_group_layout(&material_bind_group_layout);
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.add_bind_group_layout(&camera_projection_bind_group_layout);
            builder.build("Texture Pipeline", &shader, config.format)
        };

        let wireframe_texture_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            let shader = Shader::from_path("src/shaders/texture.wgsl");
            builder.add_bind_group_layout(&ubo_bind_group_layout);
            builder.add_bind_group_layout(&material_bind_group_layout);
            builder.add_bind_group_layout(&time_etc_data_bind_group);
            builder.add_bind_group_layout(&camera_projection_bind_group_layout);
            builder.wireframes();
            builder.build("Texture Pipeline", &shader, config.format)
        };

        let fun_quad_material = SpriteMaterial::new(
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

        let projection_ubo = UBO::new(&device, 1, camera_projection_bind_group_layout);

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
            wireframe_texture_pipeline,
            full_screen_mesh: full_screen_quad_mesh,
            cube_mesh,
            fun_quad_meshes,
            fun_quad_material,
            ubo: None,
            projection_ubo,

            common_shader_info: SingleUBO {
                buffer: uniform_buffer,
                bind_group: uniform_bind_group,
            },

            pipeline_selector: PipelineSelector::Texture,
            draw_wireframes: false,
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
        rp.set_pipeline(&self.lava_lamp_pipeline);
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
        rp.set_pipeline(&self.loading_animation_pipeline);
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
        rp.set_pipeline(&self.map_pipeline);
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

    fn draw_texture(&mut self, rp: &mut wgpu::RenderPass, quads: &Vec<Object>) {
        if self.draw_wireframes {
            rp.set_pipeline(&self.wireframe_texture_pipeline)
        } else {
            rp.set_pipeline(&self.texture_pipeline);
        }
        rp.set_bind_group(2, &self.common_shader_info.bind_group, &[]);
        rp.set_bind_group(3, self.projection_ubo.bind_group(0), &[]);

        {
            for i in 0..quads.len() {
                let matrix = quads[i].get_transform_matrix();
                self.ubo
                    .as_mut()
                    .unwrap()
                    .upload(i as u64, &matrix, &self.queue);
            }
        }

        rp.set_bind_group(1, self.fun_quad_material.bind_group(), &[]);

        for i in 0..quads.len() {
            let bg = self
                .ubo
                .as_ref()
                .map(|e| e.bind_group(i))
                .flatten()
                .unwrap();

            let mesh = match quads[i].mesh_type {
                MeshType::Polygon(n_sides) => self.fun_quad_meshes.get(&n_sides).unwrap(),
                MeshType::Cube => &self.cube_mesh,
            };

            mesh.set_as_active(rp);

            rp.set_bind_group(0, bg, &[]);
            rp.draw_indexed(0..mesh.index_count(), 0, 0..1);
        }
    }

    fn update_projection(&mut self, camera: &Camera) {
        let z = (self.time / 9.0).sin() * 20.0;
        let x = (self.time / 9.0).cos() * 20.0;

        let tz = (self.time / 4.0).sin() * 2.0;
        let tx = (self.time / 4.0).cos() * 2.0;

        let target = Vec3::new(tx, 0.0, tz);
        let eye = Vec3::new(x, 19.0, z);

        let up = Vec3::new(0.0, 1.0, 0.0);

        let zaxis = normalize(eye - target); // forward vector
        let xaxis = normalize(cross(up, zaxis)); // The "right" vector.
        let yaxis = normalize(cross(zaxis, xaxis)); // The "up" vector.

        let orientation = Matrix4::new(
            Vec4::new(xaxis.x, yaxis.x, zaxis.x, 0.0),
            Vec4::new(xaxis.y, yaxis.y, zaxis.y, 0.0),
            Vec4::new(xaxis.z, yaxis.z, zaxis.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        let translation = Matrix4::new(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(-eye.x, -eye.y, -eye.z, 1.0),
        );

        let view = orientation * translation;

        let fov_y: f32 = radians(40.0);
        let (sx, sy) = self.window.get_size();
        let aspect = sx as f32 / sy as f32;
        let z_near = 0.1;
        let z_far = 100.0;
        let projection = ext::perspective(fov_y, aspect, z_near, z_far);

        let view_proj = projection * view;
        self.projection_ubo.upload(0, &view_proj, &self.queue);
    }

    fn render(&mut self, world: &mut World) -> Result<(), wgpu::SurfaceError> {
        let (w, h) = self.window.get_size();

        if w == 0 || h == 0 {
            return Ok(());
        }

        self.device.poll(wgpu::Maintain::wait());

        world.camera.spin(0.04, 0.0);

        self.update_projection(&world.camera);

        let mouse_pos = self.window.get_cursor_pos();

        self.mouse_pos_smoothed[0] += (mouse_pos.0 as f32 - self.mouse_pos_smoothed[0]) * 0.06;
        self.mouse_pos_smoothed[1] += (mouse_pos.1 as f32 - self.mouse_pos_smoothed[1]) * 0.06;

        let shader_params = ShaderParams {
            mouse: (self.mouse_pos_smoothed[0], self.mouse_pos_smoothed[1]),
            time: self.time,
            resolution: (
                self.window.get_size().0 as f32,
                self.window.get_size().1 as f32,
            ),
            camera_offset: (
                world.camera.position.x,
                world.camera.position.y,
                world.camera.position.z,
            ),
        };

        if !self.paused {
            self.time += 0.005;
        }

        self.queue.write_buffer(
            &self.common_shader_info.buffer,
            0,
            &shader_params.to_bytes(),
        );

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
                    self.draw_texture(&mut renderpass, &mut world.quads);
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
    // shader_path: String,
}

async fn run() {
    let args = Args::parse();

    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    let (mut window, events) = glfw
        .create_window(800, 600, "It's WGPU time.", glfw::WindowMode::Windowed)
        .unwrap();

    let mut state = State::new(&mut window).await;

    state.window.set_framebuffer_size_polling(true);
    state.window.set_key_polling(true);
    state.window.set_mouse_button_polling(true);
    state.window.set_pos_polling(true);

    // state.window.set_cursor_mode(glfw::CursorMode::Hidden);

    // Build world
    let mut world = World::new();
    world.quads.push(Object {
        position: Vec3::new(0.0, 0.0, -9.0),
        angle: 0.0,
        vel: 0.0,
        mesh_type: MeshType::Polygon(9),
    });
    world.quads.push(Object {
        position: Vec3::new(0.0, 0.0, -5.6),
        angle: 0.0,
        vel: 0.0,
        mesh_type: MeshType::Polygon(3),
    });
    world.quads.push(Object {
        position: Vec3::new(0.2, 0.3, -4.8),
        angle: 0.4,
        vel: 0.0,
        mesh_type: MeshType::Polygon(6),
    });

    for x in (0..20).step_by(2) {
        for z in (0..20).step_by(3) {
            world.quads.push(Object {
                position: Vec3::new(x as f32, 0.0, z as f32),
                angle: 0.0,
                vel: (x * z) as f32 / 100000.0,
                mesh_type: MeshType::Cube,
            });
        }
    }

    for y in (3..20).step_by(2) {
        world.quads.push(Object {
            position: Vec3::new(0.0, y as f32, 0.0),
            angle: 0.0,
            vel: 0.0,
            mesh_type: MeshType::Cube,
        });
    }

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
                glfw::WindowEvent::Key(Key::Z, _, Action::Press, _) => {
                    state.draw_wireframes ^= true;
                }
                glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) => {
                    state.pipeline_selector = enum_iterator::next_cycle(&state.pipeline_selector);
                }
                glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) => {
                    state.pipeline_selector =
                        enum_iterator::previous_cycle(&state.pipeline_selector);
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

        match state.render(&mut world) {
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
