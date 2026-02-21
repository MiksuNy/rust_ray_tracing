use std::cell::RefCell;
use std::rc::Rc;

use crate::renderer::backend::RendererBackend;
use crate::renderer::*;
use crate::scene::{Camera, Scene};
use crate::vector::Vec3f;

mod bvh;
mod loader;
mod log;
mod renderer;
mod scene;
mod texture;
mod vector;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const MAX_SAMPLE_COUNT: usize = 1;
const MAX_BOUNCES: usize = 6;
const OBJ_PATH: &str = "../res/main_sponza/main_sponza_with_curtains.obj";
const IMAGE_PATH: &str = "output.png";

fn main() {
    let Some(renderer) = Renderer::new(RendererOptions {
        max_samples: MAX_SAMPLE_COUNT,
        max_ray_depth: MAX_BOUNCES,
        output_image_dimensions: (WIDTH, HEIGHT),
        output_image_path: None,
        backend: RendererBackend::GPU,
        is_realtime: true,
    }) else {
        return;
    };

    let Some(mut scene) = Scene::load(OBJ_PATH) else {
        return;
    };

    let mut camera = Camera::default();
    camera.position = Vec3f::new(0.0, 0.0, 5.0);
    camera.pitch = 0.0;
    camera.yaw = 90.0;
    scene.set_camera(camera);

    renderer.render(Rc::new(RefCell::new(scene)));
}
