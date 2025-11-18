use crate::image::{Image, PPM};
use crate::ray::Ray;
use crate::scene::Scene;
use crate::vector::Vec3f;
use rayon::prelude::*;

mod bvh;
mod image;
mod loader;
mod log;
mod ray;
mod scene;
mod texture;
mod vector;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const ASPECT: f32 = WIDTH as f32 / HEIGHT as f32;
const SAMPLE_COUNT: usize = 10;
const MAX_BOUNCES: usize = 6;
const DEBUG_BVH: bool = false;
const IMAGE_PATH: &str = "output.ppm";
const OBJ_PATH: &str = "../res/pbrt_dragon.obj";

fn main() {
    log_info!("Parameters");
    log_info!("- Width:        {}", WIDTH);
    log_info!("- Height:       {}", HEIGHT);
    log_info!("- Sample count: {}", SAMPLE_COUNT);
    log_info!("- Max bounces:  {}", MAX_BOUNCES);
    log_info!("- BVH debug:    {}", DEBUG_BVH);
    log_info!("- Input file:   {}", OBJ_PATH);
    log_info!("- Output file:  {}\n", IMAGE_PATH);

    let mut image: PPM = Image::new(WIDTH, HEIGHT);
    let Some(scene) = Scene::load(OBJ_PATH) else {
        return;
    };

    let start_time = std::time::Instant::now();

    let pixel_data = (0..WIDTH * HEIGHT).into_par_iter().map(|index: usize| {
        let mut rng_state: u32 =
            987612486u32.wrapping_mul((index as u32).wrapping_add(87636354u32));
        let mut final_color = Vec3f::new(0.0, 0.0, 0.0);
        let x: usize = index % WIDTH;
        let y: usize = HEIGHT - (index / WIDTH);
        let screen_x = (((x as f32 / WIDTH as f32) * 2.0) - 1.0) * ASPECT;
        let screen_y = ((y as f32 / HEIGHT as f32) * 2.0) - 1.0;

        for _ in 0..SAMPLE_COUNT {
            let mut ray = Ray::new(
                // Hard coded camera position
                Vec3f::new(0.0, 0.0, 7.0),
                Vec3f::new(
                    screen_x + (Vec3f::rand_f32(&mut rng_state) * 2.0 - 1.0) * 0.0005,
                    screen_y + (Vec3f::rand_f32(&mut rng_state) * 2.0 - 1.0) * 0.0005,
                    -2.0,
                )
                .normalized(),
            );

            final_color += Ray::trace(&mut ray, MAX_BOUNCES, &scene, &mut rng_state, DEBUG_BVH);

            // Only one sample is needed for BVH visualization
            if DEBUG_BVH {
                break;
            }
        }

        // TODO: Reimplement progress bar

        if !DEBUG_BVH {
            final_color /= SAMPLE_COUNT as f32;
        }
        final_color = Vec3f::linear_to_gamma(final_color);

        return final_color.into();
    });

    pixel_data.collect_into_vec(&mut image.pixel_data);

    log_info!("Rendering took {} ms", start_time.elapsed().as_millis());

    image.write_to_path(IMAGE_PATH);
}

mod utility {
    pub fn progress_bar(name: &str, fill_amount: f32, bar_size: usize) {
        let symbol_count: usize = (bar_size as f32 * fill_amount).floor() as usize;
        let percentage: usize = (100.0 * fill_amount).floor() as usize;

        eprint!("\x1B[2K{}: {}% [", name, percentage);
        for i in 0..bar_size {
            if i < symbol_count {
                eprint!("#");
            } else {
                eprint!(" ");
            }
        }
        eprint!("]\r");
    }
}
