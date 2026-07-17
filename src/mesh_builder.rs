use glm::*;
use wgpu::util::DeviceExt;

struct Vertex {
    position: Vec3,
    color: Vec3,
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1=> Float32x3];

    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

pub fn make_triangle(device: &wgpu::Device) -> wgpu::Buffer {
    let vertices = [
        Vertex {
            position: Vec3::new(-0.75, -0.75, 0.0),
            color: Vec3::new(1.0, 0.0, 0.0),
        },
        Vertex {
            position: Vec3::new(0.75, -0.75, 0.0),
            color: Vec3::new(0.0, 1.0, 0.0),
        },
        Vertex {
            position: Vec3::new(0.0, 0.75, 0.0),
            color: Vec3::new(0.0, 0.0, 1.0),
        },
    ];

    let bytes = any_as_u8_slice(&vertices);

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("triangle vertex buffer"),
        contents: bytes,
        usage: wgpu::BufferUsages::VERTEX,
    };

    let buffer = device.create_buffer_init(&buffer_descriptor);

    buffer
}
