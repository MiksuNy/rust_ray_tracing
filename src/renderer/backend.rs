pub mod cpu;

#[derive(Default, Debug)]
pub enum RendererBackend {
    #[default]
    CPU,
    GPU,
}
