use crate::retguicallback::RetGuiCallback;

/// Configuration options for the RetGui application.
///
/// This struct holds various options that can be used to customize the behavior
/// of the application. In particular, it configures which renderer to use and
/// sets the default window title.
#[derive(Clone)]
pub struct RetGuiOptions {
    /// The title of the application window.
    ///
    /// Defaults to `"retgui"`.
    pub app_name: String,
    pub retgui_callback: Option<RetGuiCallback>,
}

impl Default for RetGuiOptions {
    fn default() -> Self {
        Self {
            app_name: "retgui".to_string(),
            retgui_callback: None,
        }
    }
}

impl RetGuiOptions {
    pub fn basic(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            retgui_callback: None,
        }
    }

    pub fn test(title: &str, callback: RetGuiCallback) -> Self {
        Self {
            app_name: title.to_string(),
            retgui_callback: Some(callback),
        }
    }
}
