#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
pub mod accessibility;
pub mod craft_winit_state;
pub mod elements;
pub mod events;
pub mod animations;
mod options;
pub mod style;
#[cfg(test)]
mod tests;
pub mod text;
pub mod document;

mod app;
pub use craft_primitives::geometry as geometry;
pub mod layout;
pub use craft_runtime::CraftRuntime;
mod window_context;
mod utils;
#[cfg(target_arch = "wasm32")]
pub mod wasm_queue;
pub mod spatial;

pub use options::CraftOptions;
pub use craft_primitives::palette;
pub use craft_primitives::Color;

#[cfg(target_os = "android")]
pub use winit::platform::android::activity::*;

pub use craft_renderer::RendererType;
use events::internal::InternalMessage;
use craft_renderer::renderer::Renderer;
use craft_resource_manager::ResourceManager;
pub use craft_resource_manager::ResourceIdentifier;

use craft_runtime::{channel, CraftRuntimeHandle, Receiver, Sender};

pub use winit::window::{Cursor, CursorIcon};

pub use utils::craft_error::CraftError;

pub use window_context::WindowContext;

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time;
#[cfg(target_arch = "wasm32")]
use web_time as time;
#[cfg(target_arch = "wasm32")]
use crate::wasm_queue::WASM_QUEUE;
use craft_winit_state::CraftState;

use cfg_if::cfg_if;
use craft_logging::info;

#[cfg(target_os = "android")]
use {winit::event_loop::EventLoopBuilder, winit::platform::android::EventLoopBuilderExtAndroid};

use app::App;
use craft_renderer::RenderList;
use crate::app::RedrawFlags;
use crate::craft_winit_state::CraftWinitState;
use crate::elements::Element;
use crate::events::EventDispatcher;
use crate::utils::cloneable_any::CloneableAny;

#[cfg(target_arch = "wasm32")]
pub type FutureAny = dyn Future<Output = Box<dyn CloneableAny>> + 'static;

#[cfg(not(target_arch = "wasm32"))]
pub type FutureAny = dyn Future<Output = Box<dyn CloneableAny + Send + Sync>> + 'static + Send;

pub type PinnedFutureAny = Pin<Box<FutureAny>>;

#[cfg(not(target_arch = "wasm32"))]
type RendererBox = Box<dyn Renderer>;
#[cfg(target_arch = "wasm32")]
type RendererBox = Box<dyn Renderer>;

#[cfg(target_os = "android")]
pub fn internal_craft_main_with_options(
    root: Rc<RefCell<dyn Element>>,
    options: Option<CraftOptions>,
    app: AndroidApp,
) {
    info!("Craft started");

    let event_loop =
        EventLoopBuilder::default().with_android_app(app).build().expect("Failed to create winit event loop.");
    info!("Created winit event loop.");

    let craft_state = setup_craft(root, options);
    let mut winit_craft_state = CraftWinitState::new(craft_state);
    event_loop.run_app(&mut winit_craft_state).expect("run_app failed");
}

/// Starts the Craft application with the provided component specification, global state, and configuration options.
///
/// This function serves as the main entry point for launching a Craft application. It accepts a component
/// specification, a boxed global state, and optional configuration options, then delegates to the internal
/// launcher [`internal_craft_main_with_options`]. This abstraction allows users to configure their application
/// behavior via [`CraftOptions`] without interacting directly with lower-level details.
///
/// # Parameters
///
/// * `root` - The root element.
/// * `options` - An optional [`CraftOptions`] configuration. If `None` is provided, default options will be applied.
#[cfg(not(target_os = "android"))]
pub fn craft_main(
    root: Rc<RefCell<dyn Element>>,
    options: CraftOptions,
) {
    internal_craft_main_with_options(root, Some(options));
}

/// Starts the Craft application with the provided component specification, global state, and configuration options.
///
/// This function serves as the main entry point for launching a Craft application. It accepts a component
/// specification, a boxed global state, and optional configuration options, then delegates to the internal
/// launcher [`internal_craft_main_with_options`]. This abstraction allows users to configure their application
/// behavior via [`CraftOptions`] without interacting directly with lower-level details.
///
/// # Parameters
///
/// * `root` - The root element.
/// * `options` - An optional [`CraftOptions`] configuration. If `None` is provided, default options will be applied.
/// * `android_app` - The Android application instance.
#[cfg(target_os = "android")]
pub fn craft_main(
    root: Rc<RefCell<dyn Element>>,
    options: CraftOptions,
    android_app: AndroidApp,
) {
    internal_craft_main_with_options(root, Some(options), android_app);
}

#[cfg(not(target_os = "android"))]
fn internal_craft_main_with_options(
    root: Rc<RefCell<dyn Element>>,
    options: Option<CraftOptions>,
) {
    use winit::event_loop::EventLoop;

    info!("Craft started");
    let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
    info!("Created winit event loop.");

    let craft_state = setup_craft(root, options);
    let mut winit_craft_state = CraftWinitState::new(craft_state);
    event_loop.run_app(&mut winit_craft_state).expect("run_app failed");
}

pub fn setup_craft(
    root: Rc<RefCell<dyn Element>>,
    craft_options: Option<CraftOptions>,
) -> CraftState {
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

    let runtime = runtime_receiver.blocking_recv().expect("Failed to receive runtime handle");
    let runtime_copy = runtime.clone();
    #[allow(clippy::arc_with_non_send_sync)]
    let resource_manager = Arc::new(ResourceManager::new(runtime.clone()));

    let craft_app = Box::new(App {
        event_dispatcher: EventDispatcher::new(),
        root,
        app_sender: app_sender.clone(),
        #[cfg(feature = "accesskit")]
        accesskit_adapter: None,
        window: None,
        text_context: None,
        renderer: None,
        window_context: WindowContext::new(),
        resource_manager,
        reload_fonts: false,

        runtime: runtime_copy,
        modifiers: Default::default(),
        last_frame_time: time::Instant::now(),
        redraw_flags: RedrawFlags::new(true),
        render_list: RenderList::new(),

        previous_animation_flags: Default::default(),
        focus: None,
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

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba8(r, g, b, a)
}
