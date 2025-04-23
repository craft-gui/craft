pub mod color;

#[allow(clippy::module_inception)]
pub(crate) mod renderer;

#[cfg(feature = "vello_renderer")]
pub(crate) mod vello;

#[cfg(feature = "vello_cpu_renderer")]
pub(crate) mod vello_cpu;

#[cfg(feature = "vello_hybrid_renderer")]
pub(crate) mod vello_hybrid;
pub(crate) mod blank_renderer;
mod image_adapter;
pub(crate) mod text;

pub use renderer::RenderCommand;
