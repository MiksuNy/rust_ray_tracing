pub mod cpu;
pub mod gpu;

#[derive(Default, Debug)]
pub enum RendererBackend {
    #[default]
    /// Uses the CPU to render a scene.
    CPU,
    /// Uses WGPU as a backend
    WGPU,
}
