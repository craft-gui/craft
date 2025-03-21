use std::fmt::{Display, Formatter};

/// Configuration options for the Oku application.
///
/// This struct holds various options that can be used to customize the behavior
/// of the application. In particular, it configures which renderer to use and
/// sets the default window title.
pub struct OkuOptions {
    /// The type of renderer to use.
    ///
    /// The renderer is chosen based on the features enabled at compile time.
    /// See [`RendererType`] for details.
    pub renderer: RendererType,
    /// The title of the application window.
    ///
    /// Defaults to `"oku"`.
    pub window_title: String,
}

impl Default for OkuOptions {
    fn default() -> Self {
        Self {
            renderer: RendererType::default(),
            window_title: "oku".to_string(),
        }
    }
}

/// An enumeration of the available renderer types for Oku.
///
/// Depending on compile-time features, different renderers can be enabled.
/// When the `vello_renderer` feature is enabled, the [`Vello`](RendererType::Vello)
/// variant is available; otherwise, the [`Blank`](RendererType::Blank) variant is used.
#[derive(Copy, Clone, Debug)]
pub enum RendererType {
    #[cfg(feature = "vello_renderer")]
    Vello,
    #[cfg(feature = "wgpu_renderer")]
    Wgpu,
    Blank,
}

#[allow(clippy::derivable_impls)]
impl Default for RendererType {
    fn default() -> Self {
        cfg_if::cfg_if! {
          if #[cfg(feature = "vello_renderer")] {
                RendererType::Vello
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
            #[cfg(feature = "wgpu_renderer")]
            RendererType::Wgpu => write!(f, "wgpu"),
            RendererType::Blank => write!(f, "blank"),
        }
    }
}
