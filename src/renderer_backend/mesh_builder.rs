use glm::*;
use wgpu::util::DeviceExt;

pub struct Mesh {
    buffer: wgpu::Buffer,
    offset: u64,
}

impl Mesh {
    pub fn vertex_buffer(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(0..self.offset)
    }

    pub fn index_buffer(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(self.offset..)
    }

    pub fn index_format(&self) -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}

#[repr(C)] // C-style data layout
pub struct Vertex {
    position: Vec3,
    color: Vec3,
    texture_coord: Vec2,
    barycentric: UVec2,
}

impl Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Uint32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

// From: https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

pub fn make_triangle(device: &wgpu::Device) -> Mesh {
    let w = 0.8;
    let vertices: [Vertex; 3] = [
        Vertex {
            position: Vec3::new(-w, -w, 0.0),
            color: Vec3::new(1.0, 0.0, 0.0),
            texture_coord: Vec2::new(0.0, 0.0),
            barycentric: UVec2::new(0, 0),
        },
        Vertex {
            position: Vec3::new(w, -w, 0.0),
            color: Vec3::new(0.0, 1.0, 0.0),
            texture_coord: Vec2::new(1.0, 0.0),
            barycentric: UVec2::new(0, 1),
        },
        Vertex {
            position: Vec3::new(0.0, w, 0.0),
            color: Vec3::new(0.0, 0.0, 1.0),
            texture_coord: Vec2::new(1.0, 1.0),
            barycentric: UVec2::new(1, 0),
        },
    ];

    let indices: [u16; 3] = [0, 1, 2];

    let bytes_1: &[u8] = any_as_u8_slice(&vertices);
    let bytes_2: &[u8] = any_as_u8_slice(&indices);
    let bytes_merged: &[u8] = &[bytes_1, bytes_2].concat();

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Triangle vertex & index buffer"),
        contents: bytes_merged,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
    };

    let buffer = device.create_buffer_init(&buffer_descriptor);
    let offset: u64 = bytes_1.len().try_into().unwrap();

    Mesh { buffer, offset }
}

pub fn make_quad(device: &wgpu::Device) -> Mesh {
    let w = 0.8;

    let vertices: [Vertex; 4] = [
        Vertex {
            position: Vec3::new(-w, -w, 0.0),
            color: Vec3::new(1.0, 0.0, 0.0),
            texture_coord: Vec2::new(0.0, 0.0),
            barycentric: UVec2::new(0, 0),
        },
        Vertex {
            position: Vec3::new(w, -w, 0.0),
            color: Vec3::new(0.0, 1.0, 1.0),
            texture_coord: Vec2::new(1.0, 0.0),
            barycentric: UVec2::new(1, 0),
        },
        Vertex {
            position: Vec3::new(w, w, 0.0),
            color: Vec3::new(0.0, 0.0, 1.0),
            texture_coord: Vec2::new(1.0, 1.0),
            barycentric: UVec2::new(0, 1),
        },
        Vertex {
            position: Vec3::new(-w, w, 0.0),
            color: Vec3::new(1.0, 0.0, 1.0),
            texture_coord: Vec2::new(0.0, 1.0),
            barycentric: UVec2::new(1, 1),
        },
    ];
    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

    let bytes_1: &[u8] = any_as_u8_slice(&vertices);
    let bytes_2: &[u8] = any_as_u8_slice(&indices);
    let bytes_merged: &[u8] = &[bytes_1, bytes_2].concat();

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad vertex & index buffer"),
        contents: bytes_merged,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
    };

    let buffer = device.create_buffer_init(&buffer_descriptor);
    let offset: u64 = bytes_1.len().try_into().unwrap();

    Mesh { buffer, offset }
}
