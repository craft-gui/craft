#[derive(Clone, Copy, Default)]
pub(crate) enum DriverKind {
    #[default]
    Winit,
    #[cfg(test)]
    Test,
}

/// Responsible for passing events to [`App`](crate::app::App) and driving the `App` forward.
pub trait Driver {
    /// Runs the app until completion. Usually this is when all windows are closed.
    fn run(&mut self);
}
