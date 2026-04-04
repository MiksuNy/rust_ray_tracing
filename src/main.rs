use std::cell::RefCell;
use std::rc::Rc;

use crate::math::vec3::Vec3f;
use crate::renderer::backend::RendererBackend;
use crate::renderer::*;
use crate::scene::{Camera, Scene};

mod bvh;
mod loader;
mod log;
mod math;
mod renderer;
mod scene;
mod texture;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const SAMPLE_COUNT: usize = 10000;
const MAX_BOUNCES: usize = 64;
const OBJ_PATH: &str = "../res/many_dragons/many_dragons.obj";
const IMAGE_PATH: &str = "output.png";

fn main() {
    let Some(renderer) = Renderer::new(RendererOptions {
        samples: SAMPLE_COUNT,
        max_ray_depth: MAX_BOUNCES,
        output_image_dimensions: (WIDTH, HEIGHT),
        output_image_path: Some(IMAGE_PATH),
        backend: RendererBackend::GPU,
        is_realtime: true,
    }) else {
        return;
    };

    let Some(mut scene) = Scene::load(OBJ_PATH) else {
        return;
    };

    let mut camera = Camera::default();
    camera.position = Vec3f::new(0.0, 0.0, 1.9);
    camera.pitch = 0.0;
    camera.yaw = 90.0;
    scene.set_camera(camera);

    renderer.render(Rc::new(RefCell::new(scene)));
}
