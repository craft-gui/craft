/// Configuration options for the RetGui application.
///
/// This struct holds options that customize the behavior of the application.
#[derive(Clone)]
pub struct RetGuiOptions {
    /// The title of the application window.
    ///
    /// Defaults to `"retgui"`.
    pub app_name: String,
}

impl Default for RetGuiOptions {
    fn default() -> Self {
        Self {
            app_name: "retgui".to_string(),
        }
    }
}

impl RetGuiOptions {
    pub fn basic(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
        }
    }
}
