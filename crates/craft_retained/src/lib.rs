#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
pub mod accessibility;
pub mod craft_winit_state;
pub mod document;
pub mod elements;
pub mod events;
mod options;
pub mod style;
#[cfg(test)]
mod tests;
pub mod text;

mod app;
pub use craft_primitives::geometry;
pub mod layout;
pub use craft_runtime::CraftRuntime;
mod craftcallback;
pub mod spatial;
mod utils;
#[cfg(target_arch = "wasm32")]
pub mod wasm_queue;
mod window_manager;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use app::App;
use cfg_if::cfg_if;
use craft_logging::info;
pub use craft_primitives::{Color, palette};
pub use craft_renderer::RendererType;
pub use craft_resource_manager::ResourceIdentifier;
use craft_resource_manager::ResourceManager;
use craft_runtime::{CraftRuntimeHandle, Receiver, Sender, channel};
use craft_winit_state::CraftState;
pub use craftcallback::CraftCallback;
use events::internal::InternalMessage;
pub use options::CraftOptions;
pub use utils::craft_error::CraftError;
use winit::event_loop::EventLoopBuilder;
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;
#[cfg(target_os = "android")]
pub use winit::platform::android::activity::*;
pub use winit::window::{Cursor, CursorIcon, WindowAttributes};

use crate::app::RedrawFlags;
use crate::craft_winit_state::CraftWinitState;
use crate::events::EventDispatcher;
use crate::utils::cloneable_any::CloneableAny;
pub use crate::utils::style_helpers::{auto, pct, px, rgb, rgba};
#[cfg(target_arch = "wasm32")]
use crate::wasm_queue::WASM_QUEUE;

#[cfg(target_arch = "wasm32")]
pub type FutureAny = dyn Future<Output = Box<dyn CloneableAny>> + 'static;

#[cfg(not(target_arch = "wasm32"))]
pub type FutureAny = dyn Future<Output = Box<dyn CloneableAny + Send + Sync>> + 'static + Send;

pub type PinnedFutureAny = Pin<Box<FutureAny>>;

#[cfg(target_os = "android")]
use std::cell::RefCell;

pub use craft_runtime;
pub use image;

#[cfg(target_os = "android")]
thread_local! {
    static ANDROID_APP: RefCell<Option<AndroidApp>> = const { RefCell::new(None) };
}

fn craft_main_internal(options: Option<CraftOptions>) {
    info!("Craft started");

    let mut event_loop_builder = EventLoopBuilder::default();

    #[cfg(target_os = "android")]
    {
        let app = ANDROID_APP.take().expect("craft_set_android_app must be called.");
        event_loop_builder.with_android_app(app);
    }
    let event_loop = event_loop_builder.build().expect("Failed to create winit event loop.");
    info!("Created winit event loop.");

    let craft_state = setup_craft(options);
    let mut winit_craft_state = CraftWinitState::new(craft_state);
    event_loop.run_app(&mut winit_craft_state).expect("run_app failed");
}

/// Starts the Craft application.
///
/// This will block the current thread until all [`Window`](elements::Window) instances have been closed.
///
/// # Example
///
/// ```no_run
/// use craft_retained::{craft_main, CraftOptions};
/// use craft_retained::elements::Window;
///
/// fn main() {
///     Window::new("Craft");
///     craft_main(CraftOptions::default());
/// }
/// ```
pub fn craft_main(options: CraftOptions) {
    craft_main_internal(Some(options));
}

fn setup_craft(craft_options: Option<CraftOptions>) -> CraftState {
    let craft_options = craft_options.unwrap_or_default();

    let (app_sender, app_receiver) = channel::<InternalMessage>(100);
    let (runtime_sender, mut runtime_receiver) = channel::<CraftRuntimeHandle>(1);
    let (winit_sender, winit_receiver) = channel::<InternalMessage>(100);

    let winit_sender_copy = winit_sender.clone();
    cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            std::thread::spawn(move || {
                let runtime = CraftRuntime::new();
                runtime_sender.blocking_send(runtime.handle()).expect("Failed to send runtime handle");
                info!("Created async runtime");

                let future = async_main(app_receiver, winit_sender_copy);

                runtime.maybe_block_on(future);
            });
        } else {
            let runtime = CraftRuntime::new();
            runtime_sender.blocking_send(runtime.handle()).expect("Failed to send runtime handle");
            info!("Created async runtime");

            let future = crate::async_main(app_receiver, winit_sender_copy);

            runtime.maybe_block_on(future);
        }
    }

    let runtime = runtime_receiver
        .blocking_recv()
        .expect("Failed to receive runtime handle");
    let runtime_copy = runtime.clone();
    #[allow(clippy::arc_with_non_send_sync)]
    let resource_manager = Arc::new(ResourceManager::new(runtime.clone()));

    let craft_app = Box::new(App {
        event_dispatcher: EventDispatcher::new(),
        app_sender: app_sender.clone(),
        text_context: None,
        resource_manager,
        reload_fonts: false,

        runtime: runtime_copy,
        modifiers: Default::default(),
        redraw_flags: RedrawFlags::new(true),
        target_scratch: Vec::new(),

        craft_options: craft_options.clone(),
        active: false,
    });

    CraftState::new(runtime, winit_receiver, app_sender, craft_options, craft_app)
}

#[allow(unused_variables)]
async fn async_main(mut app_receiver: Receiver<InternalMessage>, winit_sender: Sender<InternalMessage>) {
    info!("starting main event loop");
    loop {
        if let Some(app_message) = app_receiver.recv().await {
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
}

#[cfg(target_os = "android")]
pub fn craft_set_android_app(app: AndroidApp) {
    ANDROID_APP.with_borrow_mut(|android_app| {
        *android_app = Some(app);
    })
}
