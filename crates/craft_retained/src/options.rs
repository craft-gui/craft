use crate::craftcallback::CraftCallback;

/// Configuration options for the Craft application.
///
/// This struct holds various options that can be used to customize the behavior
/// of the application. In particular, it configures which renderer to use and
/// sets the default window title.
#[derive(Clone)]
pub struct CraftOptions {
    /// The title of the application window.
    ///
    /// Defaults to `"craft"`.
    pub app_name: String,
    pub craft_callback: Option<CraftCallback>,
}

impl Default for CraftOptions {
    fn default() -> Self {
        Self {
            app_name: "craft".to_string(),
            craft_callback: None,
        }
    }
}

impl CraftOptions {
    pub fn basic(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            craft_callback: None,
        }
    }

    pub fn test(title: &str, callback: CraftCallback) -> Self {
        Self {
            app_name: title.to_string(),
            craft_callback: Some(callback),
        }
    }
}
