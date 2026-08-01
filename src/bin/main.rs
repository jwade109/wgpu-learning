use std::collections::HashSet;

use glfw::{fail_on_errors, Action, ClientApiHint, Key, WindowHint};
use glm::*;
use wgpu_learning::{model::*, renderer_backend::*};

fn make_world() -> World {
    let mut world = World::new();
    world.quads.push(Object {
        position: Vec3::new(0.0, 6.0, -9.0),
        angle: 0.0,
        vel: 0.0,
        kind: EntityKind::Mesh(MeshType::Polygon(9)),
        should_animate: false,
    });
    world.quads.push(Object {
        position: Vec3::new(0.0, 4.0, -5.6),
        angle: 0.0,
        vel: 0.0,
        kind: EntityKind::Mesh(MeshType::Polygon(3)),
        should_animate: false,
    });
    world.quads.push(Object {
        position: Vec3::new(0.0, 5.0, 0.0),
        angle: 0.0,
        vel: 0.01,
        kind: EntityKind::Mesh(MeshType::Tetradron),
        should_animate: false,
    });

    world.quads.push(Object {
        position: Vec3::new(0.0, 0.0, 0.0),
        angle: 0.0,
        vel: 0.0,
        kind: EntityKind::Mesh(MeshType::GroundPlane),
        should_animate: false,
    });

    for i in 0..20 {
        let a = i as f32 / 5.0;
        let z = i as f32 * 1.0 - 10.0;
        world.quads.push(Object {
            position: Vec3::new(4.5, 3.0, z),
            angle: a,
            vel: 0.0,
            kind: EntityKind::Mesh(MeshType::Quad),
            should_animate: false,
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
            kind: EntityKind::Mesh(MeshType::Cube),
            should_animate: true,
        });
    }

    for (x, y, w, h) in [
        (0, 0, 1, 1),
        (1, 0, 1, 1),
        (2, 0, 1, 1),
        // y = 1
        (1, 1, 2, 2),
        (3, 1, 2, 1),
        (5, 1, 3, 1),
        (8, 1, 1, 1),
        // y = 2
        (3, 2, 1, 1),
        (4, 2, 1, 3),
        (5, 2, 2, 1),
        (7, 2, 1, 1),
        // y = 3
        (1, 3, 1, 1),
        (2, 3, 1, 1),
        (3, 3, 1, 1),
        (5, 3, 1, 1),
        (6, 3, 1, 1),
        (7, 3, 1, 1),
        // y = 4
        (3, 4, 1, 1),
        (5, 4, 2, 1),
        // y = 6
        (6, 6, 2, 2),
        // y = 7
        (2, 7, 1, 1),
        (3, 7, 1, 1),
        (4, 7, 2, 1),
        (8, 7, 3, 1),
    ] {
        let size = 150;
        let pad = 10;
        let x = x * (size + pad) + pad;
        let y = y * (size + pad) + pad;
        let w = w * size + (w - 1) * pad;
        let h = h * size + (h - 1) * pad;
        world.ui(x, y, w, h);
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

    let mut world = make_world();

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
