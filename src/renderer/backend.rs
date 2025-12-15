pub mod cpu;

#[derive(Default)]
pub enum RendererBackend {
    #[default]
    CPU,
    GPU,
}
