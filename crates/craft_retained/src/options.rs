use craft_primitives::geometry::Size;
use craft_renderer::RendererType;

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
    pub window_title: String,
    /// The initial size of the window.
    pub window_size: Option<Size<f32>>,
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

impl CraftOptions {
    pub fn basic(title: &str) -> Self {
        Self {
            renderer: RendererType::default(),
            window_title: title.to_string(),
            window_size: None,
        }
    }
}
