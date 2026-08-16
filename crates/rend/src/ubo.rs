use crate::bind_group::BindGroupBuilder;

// From: https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

pub struct SingleUBO {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub struct UBO<T> {
    buffer: wgpu::Buffer,
    bind_groups: Vec<wgpu::BindGroup>,
    alignment: u64,
    _data: std::marker::PhantomData<T>,
    label: String,
}

impl<T> UBO<T> {
    pub fn new(
        device: &wgpu::Device,
        object_count: usize,
        layout: wgpu::BindGroupLayout,
        label: &str,
    ) -> Self {
        let min_alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let max_buffer_size = device.limits().max_uniform_buffer_binding_size as u64;
        let size_of_t = std::mem::size_of::<T>() as u64;
        let n_elements = max_buffer_size / size_of_t;

        let alignment = min_alignment.max(size_of_t);

        let n_actual_elements = max_buffer_size / alignment;

        let name = std::any::type_name::<T>();

        println!("Label:             {label}");
        println!("Type:              {name}");
        println!("Size of T:         {size_of_t}");
        println!("Min alignment:     {min_alignment}");
        println!("Max buffer size:   {max_buffer_size}");
        println!("Optimal storage:   {n_elements}");
        println!("Actual storage:    {n_actual_elements}");
        println!("Requested storage: {object_count}\n");

        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some(label),
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
            bind_groups.push(builder.build(label));
        }

        Self {
            buffer,
            bind_groups,
            alignment,
            _data: std::marker::PhantomData::default(),
            label: label.to_string(),
        }
    }

    pub fn bind_group(&self, i: usize) -> &wgpu::BindGroup {
        self.bind_groups.get(i).expect("Hey you dufus")
    }

    pub fn upload(&self, i: u64, matrix: &T, queue: &wgpu::Queue) {
        if i as usize >= self.bind_groups.len() {
            panic!(
                "Dude: {i} is greater than or equal to {} (UBO {})",
                self.bind_groups.len(),
                self.label
            );
        }
        let offset = i * self.alignment;
        let data: &[u8] = any_as_u8_slice(matrix);
        queue.write_buffer(&self.buffer, offset, data);
    }
}
