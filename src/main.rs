use crate::image::{Image, ImageFormat};
use crate::renderer::{Parameters, Renderer};
use crate::scene::Scene;
use crate::vector::Vec3f;

mod bvh;
mod image;
mod loader;
mod log;
mod ray;
mod renderer;
mod scene;
mod texture;
mod vector;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const SAMPLE_COUNT: usize = 1000;
const MAX_BOUNCES: usize = 2;
const DEBUG_BVH: bool = false;
const IMAGE_PATH: &str = "output.ppm";
const OBJ_PATH: &str = "../res/balls_metallic.obj";

fn main() {
    log_info!("System logical cores: {}\n", rayon::current_num_threads());

    log_info!("Parameters");
    log_info!("- Width:        {}", WIDTH);
    log_info!("- Height:       {}", HEIGHT);
    log_info!("- Sample count: {}", SAMPLE_COUNT);
    log_info!("- Max bounces:  {}", MAX_BOUNCES);
    log_info!("- BVH debug:    {}", DEBUG_BVH);
    log_info!("- Input file:   {}", OBJ_PATH);
    log_info!("- Output file:  {}\n", IMAGE_PATH);

    let renderer = Renderer::new(Parameters {
        samples: SAMPLE_COUNT,
        max_ray_depth: MAX_BOUNCES,
        debug_mode: DEBUG_BVH,
    });
    let mut image = Image::new(ImageFormat::PPM, WIDTH, HEIGHT);
    let Some(scene) = Scene::load(OBJ_PATH) else {
        return;
    };

    let start_time = std::time::Instant::now();
    renderer.render_to_image(&scene, &mut image);
    log_info!("Rendering took {} ms", start_time.elapsed().as_millis());

    image.write_to_path(IMAGE_PATH);
}
