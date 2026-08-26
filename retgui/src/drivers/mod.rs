pub mod headless;
pub mod winit;

/// Responsible for passing events to [`App`](crate::App) and driving the app forward.
///
/// Custom drivers can be installed with [`retgui_main_with_driver`](crate::retgui_main_with_driver).
pub trait Driver {
    /// Creates the driver and transfers ownership of the initialized app to it.
    fn new(app: crate::App) -> Self;

    /// Runs the app until completion. Usually this is when all windows are closed.
    fn run(self);
}
