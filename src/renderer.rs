use std::{cell::RefCell, rc::Rc};

use crate::{log_error, log_info, scene::Scene};
use backend::RendererBackend;

pub mod backend;

#[derive(Clone, Copy)]
pub struct Renderer {
    pub options: RendererOptions,
}

impl Renderer {
    pub fn new(options: RendererOptions) -> Option<Self> {
        if options.output_image_dimensions.0 == 0 || options.output_image_dimensions.1 == 0 {
            log_error!("Width and height must be greater than 0");
            return None;
        }
        if options.max_ray_depth == 0 {
            log_error!("Max ray depth must be greater than 0");
            return None;
        }
        if options.max_samples == 0 {
            log_error!("Max sample count must be greater than 0");
            return None;
        }
        if options.output_image_path.is_none() && !options.is_realtime {
            log_error!("Output image path must be Some if realtime mode is disabled");
            return None;
        }
        if options.backend != RendererBackend::GPU && options.is_realtime {
            log_error!("Only the GPU backend is supported for realtime mode");
            return None;
        }

        log_info!("Renderer info");
        log_info!(
            "- Output image dimensions: {}x{}",
            options.output_image_dimensions.0,
            options.output_image_dimensions.1
        );
        log_info!("- Max sample count:        {}", options.max_samples);
        log_info!("- Max bounces:             {}", options.max_ray_depth);
        log_info!("- Backend:                 {:?}", options.backend);
        log_info!("- Realtime:                {}\n", options.is_realtime);

        return Some(Self { options });
    }

    pub fn render(self, scene: Rc<RefCell<Scene>>) {
        log_info!("Rendering scene with {:?} backend", self.options.backend);

        if self.options.is_realtime {
            backend::gpu::window::render_scene_to_window(self, scene);
        } else {
            let start_time = std::time::Instant::now();
            let bytes = match self.options.backend {
                RendererBackend::CPU => backend::cpu::render_scene(self, &scene.clone().borrow()),
                RendererBackend::GPU => pollster::block_on(backend::gpu::render_scene_to_buffer(
                    self,
                    &scene.clone().borrow(),
                )),
            };
            log_info!("Rendering took {} ms", start_time.elapsed().as_millis());

            let path = self.options.output_image_path.unwrap();
            let image_result = image::save_buffer(
                path,
                bytes.as_slice(),
                self.options.output_image_dimensions.0 as u32,
                self.options.output_image_dimensions.1 as u32,
                image::ColorType::Rgba16,
            );

            if image_result.is_err() {
                log_error!(
                    "Could not write image data to '{}' with error {:?}",
                    path,
                    image_result.err()
                );
            } else {
                log_info!("Succesfully wrote image data to '{}'", path);
            }
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        log_info!("Using default renderer");
        return Self {
            options: RendererOptions::default(),
        };
    }
}

#[derive(Clone, Copy)]
pub struct RendererOptions {
    pub max_samples: usize,
    pub max_ray_depth: usize,
    pub output_image_dimensions: (usize, usize),
    pub output_image_path: Option<&'static str>,
    pub backend: RendererBackend,
    pub is_realtime: bool,
}

impl Default for RendererOptions {
    fn default() -> Self {
        return Self {
            max_samples: 1,
            max_ray_depth: 6,
            output_image_dimensions: (1920, 1080),
            output_image_path: None,
            backend: RendererBackend::default(),
            is_realtime: true,
        };
    }
}
