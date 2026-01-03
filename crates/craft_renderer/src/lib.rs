pub mod color;

#[allow(clippy::module_inception)]
pub mod renderer;

#[cfg(feature = "vello_renderer")]
pub mod vello;

#[cfg(feature = "vello_cpu_renderer")]
pub mod vello_cpu;

pub mod blank_renderer;
mod image_adapter;
mod renderer_type;
pub mod text_renderer_data;
pub(crate) mod tinyvg_helpers;
#[cfg(feature = "vello_hybrid_renderer")]
pub mod vello_hybrid;

pub use renderer::{Brush, RenderCommand, RenderList};
pub use renderer_type::RendererType;
