use crate::{log_error, log_info, scene::Scene};
use backend::RendererBackend;

pub mod backend;

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
        if options.samples == 0 {
            log_error!("Sample count must be greater than 0");
            return None;
        }

        log_info!("Renderer info");
        log_info!(
            "- Output image dimensions: {}x{}",
            options.output_image_dimensions.0,
            options.output_image_dimensions.1
        );
        log_info!("- Sample count:            {}", options.samples);
        log_info!("- Max bounces:             {}", options.max_ray_depth);
        log_info!("- Backend:                 {:?}\n", options.backend);

        let renderer = Self { options };

        return Some(renderer);
    }

    pub fn render_scene_to_path(&self, scene: &Scene, path: &str) {
        log_info!("Rendering scene with {:?} backend", self.options.backend);

        let start_time = std::time::Instant::now();
        let bytes = match self.options.backend {
            RendererBackend::CPU => backend::cpu::render_scene(self, scene),
            RendererBackend::WGPU => pollster::block_on(backend::wgpu::render_scene(self, scene)),
        };
        // FIXME: This is not accurate at all, especially with the wgpu backend
        log_info!("Rendering took {} ms", start_time.elapsed().as_millis());

        let width = self.options.output_image_dimensions.0;
        let height = self.options.output_image_dimensions.1;

        let image_result = image::save_buffer(
            path,
            bytes.as_slice(),
            width as u32,
            height as u32,
            image::ColorType::Rgba8,
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

impl Default for Renderer {
    fn default() -> Self {
        log_info!("Using default renderer");
        return Self {
            options: RendererOptions::default(),
        };
    }
}

pub struct RendererOptions {
    pub samples: usize,
    pub max_ray_depth: usize,
    pub output_image_dimensions: (usize, usize),
    pub backend: RendererBackend,
}

impl Default for RendererOptions {
    fn default() -> Self {
        return Self {
            samples: 1,
            max_ray_depth: 6,
            output_image_dimensions: (1920, 1080),
            backend: RendererBackend::default(),
        };
    }
}
