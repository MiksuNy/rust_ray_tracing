use crate::log_info;
use crate::ray::Ray;
use crate::renderer::Renderer;
use crate::scene::Scene;
use crate::vector::Vec3f;
use rayon::prelude::*;

// TODO: A simple progress indicator for rendering would be nice
pub fn render_scene(renderer: &Renderer, scene: &Scene) -> Vec<u8> {
    log_info!("Rendering scene with CPU");
    log_info!(
        "Using {} threads for rendering",
        rayon::current_num_threads()
    );

    let width = renderer.output_image_dimensions.0;
    let height = renderer.output_image_dimensions.1;

    let block_size = (width * height) / rayon::current_num_threads();

    (0..width * height)
        .into_par_iter()
        .by_uniform_blocks(block_size)
        .map(|index: usize| {
            let mut rng_state: u32 =
                987612486u32.wrapping_mul((index as u32).wrapping_add(87636354u32));
            let mut final_color = Vec3f::new(0.0, 0.0, 0.0);
            let x: usize = index % width;
            let y: usize = height - (index / width);
            let screen_x =
                (((x as f32 / width as f32) * 2.0) - 1.0) * (width as f32 / height as f32);
            let screen_y = ((y as f32 / height as f32) * 2.0) - 1.0;

            for _ in 0..renderer.samples {
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

                final_color += Ray::trace(
                    &mut ray,
                    renderer.max_ray_depth,
                    &scene,
                    &mut rng_state,
                    renderer.debug_mode,
                );

                // Only one sample is needed for BVH visualization
                if renderer.debug_mode {
                    break;
                }
            }

            if !renderer.debug_mode {
                final_color /= renderer.samples as f32;
            }
            final_color = Vec3f::linear_to_gamma(final_color);

            let rgb: [u8; 3] = final_color.into();
            return [rgb[0], rgb[1], rgb[2], 255];
        })
        .collect::<Vec<[u8; 4]>>()
        .into_flattened()
}
