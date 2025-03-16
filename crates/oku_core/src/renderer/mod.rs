pub(crate) mod color;

#[allow(clippy::module_inception)]
pub(crate) mod renderer;

#[cfg(feature = "vello_renderer")]
pub(crate) mod vello;

pub(crate) mod blank_renderer;

pub use renderer::RenderCommand;
