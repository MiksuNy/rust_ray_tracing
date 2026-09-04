use crate::log_info;
use crate::math::rand_f32;
use crate::renderer::Renderer;
use crate::scene::Scene;
use glam::Vec4Swizzles;
use ray::Ray;
use rayon::prelude::*;

mod ray;

// TODO: A simple progress indicator for rendering would be nice
pub fn render_scene(renderer: Renderer, scene: &Scene) -> Vec<u8> {
    log_info!(
        "Using {} threads for rendering",
        rayon::current_num_threads()
    );

    let width = renderer.options.output_image_dimensions.0;
    let height = renderer.options.output_image_dimensions.1;

    let block_size = (width * height) / rayon::current_num_threads();

    (0..width * height)
        .into_par_iter()
        .by_uniform_blocks(block_size)
        .map(|index: usize| {
            let mut rng_state: u32 =
                987612486u32.wrapping_mul((index as u32).wrapping_add(87636354u32));
            let mut final_color = glam::Vec3::new(0.0, 0.0, 0.0);
            let x: usize = index % width;
            let y: usize = height - (index / width);
            let screen_x =
                (((x as f32 / width as f32) * 2.0) - 1.0) * (width as f32 / height as f32);
            let screen_y = ((y as f32 / height as f32) * 2.0) - 1.0;

            for _ in 0..renderer.options.samples {
                let jitter = glam::Vec3::new(
                    rand_f32(&mut rng_state) * 2.0 - 1.0,
                    rand_f32(&mut rng_state) * 2.0 - 1.0,
                    0.0,
                ) * 0.0005;
                let direction = (scene.camera.look_at
                    * glam::Vec4::new(-screen_x + jitter.x, screen_y + jitter.y, 1.0, 1.0))
                .normalize();
                let mut ray = Ray::new(
                    // Hard coded camera position
                    scene.camera.position,
                    direction.xyz(),
                );

                final_color += Ray::trace(
                    &mut ray,
                    renderer.options.max_ray_depth,
                    &scene,
                    &mut rng_state,
                );
            }

            final_color /= renderer.options.samples as f32;
            final_color = crate::math::aces_filmic(final_color);
            final_color = crate::math::linear_to_srgb(final_color);

            let bytes = crate::math::color_to_bytes(final_color);
            return [bytes[0], bytes[1], bytes[2], 255];
        })
        .collect::<Vec<[u8; 4]>>()
        .into_flattened()
}
