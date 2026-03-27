pub mod cpu;
pub mod gpu;

#[allow(dead_code)]
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum RendererBackend {
    #[default]
    GPU,
    CPU,
}
