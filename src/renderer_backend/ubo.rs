use crate::renderer_backend::bind_group::BindGroupBuilder;

// From: https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

pub struct SingleUBO {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub struct UBO {
    buffer: wgpu::Buffer,
    bind_groups: Vec<wgpu::BindGroup>,
    alignment: u64,
}

impl UBO {
    pub fn new(device: &wgpu::Device, object_count: usize, layout: wgpu::BindGroupLayout) -> Self {
        let alignment = glm::max(
            device.limits().min_storage_buffer_offset_alignment as u32,
            std::mem::size_of::<glm::Mat4>() as u32,
        ) as u64;

        println!("Alignment is {alignment}");

        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("UBO"),
            size: object_count as u64 * alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&buffer_descriptor);

        // build bind groups
        let mut bind_groups: Vec<wgpu::BindGroup> = Vec::new();
        for i in 0..object_count {
            let mut builder = BindGroupBuilder::new(device);
            builder.set_layout(&layout);
            builder.add_buffer(&buffer, i as u64 * alignment);
            bind_groups.push(builder.build("Matrix"));
        }

        Self {
            buffer,
            bind_groups,
            alignment,
        }
    }

    pub fn bind_group(&self, i: usize) -> &wgpu::BindGroup {
        self.bind_groups.get(i).expect("Hey you dufus")
    }

    pub fn upload(&mut self, i: u64, matrix: &glm::Mat4, queue: &wgpu::Queue) {
        if i as usize >= self.bind_groups.len() {
            panic!("Dude");
        }
        let offset = i * self.alignment;
        let data: &[u8] = any_as_u8_slice(matrix);
        queue.write_buffer(&self.buffer, offset, data);
    }
}
