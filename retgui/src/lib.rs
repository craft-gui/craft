//! A retained GUI.

pub use retgui_primitives::brush::Brush;
pub use retgui_primitives::{Color, ColorStop, Extend, Gradient, GradientKind, HueDirection, LinearGradientData, RadialGradientData, SweepGradientData, geometry, palette};

pub use retgui_renderer::RendererType;

pub use retgui_resource_manager::ResourceId;

pub use retgui_runtime::{self, RetGuiRuntime};

pub use image;

pub use winit::dpi::{PhysicalSize as WinitPhysicalSize, Size as WinitSize};
#[cfg(target_os = "android")]
pub use winit::platform::android::activity::*;
pub use winit::window::{Cursor, CursorIcon, Window as WinitWindow, WindowAttributes};

pub use crate::options::RetGuiOptions;
pub use crate::utils::retgui_error::RetGuiError;
pub use crate::utils::style_helpers::{auto, pct, px, rgb, rgba};

#[cfg(target_os = "android")]
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use retgui_logging::info;

use retgui_resource_manager::ResourceManager;

use retgui_runtime::{Receiver, RetGuiRuntimeHandle, Sender, channel};

use crate::accessibility::ACCESS_TREE;
use crate::app::App;
use crate::events::internal::InternalMessage;
use crate::utils::cloneable_any::CloneableAny;
#[cfg(target_arch = "wasm32")]
use crate::wasm_queue::WASM_QUEUE;
use drivers::winit::WinitDriver;
use drivers::{Driver, DriverKind};

mod accessibility;

pub mod drivers;
pub mod elements;
pub mod events;
pub mod layout;
pub mod style;
pub mod text;
#[cfg(target_arch = "wasm32")]
pub mod wasm_queue;
pub mod winit {
    pub use winit::*;
}

mod app;
mod options;
mod perf_stats;
mod utils;
mod window_manager;

#[cfg(target_arch = "wasm32")]
pub type FutureAny = dyn Future<Output = Box<dyn CloneableAny>> + 'static;

#[cfg(not(target_arch = "wasm32"))]
pub type FutureAny = dyn Future<Output = Box<dyn CloneableAny + Send + Sync>> + 'static + Send;

pub type PinnedFutureAny = Pin<Box<FutureAny>>;

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
    retgui_main_internal(Some(options));
}

/// Sets the [`AndroidApp`] for retgui to use.
#[cfg(target_os = "android")]
pub fn retgui_set_android_app(app: AndroidApp) {
    ANDROID_APP.with_borrow_mut(|android_app| {
        *android_app = Some(app);
    })
}

fn retgui_main_internal(options: Option<RetGuiOptions>) {
    info!("RetGui started");

    let options = options.unwrap_or_default();
    let driver_kind = options.driver_kind;
    let app = create_app(options);

    match driver_kind {
        DriverKind::Winit => WinitDriver::new(app).run(),
        DriverKind::Headless => drivers::headless::HeadlessApp::new(app).drive(),
    }
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
        event_reducer: Default::default(),
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
