use std::num::NonZeroU32;

use crate::{model::translation_matrix, renderer_backend::*};
use glm::{Mat4, Vec3, Vec4};
use wgpu::*;

pub struct TextPipeline {
    pipeline: RenderPipeline,
    colors: BufferResource,
    range_info: BufferResource,
    transforms: BufferResource,
}

pub struct BufferResource {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub layout: BindGroupLayout,
}

fn make_array_resource(device: &Device, n_elements: usize, elem_size: usize) -> BufferResource {
    let n_bytes = n_elements * elem_size;
    println!("Allocating buffer with {n_elements} * {elem_size} = {n_bytes} bytes");

    let bd = BufferDescriptor {
        label: Some("Text pipeline color buffer"),
        size: n_bytes.try_into().unwrap(),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };

    let buffer = device.create_buffer(&bd);

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Text pipeline color UBO"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::all(),
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: NonZeroU32::new(10),
        }],
    });

    let bg = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Color array bind group"),
        layout: &bgl,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: None,
            }),
        }],
    });

    BufferResource {
        buffer,
        bind_group: bg,
        layout: bgl,
    }
}

pub struct GpuSampleInfo {
    pub origin_x: u32,
    pub origin_y: u32,
    pub sample_width: u32,
    pub sample_height: u32,
    pub image_width: u32,
    pub image_height: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

impl TextPipeline {
    pub const MAX_CHARS_PER_PASS: usize = 480;

    pub fn new(device: &Device, config: &SurfaceConfiguration, queue: &Queue) -> Self {
        let size = std::mem::size_of::<GpuSampleInfo>();
        assert!(size == 4 * 8);
        let range_info = make_array_resource(device, Self::MAX_CHARS_PER_PASS, size);
        let colors = make_array_resource(device, Self::MAX_CHARS_PER_PASS, 16);
        let transforms = make_array_resource(device, Self::MAX_CHARS_PER_PASS, 64);

        let material_bind_group_layout;
        {
            let mut builder = BindGroupLayoutBuilder::new(&device);
            builder.add_material();
            material_bind_group_layout = builder.build("SpriteMaterial Bind Group Layout");
        }

        let mut builder = PipelineBuilder::new(&device);
        let shader = Shader::from_path("src/shaders/text_shader.wgsl");

        builder.add_bind_group_layout(&material_bind_group_layout);
        builder.add_bind_group_layout(&colors.layout);
        builder.add_bind_group_layout(&range_info.layout);
        builder.add_bind_group_layout(&transforms.layout);

        let pipeline = builder.build_pipeline::<FullVertex>(
            "Single Texture Pipeline",
            &shader,
            config.format,
            true,
            true,
        );

        for i in 0..Self::MAX_CHARS_PER_PASS {
            let color = [1.0f32, 1.0, 1.0, 1.0];
            queue.write_buffer(&colors.buffer, 16 * i as u64, any_as_u8_slice(&color));
        }

        Self {
            pipeline,
            colors,
            range_info,
            transforms,
        }
    }

    pub fn set_color(&mut self, queue: &Queue, i: usize, color: Vec4) {
        queue.write_buffer(&self.colors.buffer, 16 * i as u64, any_as_u8_slice(&color));
    }

    pub fn set_transform(&mut self, queue: &Queue, i: usize, transform: &Mat4) {
        queue.write_buffer(
            &self.transforms.buffer,
            64 * i as u64,
            any_as_u8_slice(transform),
        );
    }

    pub fn set_range(&mut self, queue: &Queue, i: usize, range: &TextureSampleRange) {
        let gpu = GpuSampleInfo {
            origin_x: range.origin_x,
            origin_y: range.origin_y,
            sample_width: range.sample_width,
            sample_height: range.sample_height,
            image_width: range.image_width,
            image_height: range.image_height,
            _pad1: 0,
            _pad2: 0,
        };

        queue.write_buffer(
            &self.range_info.buffer,
            32 * i as u64,
            any_as_u8_slice(&gpu),
        );
    }

    pub fn draw_text(&self, rp: &mut RenderPass, mesh: &Mesh, material: &SpriteMaterial, n: usize) {
        rp.set_pipeline(&self.pipeline);

        rp.set_bind_group(0, material.bind_group(), &[]);
        rp.set_bind_group(1, &self.colors.bind_group, &[]);
        rp.set_bind_group(2, &self.range_info.bind_group, &[]);
        rp.set_bind_group(3, &self.transforms.bind_group, &[]);

        let n = n.min(Self::MAX_CHARS_PER_PASS);

        mesh.set_as_active(rp);
        rp.draw_indexed(0..mesh.index_count(), 0, 0..n as u32);
    }
}

pub fn char_transform(x: f64, y: f64, width: f64, height: f64, sx: f64, sy: f64) -> Mat4 {
    let width_scale = width as f32 / sx as f32;
    let height_scale = height as f32 / sy as f32;

    let xoff = 2.0 * (x as f32 + width as f32 / 2.0) / sx as f32 - 1.0;
    let yoff = -(2.0 * (y as f32 + height as f32 / 2.0) / sy as f32 - 1.0);

    translation_matrix(Vec3::new(xoff, yoff, 0.0))
        * mat4_diagonal(width_scale, height_scale, 1.0, 1.0)
}
