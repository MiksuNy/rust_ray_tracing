use std::{cell::RefCell, collections::HashSet, rc::Rc, sync::Arc};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    log_info,
    renderer::{
        Renderer,
        backend::gpu::{State, UniformCamera},
    },
    scene::Scene,
};

struct AppState {
    state: Option<State>,
    window: Arc<Window>,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    texture: wgpu::Texture,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    key_states: HashSet<PhysicalKey>,
}

impl AppState {
    async fn new(
        _display: OwnedDisplayHandle,
        window: Arc<Window>,
        renderer: Renderer,
        scene: &Scene,
    ) -> Self {
        let state = State::new(renderer, scene);
        let _ = window.request_inner_size(PhysicalSize::new(
            renderer.options.output_image_dimensions.0 as u32,
            renderer.options.output_image_dimensions.1 as u32,
        ));
        let size = window.inner_size();

        let surface = state.instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&state.adapter);
        let surface_format = cap.formats[0];

        let shader_module = state
            .device
            .create_shader_module(wgpu::include_wgsl!("./shader.wgsl"));

        let bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                });

        let texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: state.storage_texture.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        let render_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout],
                    immediate_size: 0,
                });
        let render_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(surface_format.into())],
                    }),
                    multiview_mask: None,
                    cache: None,
                });

        let app_state = Self {
            state: Some(state),
            window,
            size,
            surface,
            surface_format,
            texture,
            render_pipeline,
            bind_group,
            key_states: HashSet::new(),
        };

        app_state.configure_surface();

        return app_state;
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: self.surface_format,
            view_formats: vec![],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 1,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface
            .configure(&self.state.as_ref().unwrap().device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
    }

    fn render(&mut self) {
        let state = self.state.as_ref().unwrap();

        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let surface_texture_view =
            surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: None,
                    ..Default::default()
                });

        let mut command_encoder = self
            .state
            .as_ref()
            .unwrap()
            .device
            .create_command_encoder(&Default::default());

        // Ray tracing compute pass
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
            compute_pass.dispatch_workgroups(
                state.storage_texture.width() / 8,
                state.storage_texture.height() / 8,
                1,
            );
        }

        // Copy the ray traced image to the output texture
        command_encoder.copy_texture_to_texture(
            state.storage_texture.as_image_copy(),
            self.texture.as_image_copy(),
            state.storage_texture.size(),
        );

        // Finally render the copied texture to the screen
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.state
            .as_ref()
            .unwrap()
            .queue
            .submit([command_encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}

#[derive(Default)]
struct App {
    app_state: Option<AppState>,
    renderer: Renderer,
    scene: Rc<RefCell<Scene>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let app_state = pollster::block_on(AppState::new(
            event_loop.owned_display_handle(),
            window.clone(),
            self.renderer,
            &self.scene.clone().borrow(),
        ));

        self.app_state = Some(app_state);

        window.set_cursor_visible(false);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let app_state = self.app_state.as_mut().unwrap();
        let state = app_state.state.as_ref().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                log_info!("Closing window");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let scene = self.scene.clone();

                // Move cursor to the center of window if it's in focus
                let window = app_state.window.clone();
                if window.has_focus() {
                    let window_size = window.inner_size();
                    let _ = window.set_cursor_position(PhysicalPosition::new(
                        window_size.width / 2,
                        window_size.height / 2,
                    ));
                }

                let camera = &mut scene.borrow_mut().camera;

                for physical_key in app_state.key_states.iter() {
                    match physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            camera.position -= camera.forward * 0.05
                        }
                        PhysicalKey::Code(KeyCode::KeyS) => {
                            camera.position += camera.forward * 0.05
                        }
                        PhysicalKey::Code(KeyCode::KeyA) => camera.position -= camera.right * 0.05,
                        PhysicalKey::Code(KeyCode::KeyD) => camera.position += camera.right * 0.05,
                        PhysicalKey::Code(KeyCode::Space) => camera.position += camera.up * 0.05,
                        PhysicalKey::Code(KeyCode::KeyZ) => camera.position -= camera.up * 0.05,
                        _ => (),
                    }
                }

                // Update camera matrix
                camera.update_view();

                // Upload new camera data to the GPU
                let new_camera_data = UniformCamera::from(camera.clone());
                state
                    .uniform_buffers
                    .camera_buffer
                    .set_buffer_data(&state.queue, &[new_camera_data]);

                self.app_state.as_mut().unwrap().render();
                self.app_state
                    .as_ref()
                    .unwrap()
                    .get_window()
                    .request_redraw();
            }
            WindowEvent::Resized(size) => {
                self.app_state.as_mut().unwrap().resize(size);
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let scene = self.scene.clone();
        let key_states = &mut self.app_state.as_mut().unwrap().key_states;
        let camera = &mut scene.borrow_mut().camera;

        match event {
            DeviceEvent::MouseMotion { delta } => {
                camera.yaw += delta.0 as f32 * 0.1;
                camera.pitch += delta.1 as f32 * 0.1;
                if camera.pitch >= 89.0 {
                    camera.pitch = 89.0;
                } else if camera.pitch <= -89.0 {
                    camera.pitch = -89.0;
                }
            }
            DeviceEvent::Key(key_event) => {
                match key_event.state {
                    ElementState::Pressed => key_states.insert(key_event.physical_key),
                    ElementState::Released => key_states.remove(&key_event.physical_key),
                };
            }
            _ => (),
        }
    }
}

pub fn render_scene_to_window(renderer: Renderer, scene: Rc<RefCell<Scene>>) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    app.renderer = renderer;
    app.scene = scene;
    event_loop.run_app(&mut app).unwrap();
}
