pub mod cpu;
pub mod gpu;

#[allow(dead_code)]
#[derive(Default, Debug)]
pub enum RendererBackend {
    CPU,
    #[default]
    GPU,
}
