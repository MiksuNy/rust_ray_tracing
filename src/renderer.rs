use crate::log_error;
use crate::{log_info, scene::Scene};
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
}

impl Default for Renderer {
    fn default() -> Self {
        return Self {
            samples: 1,
            max_ray_depth: 6,
            debug_mode: false,
            output_image_dimensions: (1920, 1080),
            backend: RendererBackend::default(),
        };
    }
}
