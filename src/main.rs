use crate::image::{Image, PPM};
use crate::ray::Ray;
use crate::scene::Scene;
use crate::vec3::Vec3;

mod bvh;
mod image;
mod ray;
mod scene;
mod vec3;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const ASPECT: f32 = WIDTH as f32 / HEIGHT as f32;
const SAMPLE_COUNT: usize = 10;
const MAX_BOUNCES: usize = 8;
const DEBUG_BVH: bool = false;
const IMAGE_PATH: &str = "output.ppm";
const OBJ_PATH: &str = "../res/pbrt_dragon.obj";

fn main() {
    // Initialize the prng to some big value
    let mut rng_state: u32 = 987612486;

    let mut image: PPM = Image::new(WIDTH, HEIGHT);
    let scene = Scene::load_from_obj(OBJ_PATH);

    let start_time = std::time::Instant::now();

    for y in (0..HEIGHT).rev() {
        for x in 0..WIDTH {
            let mut final_color = Vec3::new(0.0, 0.0, 0.0);

            let screen_x = (((x as f32 / WIDTH as f32) * 2.0) - 1.0) * ASPECT;
            let screen_y = ((y as f32 / HEIGHT as f32) * 2.0) - 1.0;

            for _ in 0..SAMPLE_COUNT {
                let mut ray = Ray::new(
                    // Hard coded camera position
                    Vec3::new(0.0, 0.0, 4.5),
                    Vec3::new(
                        screen_x + (Vec3::rand_f32(&mut rng_state) * 2.0 - 1.0) * 0.0005,
                        screen_y + (Vec3::rand_f32(&mut rng_state) * 2.0 - 1.0) * 0.0005,
                        -1.0,
                    )
                    .normalized(),
                );

                final_color = Vec3::add(
                    final_color,
                    Ray::trace(&mut ray, MAX_BOUNCES, &scene, &mut rng_state, DEBUG_BVH),
                );

                // Only one sample is needed for BVH visualization
                if DEBUG_BVH {
                    break;
                }
            }

            if !DEBUG_BVH {
                final_color = Vec3::div(final_color, Vec3::from(SAMPLE_COUNT as f32));
            }
            final_color = Vec3::linear_to_gamma(final_color);

            image.pixel_data.push(final_color.into());
        }

        println!("Lines remaining: {}", y);
    }

    println!("Rendering took {} ms", start_time.elapsed().as_millis());

    image.write_to_path(IMAGE_PATH);
}
