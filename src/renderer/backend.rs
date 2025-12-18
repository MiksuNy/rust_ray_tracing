pub mod cpu;
pub mod wgpu;

#[derive(Default, Debug)]
pub enum RendererBackend {
    #[default]
    CPU,
    WGPU,
}
