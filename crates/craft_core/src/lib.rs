#[cfg(feature = "accesskit")]
pub mod accessibility;
pub mod components;
pub mod craft_runtime;
mod craft_winit_state;
pub mod elements;
pub mod events;
mod options;
pub mod reactive;
pub mod renderer;
pub mod style;
#[cfg(test)]
mod tests;
pub mod text;

mod app;
#[cfg(feature = "dev_tools")]
pub(crate) mod devtools;
pub mod geometry;
pub mod layout;
pub mod resource_manager;
mod view_introspection;
mod window_context;

pub use craft_runtime::CraftRuntime;
pub use options::CraftOptions;
pub use renderer::color::palette;
pub use renderer::color::Color;

#[cfg(target_os = "android")]
pub use winit::platform::android::activity::*;

use crate::events::CraftMessage;
pub use crate::options::RendererType;
use components::component::ComponentSpecification;
use events::internal::InternalMessage;
use renderer::renderer::Renderer;
use resource_manager::ResourceManager;

use tokio::sync::mpsc::{channel, Receiver, Sender};

use winit::event_loop::EventLoop;
pub use winit::window::{Cursor, CursorIcon};

pub use window_context::WindowContext;

use std::any::Any;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::craft_runtime::CraftRuntimeHandle;
use crate::reactive::reactive_tree::ReactiveTree;
use crate::reactive::state_store::{StateStore, StateStoreItem};
#[cfg(target_arch = "wasm32")]
use crate::resource_manager::wasm_queue::WASM_QUEUE;
use craft_winit_state::CraftWinitState;

use cfg_if::cfg_if;
use craft_logging::info;

#[cfg(target_os = "android")]
use {winit::event_loop::EventLoopBuilder, winit::platform::android::EventLoopBuilderExtAndroid};

use app::App;

#[cfg(target_arch = "wasm32")]
pub type FutureAny = dyn Future<Output = Box<dyn Any>> + 'static;

#[cfg(not(target_arch = "wasm32"))]
pub type FutureAny = dyn Future<Output = Box<dyn Any + Send + Sync>> + 'static + Send;

pub type PinnedFutureAny = Pin<Box<FutureAny>>;

#[cfg(not(target_arch = "wasm32"))]
type RendererBox = Box<dyn Renderer + Send>;
#[cfg(target_arch = "wasm32")]
type RendererBox = Box<dyn Renderer>;

#[cfg(target_os = "android")]
pub fn internal_craft_main_with_options(
    application: ComponentSpecification,
    global_state: GlobalState,
    options: Option<CraftOptions>,
    app: AndroidApp,
) {
    info!("Craft started");

    info!("Created winit event loop");

    let event_loop =
        EventLoopBuilder::default().with_android_app(app).build().expect("Failed to create winit event loop.");
    craft_main_with_options_2(event_loop, application, global_state, options)
}

pub(crate) type GlobalState = Box<dyn Any + Send + 'static>;

/// Starts the Craft application with the provided component specification, global state, and configuration options.
///
/// This function serves as the main entry point for launching a Craft application. It accepts a component
/// specification, a boxed global state, and optional configuration options, then delegates to the internal
/// launcher [`internal_craft_main_with_options`]. This abstraction allows users to configure their application
/// behavior via [`CraftOptions`] without interacting directly with lower-level details.
///
/// # Type Parameters
///
/// * `GlobalState`: The type use for global state. It must implement [`Send`] and have a `'static` lifetime
///   to ensure it can be safely transferred between threads.
///
/// # Parameters
///
/// * `application` - A [`ComponentSpecification`] that describes the structure and behavior of the application's components.
/// * `global_state` - A boxed instance of type `GlobalState` which holds the application's global state.
/// * `options` - An optional [`CraftOptions`] configuration. If `None` is provided, default options will be applied.
#[cfg(not(target_os = "android"))]
pub fn craft_main<GlobalState: Send + 'static>(
    application: ComponentSpecification,
    global_state: GlobalState,
    options: CraftOptions,
) {
    internal_craft_main_with_options(application, Box::new(global_state), Some(options));
}

/// Starts the Craft application with the provided component specification, global state, and configuration options.
///
/// This function serves as the main entry point for launching a Craft application. It accepts a component
/// specification, a boxed global state, and optional configuration options, then delegates to the internal
/// launcher [`internal_craft_main_with_options`]. This abstraction allows users to configure their application
/// behavior via [`CraftOptions`] without interacting directly with lower-level details.
///
/// # Type Parameters
///
/// * `GlobalState`: The type used for global state. It must implement [`Send`] and have a `'static` lifetime
///   to ensure it can be safely transferred between threads.
///
/// # Parameters
///
/// * `application` - A [`ComponentSpecification`] that describes the structure and behavior of the application's components.
/// * `global_state` - A boxed instance of type `GlobalState` which holds the application's global state.
/// * `options` - An optional [`CraftOptions`] configuration. If `None` is provided, default options will be applied.
/// * `android_app` - The Android application instance.
#[cfg(target_os = "android")]
pub fn craft_main<GlobalState: Send + 'static>(
    application: ComponentSpecification,
    global_state: GlobalState,
    options: CraftOptions,
    android_app: AndroidApp,
) {
    internal_craft_main_with_options(application, Box::new(global_state), Some(options), android_app);
}

#[cfg(not(target_os = "android"))]
fn internal_craft_main_with_options(
    application: ComponentSpecification,
    global_state: GlobalState,
    options: Option<CraftOptions>,
) {
    info!("Craft started");

    let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
    info!("Created winit event loop.");

    craft_main_with_options_2(event_loop, application, global_state, options)
}

fn craft_main_with_options_2(
    event_loop: EventLoop<()>,
    application: ComponentSpecification,
    global_state: GlobalState,
    craft_options: Option<CraftOptions>,
) {
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
    let resource_manager = Arc::new(ResourceManager::new(app_sender.clone(), runtime.clone()));

    let mut user_state = StateStore::default();

    let dummy_root_value: Box<StateStoreItem> = Box::new(());
    user_state.storage.insert(0, dummy_root_value);

    let mut dev_tools_user_state = StateStore::default();
    dev_tools_user_state.storage.insert(0, Box::new(()));

    let craft_app = Box::new(App {
        app_sender: app_sender.clone(),
        #[cfg(feature = "accesskit")]
        accesskit_adapter: None,
        app: application,
        global_state,
        window: None,
        text_context: None,
        renderer: None,
        window_context: WindowContext::new(),
        resource_manager,
        resources_collected: Default::default(),
        reload_fonts: false,
        user_tree: ReactiveTree {
            element_tree: None,
            component_tree: None,
            element_ids: Default::default(),
            component_ids: Default::default(),
            pointer_captures: Default::default(),
            update_queue: VecDeque::new(),
            user_state,
            element_state: Default::default(),
            focus: None,
        },

        #[cfg(feature = "dev_tools")]
        is_dev_tools_open: false,

        #[cfg(feature = "dev_tools")]
        dev_tree: ReactiveTree {
            element_tree: None,
            component_tree: None,
            update_queue: VecDeque::new(),
            user_state: dev_tools_user_state,
            element_state: Default::default(),
            element_ids: Default::default(),
            component_ids: Default::default(),
            pointer_captures: Default::default(),
            focus: None,
        },
        runtime: runtime_copy,
        modifiers: Default::default(),
    });

    let mut app = CraftWinitState::new(runtime, winit_receiver, app_sender, craft_options, craft_app);

    event_loop.run_app(&mut app).expect("run_app failed");
}

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
                InternalMessage::GotUserMessage(message) => {
                    winit_sender
                        .send(InternalMessage::GotUserMessage(message))
                        .await
                        .expect("Failed to send user message");
                }
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
