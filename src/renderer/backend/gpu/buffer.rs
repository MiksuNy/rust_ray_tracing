use std::num::NonZeroU64;

use wgpu::util::DeviceExt;

pub struct Buffer {
    pub buffer: wgpu::Buffer,
    pub binding: u32,
    pub bind_group_layout_entry: wgpu::BindGroupLayoutEntry,
}

impl Buffer {
    pub fn create_storage_buffer<T: bytemuck::NoUninit>(
        device: &wgpu::Device,
        binding: u32,
        data: &[T],
    ) -> Self {
        return Self {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::STORAGE,
            }),
            bind_group_layout_entry: wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(size_of::<T>() as u64)).unwrap(),
                },
                count: None,
            },
            binding,
        };
    }

    pub fn create_uniform_buffer<T: bytemuck::NoUninit>(
        device: &wgpu::Device,
        binding: u32,
        data: &[T],
    ) -> Self {
        return Self {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
            bind_group_layout_entry: wgpu::BindGroupLayoutEntry {
                binding: binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(size_of::<T>() as u64)).unwrap(),
                },
                count: None,
            },
            binding,
        };
    }

    pub fn bind_group_entry<'a>(&'a self) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: self.buffer.as_entire_binding(),
        }
    }
}
