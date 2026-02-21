use crate::{
    bvh::Node,
    log_info,
    renderer::Renderer,
    scene::{Camera, Material, Scene, Triangle},
    vector::{Mat4f, Vec3f},
};

mod buffer;
pub mod window;
use buffer::Buffer;

pub async fn render_scene_to_buffer(renderer: Renderer, scene: &Scene) -> Vec<u8> {
    let mut state = State::new(renderer, scene);

    for _ in 0..renderer.options.max_samples {
        let mut command_encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
            compute_pass.set_bind_group(0, &state.storage_texture_bind_group, &[]);
            compute_pass.set_bind_group(1, &state.storage_buffers.bind_group, &[]);
            compute_pass.set_bind_group(2, &state.uniform_buffers.bind_group, &[]);
            compute_pass.set_pipeline(&state.compute_pipeline);
            compute_pass.set_immediates(0, bytemuck::cast_slice(&[state.renderer_info]));
            compute_pass.dispatch_workgroups(
                (renderer.options.output_image_dimensions.0 / 8) as u32,
                (renderer.options.output_image_dimensions.1 / 8) as u32,
                1,
            );
        }

        command_encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &state.storage_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &state.output_staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some((renderer.options.output_image_dimensions.0 * 8) as u32),
                    rows_per_image: Some(renderer.options.output_image_dimensions.1 as u32),
                },
            },
            wgpu::Extent3d {
                width: renderer.options.output_image_dimensions.0 as u32,
                height: renderer.options.output_image_dimensions.1 as u32,
                depth_or_array_layers: 1,
            },
        );

        state.renderer_info.curr_sample += 1;

        state.queue.submit(Some(command_encoder.finish()));
    }

    let mut output_data: Vec<u8> = vec![];
    let buffer_slice = state.output_staging_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
    state
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    receiver.recv_async().await.unwrap().unwrap();
    {
        let view = buffer_slice.get_mapped_range();
        output_data.resize(view.len(), 0);
        output_data.copy_from_slice(&view[..]);
    }
    state.output_staging_buffer.unmap();

    return output_data;
}

struct State {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    compute_pipeline: wgpu::ComputePipeline,
    storage_buffers: StorageBuffers,
    uniform_buffers: UniformBuffers,
    storage_texture: wgpu::Texture,
    storage_texture_bind_group: wgpu::BindGroup,
    output_staging_buffer: wgpu::Buffer,
    renderer_info: RendererInfo,
}

impl State {
    pub fn new(renderer: Renderer, scene: &Scene) -> Self {
        let (instance, adapter) = Self::get_instance_and_adapter();
        let (device, queue) = Self::create_device_and_queue(&adapter);

        let storage_buffers = StorageBuffers::new(&device, scene);
        let uniform_buffers = UniformBuffers::new(&device, scene);

        let compute_shader_module =
            device.create_shader_module(wgpu::include_wgsl!("./gpu/rt_compute.wgsl"));

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
            format: wgpu::TextureFormat::Rgba16Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let storage_texture_view =
            storage_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (renderer.options.output_image_dimensions.0
                * 8
                * renderer.options.output_image_dimensions.1) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let storage_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba16Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });
        let storage_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &storage_texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&storage_texture_view),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &storage_texture_bind_group_layout,
                &storage_buffers.bind_group_layout,
                &uniform_buffers.bind_group_layout,
            ],
            immediate_size: 8,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &compute_shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let renderer_info = RendererInfo {
            curr_sample: 1,
            max_ray_depth: renderer.options.max_ray_depth as u32,
        };

        return Self {
            instance,
            adapter,
            device,
            queue,
            compute_pipeline,
            storage_buffers,
            uniform_buffers,
            storage_texture,
            storage_texture_bind_group,
            output_staging_buffer,
            renderer_info,
        };
    }

    fn get_instance_and_adapter() -> (wgpu::Instance, wgpu::Adapter) {
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
        return (instance, adapter);
    }

    fn create_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        let downlevel_capabilities = adapter.get_downlevel_capabilities();
        if !downlevel_capabilities
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
        {
            panic!("Adapter does not support compute shaders");
        }

        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                | wgpu::Features::IMMEDIATES
                | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM,
            required_limits: adapter.limits(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .expect("Failed to create device")
    }
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
    fn new(device: &wgpu::Device, scene: &Scene) -> Self {
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
}

impl UniformBuffers {
    fn new(device: &wgpu::Device, scene: &Scene) -> Self {
        let uniform_camera = UniformCamera {
            look_at: scene.camera.look_at,
            position: scene.camera.position,
            _pad: [0; 4],
        };
        let camera_buffer = Buffer::create_uniform_buffer(device, 0, &[uniform_camera]);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[camera_buffer.bind_group_layout_entry],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[camera_buffer.bind_group_entry()],
        });

        return Self {
            bind_group,
            bind_group_layout,
            camera_buffer,
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

impl From<Camera> for UniformCamera {
    fn from(camera: Camera) -> Self {
        return Self {
            look_at: camera.look_at,
            position: camera.position,
            _pad: [0; 4],
        };
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(4))]
struct RendererInfo {
    curr_sample: u32,
    max_ray_depth: u32,
}
