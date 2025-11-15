use std::fmt::{Display, Formatter};
use std::sync::Arc;
use winit::window::Window;
use crate::blank_renderer::BlankRenderer;
use crate::renderer::Renderer;
#[cfg(feature = "vello_renderer")]
use crate::vello::VelloRenderer;

#[cfg(feature = "vello_cpu_renderer")]
use crate::vello_cpu::VelloCpuRenderer;

#[cfg(feature = "vello_hybrid_renderer")]
use crate::vello_hybrid::VelloHybridRenderer;

/// An enumeration of the available renderer types for Craft.
///
/// Depending on compile-time features, different renderers can be enabled.
/// When the `vello_renderer` feature is enabled, the [`Vello`](RendererType::Vello)
/// variant is available; otherwise, the [`Blank`](RendererType::Blank) variant is used.
#[derive(Copy, Clone, Debug)]
pub enum RendererType {
    #[cfg(feature = "vello_renderer")]
    Vello,
    #[cfg(feature = "vello_cpu_renderer")]
    VelloCPU,
    #[cfg(feature = "vello_hybrid_renderer")]
    VelloHybrid,
    Blank,
}

#[allow(clippy::derivable_impls)]
impl Default for RendererType {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "vello_renderer")] {
                RendererType::Vello
            } else if #[cfg(feature = "vello_hybrid_renderer")]{
                RendererType::VelloHybrid
            } else if #[cfg(feature = "vello_cpu_renderer")]{
                RendererType::VelloCPU
            } else {
                RendererType::Blank
            }
        }
    }
}

impl Display for RendererType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "vello_renderer")]
            RendererType::Vello => write!(f, "vello/wgpu"),
            #[cfg(feature = "vello_cpu_renderer")]
            RendererType::VelloCPU => write!(f, "vello/cpu"),
            #[cfg(feature = "vello_hybrid_renderer")]
            RendererType::VelloHybrid => write!(f, "vello/hybrid"),
            RendererType::Blank => write!(f, "blank"),
        }
    }
}

impl RendererType {

    pub async fn create(&self, window: Arc<Window>) -> Box<dyn Renderer> {
        let renderer: Box<dyn Renderer> = match self {
            #[cfg(feature = "vello_renderer")]
            RendererType::Vello => Box::new(VelloRenderer::new(window, false).await),
            #[cfg(feature = "vello_cpu_renderer")]
            RendererType::VelloCPU => Box::new(VelloCpuRenderer::new(window)),
            #[cfg(feature = "vello_hybrid_renderer")]
            RendererType::VelloHybrid => Box::new(VelloHybridRenderer::new(window).await),
            RendererType::Blank => {
                // So the linter does not complain about window being unused.
                let _ = window;
                Box::new(BlankRenderer)
            },
        };

        renderer
    }
}