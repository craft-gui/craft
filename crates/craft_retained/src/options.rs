use craft_primitives::geometry::Size;
use craft_renderer::RendererType;

use crate::craftcallback::CraftCallback;

/// Configuration options for the Craft application.
///
/// This struct holds various options that can be used to customize the behavior
/// of the application. In particular, it configures which renderer to use and
/// sets the default window title.
#[derive(Clone)]
pub struct CraftOptions {
    /// The type of renderer to use.
    ///
    /// The renderer is chosen based on the features enabled at compile time.
    /// See [`RendererType`] for details.
    pub renderer: RendererType,
    /// The title of the application window.
    ///
    /// Defaults to `"craft"`.
    pub app_name: String,
    /// The initial size of the window.
    pub window_size: Option<Size<f32>>,
    pub craft_callback: Option<CraftCallback>,
}

impl Default for CraftOptions {
    fn default() -> Self {
        Self {
            renderer: RendererType::default(),
            app_name: "craft".to_string(),
            window_size: None,
            craft_callback: None,
        }
    }
}

impl CraftOptions {
    pub fn basic(app_name: &str) -> Self {
        Self {
            renderer: RendererType::default(),
            app_name: app_name.to_string(),
            window_size: None,
            craft_callback: None,
        }
    }

    pub fn test(title: &str, callback: CraftCallback) -> Self {
        Self {
            #[cfg(feature = "vello_cpu_renderer")]
            renderer: RendererType::VelloCPU,
            #[cfg(not(feature = "vello_cpu_renderer"))]
            renderer: RendererType::default(),
            app_name: title.to_string(),
            window_size: None,
            craft_callback: Some(callback),
        }
    }
}
