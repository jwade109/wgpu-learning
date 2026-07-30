use glm::*;
use noise::{NoiseFn, Perlin};
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
    color: Vec4,
    tex_coord: Vec2,
}

impl Vertex {
    pub fn new(position: Vec3, color: Vec4, tex_coord: Vec2) -> Self {
        Self {
            position,
            color,
            tex_coord,
        }
    }

    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }

    pub fn to_bytes(&self) -> [u8; 36] {
        let arr = [
            self.position.x,
            self.position.y,
            self.position.z,
            self.color.x,
            self.color.y,
            self.color.z,
            self.color.w,
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
            Vec3::new(-size, -size, 0.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec2::new(0.0, 0.0),
        ),
        Vertex::new(
            Vec3::new(size, -size, 0.0),
            Vec4::new(0.0, 1.0, 1.0, 1.0),
            Vec2::new(1.0, 0.0),
        ),
        Vertex::new(
            Vec3::new(size, size, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 1.0),
            Vec2::new(1.0, 1.0),
        ),
        Vertex::new(
            Vec3::new(-size, size, 0.0),
            Vec4::new(1.0, 0.0, 1.0, 1.0),
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
            let pos = Vec3::new(x, y, 0.0);
            let r = a.sin() * 0.5 + 0.5;
            let g = a.cos() * 0.5 + 0.5;
            let b = (a * 0.5).sin() * 0.5 + 0.5;
            let color = Vec4::new(r, g, b, 1.0);
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

fn quad_indices_to_tris(a: u16, b: u16, c: u16, d: u16) -> [u16; 6] {
    [a, b, c, a, c, d]
}

pub fn make_cube(device: &wgpu::Device, color: Vec4) -> Mesh {
    let x = 0.5;
    let y = 0.5;
    let z = 0.5;
    let vertices = vec![
        Vertex::new(Vec3::new(-x, -y, -z), color, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(x, -y, -z), color, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(x, y, -z), color, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-x, y, -z), color, Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new(-x, -y, z), color, Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(x, -y, z), color, Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(x, y, z), color, Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-x, y, z), color, Vec2::new(0.0, 1.0)),
    ];

    let indices = [
        quad_indices_to_tris(3, 2, 1, 0),
        quad_indices_to_tris(4, 5, 6, 7),
        quad_indices_to_tris(0, 1, 5, 4),
        quad_indices_to_tris(2, 3, 7, 6),
        quad_indices_to_tris(3, 0, 4, 7),
        quad_indices_to_tris(1, 2, 6, 5),
    ]
    .concat();

    mesh_from_vi(device, &vertices, &indices)
}

pub fn make_rough_ground_plane(device: &wgpu::Device) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let n_quads_x = 100;
    let n_quads_y = 100;

    let perlin = Perlin::new(1);

    let eval_height = |x: f32, z: f32| {
        // let dsq = x.powi(2) + z.powi(2);
        // return (-1.0 / (0.01 * dsq)).clamp(-100.0, 1.0);
        let y1 = perlin.get([x as f64 / 5.0 + 0.5, 0.5, z as f64 / 5.0 + 0.5]);
        let y2 = perlin.get([x as f64 + 0.5, 0.5, z as f64 + 0.5]) * 0.4;
        let y3 = perlin.get([x as f64 / 18.0, 0.5, z as f64 / 18.0 + 0.5]) * 3.0;
        return y1 + y2 + y3;
    };

    for xi in 0..=n_quads_x {
        for zi in 0..=n_quads_y {
            let x = xi as f32 - 50.0;
            let z = zi as f32 - 50.0;
            let y = eval_height(x, z);
            let position = Vec3::new(x as f32, y as f32, z as f32);
            let color = Vec4::new(0.3, 0.8, 0.2, 1.0);
            let tex_coord = Vec2::new(0.0, 0.0);
            let v = Vertex::new(position, color, tex_coord);
            vertices.push(v);
        }
    }

    for x in 0..n_quads_x {
        for y in 0..n_quads_y {
            let stride = n_quads_y + 1;
            let b = x + y * (n_quads_y + 1);
            indices.extend(quad_indices_to_tris(b, b + 1, b + stride + 1, b + stride));
        }
    }

    mesh_from_vi(device, &vertices, &indices)
}
