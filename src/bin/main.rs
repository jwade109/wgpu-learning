use std::collections::HashSet;

use glfw::{fail_on_errors, Action, ClientApiHint, Key, WindowHint};
use glm::*;
use wgpu_learning::{model::*, renderer_backend::*};

fn make_world(renderer: &mut Renderer) -> World {
    let quad_id = renderer.spawn_mesh(make_quad(&renderer.device, 1.0));
    let cube_id = renderer.spawn_mesh(make_cube(&renderer.device, Vec4::new(1.0, 0.6, 0.6, 0.4)));
    let tetra_id = renderer.spawn_mesh(make_tetrahedron(&renderer.device));
    let nine_gon_id = renderer.spawn_mesh(make_n_gon(&renderer.device, 9));

    let fun_id = renderer.load_texture("img/invincible.jpg");

    let font_id = renderer.load_font("cambria");
    let font_id = renderer.load_font("consolas");
    let font_id = renderer.load_font("garamond");
    let font_id = renderer.load_font("arial");

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

    let mut x_origin = 100;
    let mut y_origin = 200;
    let scale = 0.7;

    let (font, _) = renderer.fonts.get(&font_id).unwrap();

    let text = "Lorem Ipsum is simply dummy text of the printing and typesetting \
        industry. Lorem Ipsum has been the industry's standard dummy text ever since 1966, \
        when designers at Letraset and James Mosley, the librarian at St Bride Printing Library \
        in London, took a 1914 Cicero translation and scrambled it to make dummy text for \
        Letraset's Body Type sheets. It has survived not only many decades, but also the \
        leap into electronic typesetting, remaining essentially unchanged. \
        It was popularised thanks to these sheets and more recently with desktop publishing \
        software like Aldus PageMaker and Microsoft Word including versions of Lorem Ipsum.";

    for (i, ch) in text.char_indices() {

        let Some(data) = font.characters.get(&ch) else {
            continue;
        };

        let w = data.width as f32 * scale;
        let h = data.height as f32 * scale;

        let x = x_origin - (data.origin_x as f32 * scale).round() as i32;
        let y = y_origin - (data.origin_y as f32 * scale).round() as i32;

        world.ui(x, y, w.round() as i32, h.round() as i32, ch, font_id);

        x_origin += (data.advance as f32 * scale).round() as i32;

        if i % 60 == 0 && i > 0 {
            y_origin += (100 as f32 * scale) as i32;
            x_origin = 100;
        }
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

    while !renderer.window.should_close() {
        glfw.poll_events();

        renderer.update(&mut world);

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

        match renderer.render(&mut world) {
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
