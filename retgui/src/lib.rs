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
use std::sync::Arc;

use retgui_logging::info;

use retgui_resource_manager::ResourceManager;

use retgui_runtime::{Receiver, RetGuiRuntimeHandle, Sender, channel};

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use crate::accessibility::ACCESS_TREE;
use crate::drivers::Driver;
use crate::drivers::winit::WinitDriver;
use crate::events::internal::InternalMessage;
#[cfg(target_arch = "wasm32")]
use crate::wasm_queue::WASM_QUEUE;

pub mod drivers;
pub mod elements;
pub mod events;
pub mod layout;
pub mod style;
pub mod text;
#[cfg(target_arch = "wasm32")]
pub mod wasm_queue;

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
    D::new(create_app(options)).run();
}

/// Sets the [`AndroidApp`] for retgui to use.
#[cfg(target_os = "android")]
pub fn retgui_set_android_app(app: AndroidApp) {
    ANDROID_APP.with_borrow_mut(|android_app| {
        *android_app = Some(app);
    })
}

fn create_app(retgui_options: RetGuiOptions) -> App {
    let (app_sender, app_receiver) = channel::<InternalMessage>(100);
    let (runtime_sender, mut runtime_receiver) = channel::<RetGuiRuntimeHandle>(1);
    let (winit_sender, winit_receiver) = channel::<InternalMessage>(100);

    let winit_sender_copy = winit_sender.clone();

    let setup_runtime = move || {
        let runtime = RetGuiRuntime::new();
        runtime_sender
            .blocking_send(runtime.handle())
            .expect("Failed to send runtime handle");

        info!("Created async runtime");

        let future = async_main(app_receiver, winit_sender_copy);
        runtime.maybe_block_on(future);
    };

    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(setup_runtime);

    #[cfg(target_arch = "wasm32")]
    setup_runtime();

    let runtime = runtime_receiver
        .blocking_recv()
        .expect("Failed to receive runtime handle");

    #[allow(clippy::arc_with_non_send_sync)]
    let resource_manager = Arc::new(ResourceManager::new(runtime.clone()));

    let (event_dispatcher, text_context) =
        ACCESS_TREE.with(|access_tree| (access_tree.event_dispatcher.clone(), access_tree.text_context.clone()));

    App {
        event_dispatcher,
        app_sender: app_sender.clone(),
        event_receiver: winit_receiver,
        text_context,
        resource_manager,
        reload_fonts: false,
        runtime: runtime.clone(),
        target_scratch: Vec::new(),
        retgui_options: retgui_options.clone(),
        active: false,
        wait_cancelled: false,
        close_requested: false,
        #[cfg(feature = "audio")]
        last_audio_ui_update: None,
    }
}

#[allow(unused_variables)]
async fn async_main(mut app_receiver: Receiver<InternalMessage>, winit_sender: Sender<InternalMessage>) {
    info!("starting main event loop");
    while let Some(app_message) = app_receiver.recv().await {
        #[cfg(target_arch = "wasm32")]
        WASM_QUEUE.with_borrow_mut(|wasm_queue| {
            wasm_queue.push(app_message);
        });

        #[cfg(not(target_arch = "wasm32"))]
        match app_message {
            InternalMessage::ResourceEvent(resource_event) => {
                winit_sender
                    .send(InternalMessage::ResourceEvent(resource_event))
                    .await
                    .expect("Failed to send resource event");
            }
        }
    }
}
