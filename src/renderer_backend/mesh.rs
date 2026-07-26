use glm::*;
use wgpu::util::DeviceExt;

pub struct Mesh {
    buffer: wgpu::Buffer,
    offset: u64,
    index_count: u32,
}

impl Mesh {
    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    pub fn vertex_buffer(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(0..self.offset)
    }

    pub fn index_buffer(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(self.offset..)
    }

    pub fn index_format(&self) -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }

    pub fn set_as_active(&self, rp: &mut wgpu::RenderPass) {
        rp.set_vertex_buffer(0, self.vertex_buffer());
        rp.set_index_buffer(self.index_buffer(), self.index_format());
    }
}

#[repr(C)] // C-style data layout
pub struct Vertex {
    position: Vec3,
    color: Vec3,
    tex_coord: Vec2,
}

impl Vertex {
    pub fn new(position: Vec3, color: Vec3, tex_coord: Vec2) -> Self {
        Self {
            position,
            color,
            tex_coord,
        }
    }

    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let arr = [
            self.position.x,
            self.position.y,
            self.position.z,
            self.color.x,
            self.color.y,
            self.color.z,
            self.tex_coord.x,
            self.tex_coord.y,
        ];

        bytemuck::bytes_of(&arr).try_into().unwrap()
    }
}

fn vertices_to_bytes(vertices: &[Vertex]) -> Vec<u8> {
    vertices
        .iter()
        .map(|v| v.to_bytes().to_vec())
        .collect::<Vec<Vec<u8>>>()
        .concat()
}

fn indices_to_bytes(indices: &[u16]) -> Vec<u8> {
    indices
        .iter()
        .map(|i| i.to_le_bytes().to_vec())
        .collect::<Vec<Vec<u8>>>()
        .concat()
}

fn mesh_from_vi(device: &wgpu::Device, vertices: &[Vertex], indices: &[u16]) -> Mesh {
    let bytes_1: &[u8] = &vertices_to_bytes(vertices);
    let bytes_2: &[u8] = &indices_to_bytes(&indices);
    let bytes_merged: &[u8] = &[bytes_1, bytes_2].concat();

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad vertex & index buffer"),
        contents: bytes_merged,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
    };

    let buffer = device.create_buffer_init(&buffer_descriptor);
    let offset: u64 = bytes_1.len().try_into().unwrap();

    Mesh {
        buffer,
        offset,
        index_count: indices.len() as u32,
    }
}

pub fn make_quad(device: &wgpu::Device, size: f32) -> Mesh {
    let vertices: [Vertex; 4] = [
        Vertex::new(
            Vec3::new(-size, -size, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec2::new(0.0, 0.0),
        ),
        Vertex::new(
            Vec3::new(size, -size, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec2::new(1.0, 0.0),
        ),
        Vertex::new(
            Vec3::new(size, size, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec2::new(1.0, 1.0),
        ),
        Vertex::new(
            Vec3::new(-size, size, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec2::new(0.0, 1.0),
        ),
    ];

    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

    mesh_from_vi(device, &vertices, &indices)
}

pub fn make_n_gon(device: &wgpu::Device, n: usize) -> Mesh {
    let vertices = (0..n)
        .map(|i| {
            let a = 2.0 * std::f32::consts::PI * i as f32 / n as f32;
            let x = a.cos();
            let y = a.sin();
            (a, x, y)
        })
        .map(|(a, x, y)| {
            let pos = Vec3::new(x, y, 1.0);
            let r = a.sin() * 0.5 + 0.5;
            let g = a.cos() * 0.5 + 0.5;
            let b = (a * 0.5).sin() * 0.5 + 0.5;
            let color = Vec3::new(r, g, b);
            let tx = Vec2::new(x, y);
            Vertex::new(pos, color, tx)
        })
        .collect::<Vec<_>>();

    let indices: Vec<u16> = (0..n - 2)
        .map(|i| {
            let i = i as u16;
            [0, 1 + i, 2 + i]
        })
        .collect::<Vec<_>>()
        .concat();

    mesh_from_vi(device, &vertices, &indices)
}
