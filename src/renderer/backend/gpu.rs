use crate::{
    bvh::Node,
    log_info,
    renderer::Renderer,
    scene::{Material, Scene, Triangle},
    vector::{Mat4f, Vec3f},
};

mod buffer;
use buffer::Buffer;

pub async fn render_scene(renderer: &Renderer, scene: &Scene) -> Vec<u8> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
    });
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("Failed to create adapter");
    log_info!("{:#?}", adapter.get_info());
    log_info!(
        "Max storage buffer binding size: {} MB",
        adapter.limits().max_storage_buffer_binding_size as f32 / 1024.0 / 1024.0
    );

    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    if !downlevel_capabilities
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        panic!("Adapter does not support compute shaders");
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | wgpu::Features::TIMESTAMP_QUERY
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
            | wgpu::Features::TEXTURE_BINDING_ARRAY
            | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
        required_limits: adapter.limits(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("Failed to create device");

    let module = device.create_shader_module(wgpu::include_wgsl!("./rt_compute.wgsl"));

    let storage_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: renderer.options.output_image_dimensions.0 as u32,
            height: renderer.options.output_image_dimensions.1 as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let storage_texture_view = storage_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (renderer.options.output_image_dimensions.0
            * 4
            * renderer.options.output_image_dimensions.1) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let storage_buffers = StorageBuffers::setup(&device, scene);
    let uniform_buffers = UniformBuffers::setup(&device, scene, renderer);

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &texture_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&storage_texture_view),
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            &texture_bind_group_layout,
            &storage_buffers.bind_group_layout,
            &uniform_buffers.bind_group_layout,
        ],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let mut command_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        // Begin compute pass
        let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_bind_group(0, &texture_bind_group, &[]);
        compute_pass.set_bind_group(1, &storage_buffers.bind_group, &[]);
        compute_pass.set_bind_group(2, &uniform_buffers.bind_group, &[]);
        compute_pass.set_pipeline(&pipeline);
        compute_pass.dispatch_workgroups(
            (renderer.options.output_image_dimensions.0 / 8) as u32,
            (renderer.options.output_image_dimensions.1 / 8) as u32,
            1,
        );
        // End compute pass
    }

    command_encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &storage_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_staging_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((renderer.options.output_image_dimensions.0 * 4) as u32),
                rows_per_image: Some(renderer.options.output_image_dimensions.1 as u32),
            },
        },
        wgpu::Extent3d {
            width: renderer.options.output_image_dimensions.0 as u32,
            height: renderer.options.output_image_dimensions.1 as u32,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(command_encoder.finish()));

    let mut output_data: Vec<u8> = vec![];
    let buffer_slice = output_staging_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv_async().await.unwrap().unwrap();
    {
        let view = buffer_slice.get_mapped_range();
        output_data.resize(view.len(), 0);
        output_data.copy_from_slice(&view[..]);
    }
    output_staging_buffer.unmap();

    return output_data;
}

#[allow(dead_code)]
struct StorageBuffers {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    triangle_buffer: Buffer,
    bvh_buffer: Buffer,
    material_buffer: Buffer,
    texture_data_buffer: Buffer,
    texture_info_buffer: Buffer,
}

impl StorageBuffers {
    fn setup(device: &wgpu::Device, scene: &Scene) -> Self {
        let triangle_buffer = Buffer::create_storage_buffer(device, 0, &scene.tris);
        let bvh_buffer = Buffer::create_storage_buffer(device, 1, &scene.bvh.nodes);
        let material_buffer = Buffer::create_storage_buffer(device, 2, &scene.materials);
        log_info!(
            "Created a storage buffer for scene triangles: {:.2} MB ({} tris)",
            triangle_buffer.buffer.size() as f32 / 1024.0 / 1024.0,
            triangle_buffer.buffer.size() / size_of::<Triangle>() as u64
        );
        log_info!(
            "Created a storage buffer for BVH nodes: {:.2} MB ({} nodes)",
            bvh_buffer.buffer.size() as f32 / 1024.0 / 1024.0,
            bvh_buffer.buffer.size() / size_of::<Node>() as u64
        );
        log_info!(
            "Created a storage buffer for materials: {:.2} KB ({} materials)",
            material_buffer.buffer.size() as f32 / 1024.0,
            material_buffer.buffer.size() / size_of::<Material>() as u64
        );
        let mut texture_data: Vec<u32> = Vec::new();
        let mut texture_info: Vec<[u32; 3]> = Vec::new();
        if !scene.textures.is_empty() {
            scene.textures.iter().for_each(|texture| {
                let data = texture
                    .pixel_data
                    .iter()
                    .map(|bytes| {
                        return u32::from_le_bytes(*bytes);
                    })
                    .collect::<Vec<u32>>();
                texture_info.push([
                    texture.width as u32,
                    texture.height as u32,
                    texture_data.len() as u32,
                ]);
                texture_data.extend_from_slice(data.as_slice());
            });
        } else {
            texture_data.push(0);
            texture_info.push([0, 0, 0]);
        }
        let texture_data_buffer = Buffer::create_storage_buffer(device, 3, &texture_data);
        let texture_info_buffer = Buffer::create_storage_buffer(device, 4, &texture_info);
        log_info!(
            "Created a storage buffer for textures: {:.2} MB ({} textures)",
            texture_data_buffer.buffer.size() as f32 / 1024.0 / 1024.0,
            scene.textures.len()
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                triangle_buffer.bind_group_layout_entry,
                bvh_buffer.bind_group_layout_entry,
                material_buffer.bind_group_layout_entry,
                texture_data_buffer.bind_group_layout_entry,
                texture_info_buffer.bind_group_layout_entry,
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                triangle_buffer.bind_group_entry(),
                bvh_buffer.bind_group_entry(),
                material_buffer.bind_group_entry(),
                texture_data_buffer.bind_group_entry(),
                texture_info_buffer.bind_group_entry(),
            ],
        });

        return Self {
            bind_group,
            bind_group_layout,
            triangle_buffer,
            bvh_buffer,
            material_buffer,
            texture_data_buffer,
            texture_info_buffer,
        };
    }
}

#[allow(dead_code)]
struct UniformBuffers {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    camera_buffer: Buffer,
    renderer_buffer: Buffer,
}

impl UniformBuffers {
    fn setup(device: &wgpu::Device, scene: &Scene, renderer: &Renderer) -> Self {
        let uniform_camera = UniformCamera {
            look_at: scene.camera.look_at,
            position: scene.camera.position,
            _pad: [0; 4],
        };
        let camera_buffer = Buffer::create_uniform_buffer(device, 0, &[uniform_camera]);
        let uniform_renderer = UniformRenderer {
            samples: renderer.options.samples as u32,
            max_ray_depth: renderer.options.max_ray_depth as u32,
        };
        let renderer_buffer = Buffer::create_uniform_buffer(device, 1, &[uniform_renderer]);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                camera_buffer.bind_group_layout_entry,
                renderer_buffer.bind_group_layout_entry,
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                camera_buffer.bind_group_entry(),
                renderer_buffer.bind_group_entry(),
            ],
        });

        return Self {
            bind_group,
            bind_group_layout,
            camera_buffer,
            renderer_buffer,
        };
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
struct UniformCamera {
    look_at: Mat4f,
    position: Vec3f,
    _pad: [u8; 4],
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct UniformRenderer {
    samples: u32,
    max_ray_depth: u32,
}
