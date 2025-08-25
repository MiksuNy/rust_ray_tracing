use crate::ray::Ray;
use crate::scene::Scene;
use crate::vec3::Vec3;
use std::io::Write;

mod bvh;
mod ray;
mod scene;
mod vec3;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const ASPECT: f32 = WIDTH as f32 / HEIGHT as f32;
const SAMPLE_COUNT: usize = 1;
const MAX_BOUNCES: usize = 4;
const DEBUG_BVH: bool = false;

fn main() {
    // Initialize the prng to some big value
    let mut rng_state: u32 = 987612486;

    let mut output_buffer: Vec<u8> = Vec::new();
    output_buffer.reserve_exact(((WIDTH + WIDTH) * HEIGHT) as usize);
    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("output.ppm")
        .unwrap();
    let _ = output_file.write_fmt(format_args!("P3\n{} {}\n255\n", WIDTH, HEIGHT));

    let model = Scene::load_from_obj("../res/cornell_box.obj");

    let start_time = std::time::Instant::now();

    for y in (0..HEIGHT).rev() {
        for x in 0..WIDTH {
            let mut final_color = Vec3::new(0.0, 0.0, 0.0);

            let screen_x = (((x as f32 / WIDTH as f32) * 2.0) - 1.0) * ASPECT;
            let screen_y = ((y as f32 / HEIGHT as f32) * 2.0) - 1.0;

            for _ in 0..SAMPLE_COUNT {
                let mut ray = Ray::new(
                    Vec3::new(0.0, 0.0, 2.0),
                    Vec3::new(
                        screen_x + Vec3::rand_f32(&mut rng_state) * 0.0005,
                        screen_y + Vec3::rand_f32(&mut rng_state) * 0.0005,
                        -1.0,
                    )
                    .normalized(),
                );

                final_color = Vec3::add(
                    final_color,
                    Ray::trace(&mut ray, MAX_BOUNCES, &model, &mut rng_state, DEBUG_BVH),
                );

                // Only one sample is needed for BVH visualization
                if DEBUG_BVH {
                    break;
                }
            }

            final_color = Vec3::div(final_color, Vec3::from(SAMPLE_COUNT as f32));
            final_color = Vec3::linear_to_gamma(final_color);

            for c in final_color.to_color() {
                let _ = output_buffer.write(c.to_string().as_str().as_bytes());
                let _ = output_buffer.write(b" ");
            }
        }
        let _ = output_buffer.write(b"\n");

        println!("Lines remaining: {}", y);
    }

    let _ = output_file.write(output_buffer.as_slice());

    let end_time = std::time::Instant::now();

    println!("Rendering took {} ms", (end_time - start_time).as_millis());
}
