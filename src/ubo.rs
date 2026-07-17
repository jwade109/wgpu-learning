pub struct UBO {
    pub buffer: wgpu::Buffer,
    pub bind_groups: Vec<wgpu::BindGroup>,
    alignment: u64,
}

impl UBO {
    pub fn new(device: &wgpu::Device, object_count: usize, layout: wgpu::BindGroupLayout) -> Self {
        let alignment = device.limits().min_uniform_buffer_offset_alignment;
        let alignment = glm::max(std::mem::size_of::<glm::Mat4>() as u32, alignment) as u64;

        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("UBO"),
            size: object_count as u64 * alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let buffer = device.create_buffer(&buffer_descriptor);

        let mut bind_groups: Vec<wgpu::BindGroup> = Vec::new();

        for i in 0..object_count {
            // TODO
        }

        todo!()
    }
}
