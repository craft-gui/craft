use std::fmt::{Display, Formatter};
use crate::geometry::Size;

/// Configuration options for the Craft application.
///
/// This struct holds various options that can be used to customize the behavior
/// of the application. In particular, it configures which renderer to use and
/// sets the default window title.
pub struct CraftOptions {
    /// The type of renderer to use.
    ///
    /// The renderer is chosen based on the features enabled at compile time.
    /// See [`RendererType`] for details.
    pub renderer: RendererType,
    /// The title of the application window.
    ///
    /// Defaults to `"craft"`.
    pub window_title: String,
    /// The initial size of the window.
    pub window_size: Option<Size<f32>>
}

impl Default for CraftOptions {
    fn default() -> Self {
        Self {
            renderer: RendererType::default(),
            window_title: "craft".to_string(),
            window_size: None,
        }
    }
}

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

impl CraftOptions {
    
    pub fn basic(title: &str) -> Self {
        Self {
            renderer: RendererType::default(),
            window_title: title.to_string(),
            window_size: None,
        }
    }
    
}
