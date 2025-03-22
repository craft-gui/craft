pub mod color;

#[allow(clippy::module_inception)]
pub(crate) mod renderer;

#[cfg(feature = "wgpu_renderer")]
pub mod wgpu;

#[cfg(feature = "vello_renderer")]
pub(crate) mod vello;

pub(crate) mod blank_renderer;
mod text;

pub use renderer::RenderCommand;
