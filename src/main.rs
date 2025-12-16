use renderer::backend::RendererBackend;

use crate::renderer::Renderer;
use crate::scene::Scene;
use crate::vector::Vec3f;

mod bvh;
mod loader;
mod log;
mod ray;
mod renderer;
mod scene;
mod texture;
mod vector;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const SAMPLE_COUNT: usize = 1;
const MAX_BOUNCES: usize = 6;
const DEBUG_BVH: bool = false;
const OBJ_PATH: &str = "../res/dragon/dragon.obj";
const IMAGE_PATH: &str = "output.png";

fn main() {
    let Some(renderer) = Renderer::new(
        SAMPLE_COUNT,
        MAX_BOUNCES,
        DEBUG_BVH,
        (WIDTH, HEIGHT),
        RendererBackend::GPU,
    ) else {
        return;
    };

    let Some(scene) = Scene::load(OBJ_PATH) else {
        return;
    };

    renderer.render_scene_to_path(&scene, IMAGE_PATH);
}
