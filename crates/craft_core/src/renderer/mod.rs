pub mod color;

#[allow(clippy::module_inception)]
pub(crate) mod renderer;

#[cfg(feature = "wgpu_renderer")]
pub mod wgpu;

#[cfg(feature = "vello_renderer")]
pub(crate) mod vello;

#[cfg(feature = "vello_cpu_renderer")]
pub(crate) mod vello_cpu;

#[cfg(feature = "vello_hybrid_renderer")]
pub(crate) mod vello_hybrid;
pub(crate) mod blank_renderer;
mod image_adapter;

pub use renderer::RenderCommand;
