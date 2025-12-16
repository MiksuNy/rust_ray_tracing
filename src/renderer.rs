use crate::{log_error, log_info, scene::Scene};
use backend::RendererBackend;

pub mod backend;

pub struct Renderer {
    pub samples: usize,
    pub max_ray_depth: usize,
    pub debug_mode: bool,
    pub output_image_dimensions: (usize, usize),
    pub backend: RendererBackend,
}

impl Renderer {
    pub fn new(
        samples: usize,
        max_ray_depth: usize,
        debug_mode: bool,
        output_image_dimensions: (usize, usize),
        backend: RendererBackend,
    ) -> Option<Self> {
        if output_image_dimensions.0 <= 0 || output_image_dimensions.1 <= 0 {
            log_error!(
                "Tried to create a renderer with invalid image dimensions: {}x{}",
                output_image_dimensions.0,
                output_image_dimensions.1
            );
            return None;
        }
        let renderer = Self {
            samples,
            max_ray_depth,
            debug_mode,
            output_image_dimensions,
            backend,
        };
        renderer.log_info();

        return Some(renderer);
    }

    pub fn render_scene_to_path(&self, scene: &Scene, path: &str) {
        let start_time = std::time::Instant::now();

        let bytes = match self.backend {
            RendererBackend::CPU => backend::cpu::render_scene(self, scene),
            RendererBackend::GPU => todo!(),
        };

        log_info!("Rendering took {} ms", start_time.elapsed().as_millis());

        let width = self.output_image_dimensions.0;
        let height = self.output_image_dimensions.1;

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

    fn log_info(&self) {
        log_info!("Renderer info");
        log_info!(
            "- Output image dimensions: {}x{}",
            self.output_image_dimensions.0,
            self.output_image_dimensions.1
        );
        log_info!("- Sample count:            {}", self.samples);
        log_info!("- Max bounces:             {}", self.max_ray_depth);
        log_info!("- Debug mode:              {}", self.debug_mode);
        log_info!("- Backend:                 {:?}\n", self.backend);
    }
}

impl Default for Renderer {
    fn default() -> Self {
        log_info!("Using default renderer");
        return Self {
            samples: 1,
            max_ray_depth: 6,
            debug_mode: false,
            output_image_dimensions: (1920, 1080),
            backend: RendererBackend::default(),
        };
    }
}
