use std::collections::HashSet;

use glfw::{fail_on_errors, Action, ClientApiHint, Key, WindowHint};
use glm::*;
use wgpu_learning::{model::*, renderer_backend::*};

fn make_string_commands(
    commands: &mut RenderCommands,
    renderer: &Renderer,
    font_id: usize,
    font_size: f64,
    mut x_origin: f64,
    mut y_origin: f64,
    text: &str,
) {
    let (font, _sprite) = renderer.fonts.get(&font_id).unwrap();

    let mut col_offset = 0;

    for ch in text.chars() {
        if ch == '\n' {
            y_origin += font.size as f64 * font_size;
            x_origin = 100.0;
            col_offset = 0;
            continue;
        }

        let Some(data) = font.characters.get(&ch) else {
            continue;
        };

        if ch == ' ' && col_offset == 0 {
            continue;
        }

        let w = data.width as f64 * font_size;
        let h = data.height as f64 * font_size;

        let x = x_origin - data.origin_x as f64 * font_size;
        let y = y_origin - data.origin_y as f64 * font_size + font.size as f64 * font_size;

        let x = x.round() as i32;
        let y = y.round() as i32;

        let w = w.round() as i32;
        let h = h.round() as i32;

        if ch != ' ' {
            commands.char(x, y, w, h, ch, font_id);
        }

        col_offset += 1;

        x_origin += data.advance as f64 * font_size;

        if ch == ' ' && x_origin + 400.0 + w as f64 > renderer.window.get_size().0 as f64 {
            y_origin += font.size as f64 * font_size;
            x_origin = 100.0;
            col_offset = 0;
        }
    }
}

fn make_commands(renderer: &Renderer, font_index: i32, font_size: f64) -> RenderCommands {
    let fonts = renderer.fonts.keys().collect::<Vec<_>>();
    let font_id = *fonts[(font_index % fonts.len() as i32) as usize];

    let (font, _) = renderer.fonts.get(&font_id).unwrap();

    let info = format!("{} {:0.2}", font.name, font_size);

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
        per hour).";

    let mut commands = RenderCommands::new();

    make_string_commands(&mut commands, renderer, font_id, 0.8, 100.0, 100.0, &info);

    make_string_commands(
        &mut commands,
        renderer,
        font_id,
        font_size,
        100.0,
        200.0,
        &text,
    );

    commands
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
    let mut font_size = 1.1f64;
    let mut target_font_size = 1.0f64;

    while !renderer.window.should_close() {
        glfw.poll_events();

        renderer.update(&mut world);

        font_size += (target_font_size - font_size) * 0.03;

        let commands = make_commands(&renderer, font_index, font_size);
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
                    target_font_size = target_font_size.clamp(0.1, 4.0);
                }
                glfw::WindowEvent::Key(Key::K, _, Action::Press, _) => {
                    target_font_size /= 1.1;
                    target_font_size = target_font_size.clamp(0.1, 4.0);
                }

                //Window was moved
                glfw::WindowEvent::Pos(..) => {
                    renderer.update_surface();
                    renderer.resize(renderer.size);
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
                renderer.resize(renderer.size);
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

fn main() {
    pollster::block_on(run());
}
