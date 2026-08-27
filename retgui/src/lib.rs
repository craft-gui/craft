//! A retained GUI.

pub use image;

pub use retgui_primitives::brush::Brush;
pub use retgui_primitives::{Color, ColorStop, Extend, Gradient, GradientKind, HueDirection, LinearGradientData, RadialGradientData, SweepGradientData, geometry, palette};

pub use retgui_renderer::RendererType;

pub use retgui_resource_manager::ResourceId;

pub use retgui_runtime::{self, RetGuiRuntime};

pub use winit;

pub use crate::app::{App, WindowEventResult};
pub use crate::options::RetGuiOptions;
pub use crate::utils::retgui_error::RetGuiError;
pub use crate::utils::style_helpers::{auto, pct, px, rgb, rgba};

#[cfg(target_os = "android")]
use std::cell::RefCell;

use retgui_logging::info;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use crate::drivers::Driver;
use crate::drivers::winit::WinitDriver;

pub mod drivers;
pub mod elements;
pub mod events;
pub mod layout;
pub mod style;
pub mod text;

mod accessibility;
mod app;
mod options;
mod perf_stats;
mod utils;
mod window_manager;

#[cfg(target_os = "android")]
thread_local! {
    static ANDROID_APP: RefCell<Option<AndroidApp>> = const { RefCell::new(None) };
}

/// Starts the RetGui application.
///
/// This will block the current thread until all [`Window`](elements::Window) instances have been closed.
/// On Android, [`retgui_set_android_app`] must be called prior.
///
/// # Example
///
/// ```no_run
/// use retgui::{retgui_main, RetGuiOptions};
/// use retgui::elements::Window;
///
/// fn main() {
///     Window::new("RetGui");
///     retgui_main(RetGuiOptions::default());
/// }
/// ```
pub fn retgui_main(options: RetGuiOptions) {
    retgui_main_with_driver::<WinitDriver>(options);
}

/// Starts the RetGui application using a custom [`Driver`].
///
/// RetGui initializes the [`App`], constructs `D` through [`Driver::new`], and
/// transfers ownership of the app to the driver. This is the advanced
/// counterpart to [`retgui_main`]; most applications should use the default
/// winit driver.
///
/// # Example
///
/// ```no_run
/// use retgui::drivers::Driver;
/// use retgui::{App, RetGuiOptions, retgui_main_with_driver};
///
/// struct PlatformDriver {
///     app: App,
/// }
///
/// impl Driver for PlatformDriver {
///     fn new(app: App) -> Self {
///         Self { app }
///     }
///
///     fn run(mut self) {
///         self.app.on_resume(None);
///         while !self.app.close_requested() {
///             self.app.on_about_to_wait(None);
///             // Wait for and forward native events here.
///             break;
///         }
///     }
/// }
///
/// retgui_main_with_driver::<PlatformDriver>(RetGuiOptions::default());
/// ```
pub fn retgui_main_with_driver<D>(options: RetGuiOptions)
where
    D: Driver,
{
    info!("RetGui started");
    D::new(App::new(options)).run();
}

/// Sets the [`AndroidApp`] for retgui to use.
#[cfg(target_os = "android")]
pub fn retgui_set_android_app(app: AndroidApp) {
    ANDROID_APP.with_borrow_mut(|android_app| {
        *android_app = Some(app);
    })
}
