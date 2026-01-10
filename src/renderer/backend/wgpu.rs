use std::num::NonZeroU64;

use wgpu::util::DeviceExt;

use crate::{
    bvh::Node,
    log_info,
    renderer::Renderer,
    scene::{Material, Scene, Triangle},
    vector::{Mat4f, Vec3f},
};

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

    unsafe {
        device.start_graphics_debugger_capture();
    }

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

    let triangle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Triangles"),
        contents: bytemuck::cast_slice(scene.tris.as_slice()),
        usage: wgpu::BufferUsages::STORAGE,
    });
    log_info!(
        "Created a storage buffer for scene triangles: {:.2} MB ({} tris)",
        triangle_buffer.size() as f32 / 1024.0 / 1024.0,
        triangle_buffer.size() / size_of::<Triangle>() as u64
    );
    let bvh_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("BVH nodes"),
        contents: bytemuck::cast_slice(scene.bvh.nodes.as_slice()),
        usage: wgpu::BufferUsages::STORAGE,
    });
    log_info!(
        "Created a storage buffer for BVH nodes: {:.2} MB ({} nodes)",
        bvh_buffer.size() as f32 / 1024.0 / 1024.0,
        bvh_buffer.size() / size_of::<Node>() as u64
    );
    let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Materials"),
        contents: bytemuck::cast_slice(scene.materials.as_slice()),
        usage: wgpu::BufferUsages::STORAGE,
    });
    log_info!(
        "Created a storage buffer for materials: {:.2} KB ({} materials)",
        material_buffer.size() as f32 / 1024.0,
        material_buffer.size() / size_of::<Material>() as u64
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
    let texture_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Texture data"),
        contents: bytemuck::cast_slice(texture_data.as_slice()),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let texture_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Texture info"),
        contents: bytemuck::cast_slice(texture_info.as_slice()),
        usage: wgpu::BufferUsages::STORAGE,
    });
    log_info!(
        "Created a storage buffer for textures: {:.2} MB ({} textures)",
        texture_data_buffer.size() as f32 / 1024.0 / 1024.0,
        scene.textures.len()
    );

    let camera_look_at_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera look at"),
        contents: bytemuck::cast_slice(&scene.camera.look_at.data),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let camera_position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera position"),
        contents: bytemuck::cast_slice(&scene.camera.position.data),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let buffers_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(size_of::<Triangle>() as u64)).unwrap(),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(size_of::<Node>() as u64)).unwrap(),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(size_of::<Material>() as u64)).unwrap(),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(4u64)).unwrap(),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(12u64)).unwrap(),
                },
                count: None,
            },
        ],
    });
    let uniforms_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(size_of::<Mat4f>() as u64)).unwrap(),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(size_of::<Vec3f>() as u64)).unwrap(),
                },
                count: None,
            },
        ],
    });
    let buffers_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &buffers_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&storage_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: triangle_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: bvh_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: material_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: texture_data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: texture_info_buffer.as_entire_binding(),
            },
        ],
    });
    let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &uniforms_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_look_at_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: camera_position_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&buffers_group_layout, &uniforms_group_layout],
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

    let timestamp_capacity: u32 = 2;
    let timestamp_query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
        label: None,
        ty: wgpu::QueryType::Timestamp,
        count: timestamp_capacity,
    });
    let timestamp_query_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (timestamp_capacity * 8) as u64,
        usage: wgpu::BufferUsages::QUERY_RESOLVE
            | wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut command_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        // Begin compute pass
        let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_bind_group(0, &buffers_bind_group, &[]);
        compute_pass.set_bind_group(1, &uniforms_bind_group, &[]);
        compute_pass.set_pipeline(&pipeline);
        compute_pass.write_timestamp(&timestamp_query_set, 0);
        compute_pass.dispatch_workgroups(
            (renderer.options.output_image_dimensions.0 / 8) as u32,
            (renderer.options.output_image_dimensions.1 / 8) as u32,
            1,
        );
        compute_pass.write_timestamp(&timestamp_query_set, 1);
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

    command_encoder.resolve_query_set(
        &timestamp_query_set,
        0..timestamp_capacity,
        &timestamp_query_buffer,
        0,
    );

    queue.submit(Some(command_encoder.finish()));

    let timestamp_read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: timestamp_query_buffer.size(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut copy_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    copy_encoder.copy_buffer_to_buffer(
        &timestamp_query_buffer,
        0,
        &timestamp_read_buffer,
        0,
        timestamp_query_buffer.size(),
    );
    queue.submit(Some(copy_encoder.finish()));
    timestamp_read_buffer.map_async(wgpu::MapMode::Read, .., |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let timestamps_range = timestamp_read_buffer.get_mapped_range(..);
    let timestamps = bytemuck::cast_slice::<u8, u64>(timestamps_range.get(..).unwrap());
    let timestamp_difference_ms = (timestamps[1] - timestamps[0]) as f32 / 1000.0 / 1000.0;
    log_info!(
        "Compute shader dispatch took {:.2} ms ({:.2} fps)",
        timestamp_difference_ms,
        1000.0 / timestamp_difference_ms
    );

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

    unsafe {
        device.stop_graphics_debugger_capture();
    }

    return output_data;
}
