pub mod cpu;
pub mod gpu;

#[derive(Default, Debug)]
pub enum RendererBackend {
    #[default]
    CPU,
    GPU,
}
