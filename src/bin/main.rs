use std::collections::{BTreeMap, HashSet};

use glfw::{fail_on_errors, Action, ClientApiHint, Key, WindowHint};
use glm::*;
use wgpu_learning::{model::*, renderer_backend::*};

fn make_commands(commands: &mut RenderCommands, font_index: i32, font_size: f64) {
    let fonts = commands.fonts.keys().collect::<Vec<_>>();
    let font_id = *fonts[(font_index % fonts.len() as i32) as usize];

    let font = commands.fonts.get(&font_id).unwrap();

    let info = format!("{} {:0.2} px", font.name, font_size);

    let text = "Saturn is the sixth planet from the Sun and the \
        second largest in the Solar System, after Jupiter. It is a gas giant, \
        with an average radius of about 9 times that of Earth. It has an \
        eighth of the average density of Earth, but is over 95 times more \
        massive. Even though Saturn is almost as big as Jupiter, Saturn has \
        less than a third of its mass. Saturn orbits the Sun at a distance \
        of 9.59 AU (1,434 million km), with an orbital period of 29.45 years.\
        \n\n\
        Saturn's interior is thought to be composed of a rocky core, surrounded \
        by a deep layer of metallic hydrogen, an intermediate layer of liquid \
        hydrogen and liquid helium, and an outer layer of gas. Saturn has a \
        pale yellow hue, due to ammonia crystals in its upper atmosphere. An \
        electrical current in the metallic hydrogen layer is thought to give \
        rise to Saturn's planetary magnetic field, which is weaker than Earth's, \
        but has a magnetic moment 580 times that of Earth because of Saturn's \
        greater size. Saturn's magnetic field strength is about a twentieth \
        that of Jupiter.[27] The outer atmosphere is generally bland and \
        lacking in contrast, although long-lived features can appear. Wind \
        speeds on Saturn can reach 1,800 kilometres per hour (1,100 miles \
        per hour).\
        \n\n\
        The planet has a bright and extensive system of rings, composed mainly \
        of ice particles, with a smaller amount of rocky debris and dust. At \
        least 293 moons orbit the planet, of which 63 are officially named; \
        these do not include the hundreds of moonlets in the rings. Titan, \
        Saturn's largest moon and the second largest in the Solar System, is \
        larger (but less massive) than the planet Mercury and is the only moon \
        in the Solar System that has a substantial atmosphere.[28]";

    let layout_width = 1600.0;

    commands.paragraph(font_id, 40.0, 200.0, 100.0, &info, None);
    commands.paragraph(font_id, font_size, 200.0, 200.0, &text, Some(layout_width));

    let gray = Vec4::new(1.0, 1.0, 1.0, 0.3);
    let white = Vec4::new(1.0, 1.0, 1.0, 1.0);

    commands.rect(150.0, 100.0, 7.0, 1000.0, white);
    commands.rect(200.0, 180.0, layout_width, 7.0, gray);
}

fn make_world(renderer: &mut Renderer) -> World {
    let quad_id = renderer.spawn_mesh(make_quad(&renderer.device, 1.0));
    let cube_id = renderer.spawn_mesh(make_cube(&renderer.device, Vec4::new(1.0, 0.6, 0.6, 0.4)));
    let tetra_id = renderer.spawn_mesh(make_tetrahedron(&renderer.device));
    let nine_gon_id = renderer.spawn_mesh(make_n_gon(&renderer.device, 9));

    renderer.load_texture("img/invincible.jpg");

    renderer.load_font("cambria");
    renderer.load_font("consolas");
    renderer.load_font("garamond");
    renderer.load_font("arial");
    renderer.load_font("calibri");

    let mut world = World::new();

    for x in [-100, 0, 100] {
        for z in [-100, 0, 100] {
            let id = renderer.spawn_ground_plane(x, z, 100);
            world.ground_plane(x, z, id);
        }
    }

    world.quads.push(Object {
        position: Vec3::new(0.0, 6.0, -9.0),
        angle: 0.0,
        vel: 0.0,
        kind: EntityKind::Mesh,
        should_animate: false,
        mesh_id: nine_gon_id,
    });
    world.quads.push(Object {
        position: Vec3::new(0.0, 4.0, -5.6),
        angle: 0.0,
        vel: 0.0,
        kind: EntityKind::Mesh,
        should_animate: false,
        mesh_id: nine_gon_id,
    });
    world.quads.push(Object {
        position: Vec3::new(0.0, 5.0, 0.0),
        angle: 0.0,
        vel: 0.01,
        kind: EntityKind::Mesh,
        should_animate: false,
        mesh_id: tetra_id,
    });

    for i in 0..20 {
        let a = i as f32 / 5.0;
        let z = i as f32 * 1.0 - 10.0;
        world.quads.push(Object {
            position: Vec3::new(4.5, 3.0, z),
            angle: a,
            vel: 0.0,
            kind: EntityKind::Mesh,
            should_animate: false,
            mesh_id: quad_id,
        });
    }

    for i in (0..200).step_by(14) {
        let a = i as f32 / 6.0;
        let r = 3.0 + i as f32 / 8.0;
        let x = a.cos() * r;
        let z = a.sin() * r;
        world.quads.push(Object {
            position: Vec3::new(x as f32, 0.0, z as f32),
            angle: i as f32 * 0.4,
            vel: 1.0,
            kind: EntityKind::Mesh,
            should_animate: true,
            mesh_id: cube_id,
        });
    }

    world
}

fn make_camera_controls(keys: &HashSet<Key>) -> CameraControls {
    let mut ctrls = CameraControls::default();
    if keys.contains(&Key::Space) {
        ctrls.y_axis = CamDir::Positive
    }
    if keys.contains(&Key::LeftShift) {
        ctrls.y_axis = CamDir::Negative
    }
    if keys.contains(&Key::A) {
        ctrls.x_axis = CamDir::Negative;
    }
    if keys.contains(&Key::D) {
        ctrls.x_axis = CamDir::Positive;
    }
    if keys.contains(&Key::W) {
        ctrls.z_axis = CamDir::Positive;
    }
    if keys.contains(&Key::S) {
        ctrls.z_axis = CamDir::Negative;
    }
    ctrls
}

async fn run() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    let (mut window, events) = glfw
        .create_window(800, 600, "It's WGPU time.", glfw::WindowMode::Windowed)
        .unwrap();

    let mut renderer = Renderer::new(&mut window).await;

    renderer.window.set_framebuffer_size_polling(true);
    renderer.window.set_key_polling(true);
    renderer.window.set_mouse_button_polling(true);
    renderer.window.set_pos_polling(true);

    let mut keys_pressed = HashSet::new();

    // renderer.window.set_cursor_mode(glfw::CursorMode::Hidden);

    let mut world = make_world(&mut renderer);

    let mut font_index = 0i32;
    let mut font_size = 80.0f64;
    let mut target_font_size = 72.0f64;

    let font_info: BTreeMap<usize, FontInfo> = renderer
        .fonts
        .iter()
        .map(|(id, (font, _sprite))| (*id, font.clone()))
        .collect();

    while !renderer.window.should_close() {
        glfw.poll_events();

        renderer.update(&mut world);

        font_size += (target_font_size - font_size) * 0.03;

        let mut commands = RenderCommands::new(font_info.clone());

        make_commands(&mut commands, font_index, font_size);
        let ctrls = make_camera_controls(&keys_pressed);

        world.update(16.67 / 1000.0, &ctrls);

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(key, _, Action::Press, _) => {
                    keys_pressed.insert(key);
                }
                glfw::WindowEvent::Key(key, _, Action::Release, _) => {
                    keys_pressed.remove(&key);
                }
                _ => (),
            }

            match event {
                //Hit escape
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    renderer.window.set_should_close(true)
                }
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    renderer.paused ^= true;
                }
                glfw::WindowEvent::Key(Key::Z, _, Action::Press, _) => {
                    renderer.draw_wireframes ^= true;
                }
                glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) => {
                    renderer.pipeline_selector =
                        enum_iterator::next_cycle(&renderer.pipeline_selector);
                }
                glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) => {
                    renderer.pipeline_selector =
                        enum_iterator::previous_cycle(&renderer.pipeline_selector);
                }

                glfw::WindowEvent::Key(Key::M, _, Action::Press, _) => {
                    font_index += 1;
                    font_index = font_index.clamp(0, renderer.fonts.len() as i32 - 1);
                }
                glfw::WindowEvent::Key(Key::N, _, Action::Press, _) => {
                    font_index -= 1;
                    font_index = font_index.clamp(0, renderer.fonts.len() as i32 - 1);
                }

                glfw::WindowEvent::Key(Key::L, _, Action::Press, _) => {
                    target_font_size *= 1.1;
                    target_font_size = target_font_size.clamp(10.0, 250.0);
                }
                glfw::WindowEvent::Key(Key::K, _, Action::Press, _) => {
                    target_font_size /= 1.1;
                    target_font_size = target_font_size.clamp(10.0, 250.0);
                }

                //Window was moved
                glfw::WindowEvent::Pos(..) => {
                    renderer.update_surface();
                    renderer.resize(renderer.window.get_size());
                }

                //Window was resized
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    renderer.update_surface();
                    renderer.resize((width, height));
                }
                _ => {}
            }
        }

        match renderer.render(&mut world, &commands) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                renderer.update_surface();
                renderer.resize(renderer.window.get_size());
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

fn main() {
    pollster::block_on(run());
}
