pub mod headless;
pub mod winit;

#[derive(Clone, Copy, Default)]
pub enum DriverKind {
    #[default]
    Winit,
    Headless,
}

/// Responsible for passing events to [`App`](crate::app::App) and driving the `App` forward.
pub trait Driver {
    /// Runs the app until completion. Usually this is when all windows are closed.
    fn run(self);
}
