use std::cell::RefCell;
use std::rc::Rc;

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

const OBJ_PATH: &str = "../res/cyberpunk/cyberpunk.obj";
const IMAGE_PATH: &str = "output.png";

fn main() {
    let Some(renderer) = Renderer::new(RendererOptions {
        samples: 2000,
        max_ray_depth: 12,
        output_image_dimensions: (1920, 1080),
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
    camera.position = glam::Vec3::new(0.45856124, 0.42353606, -8.8515625);
    camera.pitch = -0.00004978478;
    camera.yaw = 289.9035;
    scene.set_camera(camera);

    renderer.render(Rc::new(RefCell::new(scene)));
}
