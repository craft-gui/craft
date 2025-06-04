pub mod color;

#[allow(clippy::module_inception)]
pub(crate) mod renderer;

#[cfg(feature = "vello_renderer")]
pub(crate) mod vello;

#[cfg(feature = "vello_cpu_renderer")]
pub(crate) mod vello_cpu;

pub(crate) mod blank_renderer;
mod image_adapter;
pub(crate) mod tinyvg_helpers;
#[cfg(feature = "vello_hybrid_renderer")]
pub(crate) mod vello_hybrid;

pub use renderer::Brush;
pub use renderer::RenderCommand;
pub use renderer::RenderList;
