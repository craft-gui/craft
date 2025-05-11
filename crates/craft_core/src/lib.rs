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
mod text;

pub mod app_message;
#[cfg(feature = "dev_tools")]
pub(crate) mod devtools;
pub mod geometry;
pub mod resource_manager;
mod view_introspection;
mod layout;

pub use craft_runtime::CraftRuntime;
pub use options::CraftOptions;
pub use renderer::color::palette;
pub use renderer::color::Color;

#[cfg(target_os = "android")]
pub use winit::platform::android::activity::*;

use crate::events::{CraftMessage, Event, EventDispatchType, KeyboardInput, MouseWheel, PointerButton, PointerMoved};
pub use crate::options::RendererType;
use crate::reactive::element_state_store::ElementStateStore;
use crate::style::{Display, Unit, Wrap};
use app_message::AppMessage;
use components::component::{ComponentId, ComponentSpecification};
use elements::container::Container;
use elements::element::Element;
use layout::layout_context::{measure_content, LayoutContext};
use events::internal::InternalMessage;
use events::resource_event::ResourceEvent;
use events::update_queue_entry::UpdateQueueEntry;
use events::Message;
use reactive::element_id::reset_unique_element_id;
use reactive::fiber_node::FiberNode;
use reactive::tree::{diff_trees, ComponentTreeNode};
use renderer::renderer::Renderer;
use resource_manager::ResourceManager;

#[cfg(target_arch = "wasm32")]
use {std::cell::RefCell, web_time as time};

#[cfg(target_arch = "wasm32")]
thread_local! {
    pub static MESSAGE_QUEUE: RefCell<Vec<Message>> = const {RefCell::new(Vec::new())};
}

use taffy::{AvailableSpace, NodeId, TaffyTree};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event_loop::EventLoop;
#[cfg(feature = "dev_tools")]
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window};
pub use winit::window::{Cursor, CursorIcon};

use std::any::Any;
use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use cfg_if::cfg_if;
use craft_logging::{info, span, Level};
#[cfg(not(target_arch = "wasm32"))]
use std::time;
use winit::event::{ElementState, Ime, Modifiers, MouseButton};
#[cfg(target_os = "android")]
use {winit::event_loop::EventLoopBuilder, winit::platform::android::EventLoopBuilderExtAndroid};

#[cfg(target_arch = "wasm32")]
use {
    parley::GenericFamily,
    peniko::Blob,
};

const WAIT_TIME: time::Duration = time::Duration::from_millis(15);
#[cfg(target_arch = "wasm32")]
pub type FutureAny = dyn Future<Output = Box<dyn Any>> + 'static;

#[cfg(not(target_arch = "wasm32"))]
pub type FutureAny = dyn Future<Output = Box<dyn Any + Send + Sync>> + 'static + Send;

pub type PinnedFutureAny = Pin<Box<FutureAny>>;

#[derive(Default)]
struct ReactiveTree {
    element_tree: Option<Box<dyn Element>>,
    component_tree: Option<ComponentTreeNode>,
    element_ids: HashSet<ComponentId>,
    component_ids: HashSet<ComponentId>,
    /// Stores a pointer device id and their pointer captured element.
    pointer_captures: HashMap<i64, ComponentId>,
    update_queue: VecDeque<UpdateQueueEntry>,
    user_state: StateStore,
    element_state: ElementStateStore,
}

#[derive(Debug)]
/// User-level API to get and set common window properties.
/// All values are in logical pixels.
pub struct WindowContext {
    window_size: Size<f32>,
    mouse_position: Option<Point>,
    cursor: Option<Cursor>,

    requested_window_width: Option<f32>,
    requested_window_height: Option<f32>,
    requested_mouse_position_x: Option<f32>,
    requested_mouse_position_y: Option<f32>,
    requested_cursor: Option<Cursor>,
}

impl WindowContext {
    pub(crate) fn new() -> WindowContext {
        Self {
            window_size: Default::default(),
            mouse_position: None,
            cursor: None,
            requested_window_width: None,
            requested_window_height: None,
            requested_mouse_position_x: None,
            requested_mouse_position_y: None,
            requested_cursor: None,
        }
    }

    pub fn cursor(&self) -> Option<Cursor> {
        self.cursor.clone()
    }
    
    pub fn window_width(&self) -> f32 {
        self.window_size.width
    }
    pub fn window_height(&self) -> f32 {
        self.window_size.height
    }
    
    pub fn mouse_position_x(&self) -> Option<f32> {
        self.mouse_position.map(|pos| pos.x)
    }

    pub fn mouse_position_y(&self) -> Option<f32> {
        self.mouse_position.map(|pos| pos.y)
    }

    pub fn set_window_width(&mut self, width: f32) {
        self.requested_window_width = Some(width);
    }

    pub fn set_window_height(&mut self, height: f32) {
        self.requested_window_height = Some(height);
    }

    pub fn set_mouse_position_x(&mut self, x: f32) {
        self.requested_mouse_position_x = Some(x);
    }

    pub fn set_mouse_position_y(&mut self, y: f32) {
        self.requested_mouse_position_y = Some(y);
    }
    
    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.requested_cursor = Some(cursor);
    }
    
    pub(crate) fn reset(&mut self) {
        *self = WindowContext {
            window_size: self.window_size,
            mouse_position: self.mouse_position,
            cursor: None,
            
            requested_window_width: None,
            requested_window_height: None,
            requested_mouse_position_x: None,
            requested_mouse_position_y: None,
            requested_cursor: None,
        }
    }
}

pub(crate) fn get_scale_factor(window: &Option<Arc<dyn Window>>) -> f64 {
    if let Some(window) = window {
        window.scale_factor()
    } else { 
        1.0
    }
}

struct App {
    app: ComponentSpecification,
    global_state: GlobalState,
    window: Option<Arc<dyn Window>>,
    text_context: Option<TextContext>,
    renderer: Option<Box<dyn Renderer + Send>>,
    mouse_position: Option<Point>,
    reload_fonts: bool,
    resource_manager: Arc<ResourceManager>,
    /// Resources that have already been collected.
    /// We use this in view_introspection, so that we don't request the download
    /// of a resource too many times.
    resources_collected: HashMap<ResourceIdentifier, bool>,
    winit_sender: Sender<AppMessage>,

    user_tree: ReactiveTree,
    window_context: WindowContext,

    #[cfg(feature = "dev_tools")]
    is_dev_tools_open: bool,

    #[cfg(feature = "dev_tools")]
    dev_tree: ReactiveTree,
}

impl App {
    fn setup_text_context(&mut self) {
        if self.text_context.is_none() {
            #[cfg(target_arch = "wasm32")]
            let mut text_context = TextContext::new();
            #[cfg(not(target_arch = "wasm32"))]
            let text_context = TextContext::new();

            #[cfg(target_arch = "wasm32")]
            {
                let variable_roboto = include_bytes!("../../../fonts/Roboto-VariableFont_wdth,wght.ttf");
                let roboto_blog = Blob::new(Arc::new(variable_roboto));
                let fonts = text_context.font_context.collection.register_fonts(roboto_blog, None);

                // Register all the Roboto families under GenericFamily::SystemUi. 
                // This will become the fallback font for platforms like WASM.
                text_context.font_context.collection.append_generic_families(GenericFamily::SystemUi, fonts.iter().map(|f| f.0));                
            }
            
            self.text_context = Some(text_context);
        }
    }
}

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

#[cfg(feature = "dev_tools")]
use crate::devtools::dev_tools_component::dev_tools_view;

use crate::components::PointerCapture;
use crate::elements::base_element_state::DUMMY_DEVICE_ID;
use crate::geometry::{Point, Rectangle, Size};
use crate::reactive::state_store::{StateStore, StateStoreItem};
use crate::resource_manager::resource_type::ResourceType;
use crate::view_introspection::scan_view_for_resources;
use craft_winit_state::CraftWinitState;
use crate::renderer::renderer::RenderList;
use crate::resource_manager::ResourceIdentifier;
use crate::text::text_context::TextContext;

pub(crate) type GlobalState = Box<dyn Any + Send + 'static>;

/// Starts the Craft application with the provided component specification, global state, and configuration options.
///
/// This function serves as the main entry point for launching an Craft application. It accepts a component
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
pub fn craft_main_with_options<GlobalState: Send + 'static>(
    application: ComponentSpecification,
    global_state: GlobalState,
    options: Option<CraftOptions>,
) {
    internal_craft_main_with_options(application, Box::new(global_state), options);
}

/// Starts the Craft application with the provided component specification, global state, and configuration options.
///
/// This function serves as the main entry point for launching an Craft application. It accepts a component
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
pub fn craft_main_with_options<GlobalState: Send + 'static>(
    application: ComponentSpecification,
    global_state: GlobalState,
    options: Option<CraftOptions>,
    android_app: AndroidApp,
) {
    internal_craft_main_with_options(application, Box::new(global_state), options, android_app);
}

#[cfg(not(target_os = "android"))]
fn internal_craft_main_with_options(
    application: ComponentSpecification,
    global_state: GlobalState,
    options: Option<CraftOptions>,
) {
    info!("Craft started");

    info!("Creating winit event loop.");

    let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
    info!("Created winit event loop.");

    craft_main_with_options_2(event_loop, application, global_state, options)
}

fn craft_main_with_options_2(
    event_loop: EventLoop,
    application: ComponentSpecification,
    global_state: GlobalState,
    craft_options: Option<CraftOptions>,
) {
    let craft_options = craft_options.unwrap_or_default();

    let runtime = CraftRuntime::new();
    info!("Created async runtime");

    let (app_sender, app_receiver) = channel::<AppMessage>(100);
    let (winit_sender, winit_receiver) = channel::<AppMessage>(100);
    let resource_manager = Arc::new(ResourceManager::new(app_sender.clone()));

    let app_sender_copy = app_sender.clone();
    let resource_manager_copy = resource_manager.clone();

    let future =
        async_main(application, app_receiver, winit_sender, app_sender_copy, resource_manager_copy, global_state);

    runtime.runtime_spawn(future);

    let mut app = CraftWinitState::new(runtime, winit_receiver, app_sender, craft_options);

    event_loop.run_app(&mut app).expect("run_app failed");
}

#[cfg(target_arch = "wasm32")]
async fn send_response(_app_message: AppMessage, _sender: &mut Sender<AppMessage>) {
}

#[cfg(not(target_arch = "wasm32"))]
async fn send_response(app_message: AppMessage, sender: &mut Sender<AppMessage>) {
    if app_message.blocking {
        sender.send(app_message).await.expect("send failed");
    }
}

async fn async_main(
    component_spec_application: ComponentSpecification,
    mut app_receiver: Receiver<AppMessage>,
    winit_sender: Sender<AppMessage>,
    mut app_sender: Sender<AppMessage>,
    resource_manager: Arc<ResourceManager>,
    global_state: GlobalState,
) {
    let mut user_state = StateStore::default();

    let dummy_root_value: Box<StateStoreItem> = Box::new(());
    user_state.storage.insert(0, dummy_root_value);

    let mut dev_tools_user_state = StateStore::default();
    dev_tools_user_state.storage.insert(0, Box::new(()));

    let mut app = Box::new(App {
        app: component_spec_application,
        global_state,
        window: None,
        text_context: None,
        renderer: None,
        window_context: WindowContext::new(),
        resource_manager,
        resources_collected: Default::default(),
        winit_sender: winit_sender.clone(),
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
        },
        mouse_position: None,
    });

    info!("starting main event loop");
    loop {
        if let Some(app_message) = app_receiver.recv().await {
            let mut dummy_message = AppMessage::new(app_message.id, InternalMessage::Confirmation);
            dummy_message.blocking = app_message.blocking;

            match app_message.data {
                InternalMessage::RequestRedraw(scale_factor, surface_size) => {
                    on_request_redraw(&mut app, scale_factor, surface_size).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::Close => {
                    info!("Craft Closing");

                    send_response(dummy_message, &mut app.winit_sender).await;
                    break;
                }
                InternalMessage::Confirmation => {}
                InternalMessage::Resume(window, renderer) => {
                    on_resume(&mut app, window.clone(), renderer).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::Resize(new_size) => {
                    on_resize(&mut app, new_size).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::MouseWheel(mouse_wheel) => {
                    on_mouse_wheel(&mut app, mouse_wheel).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::PointerButton(pointer_button) => {
                    on_pointer_button(&mut app, pointer_button).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::PointerMoved(pointer_moved) => {
                    on_pointer_moved(&mut app, pointer_moved.clone()).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::Ime(ime) => {
                    on_ime(&mut app, ime.clone()).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::ProcessUserEvents => {
                    on_process_user_events(app.window.clone(), &mut app_sender, &mut app.user_tree);
                    #[cfg(feature = "dev_tools")]
                    on_process_user_events(app.window.clone(), &mut app_sender, &mut app.dev_tree);
                }
                InternalMessage::GotUserMessage(message) => {
                    let update_fn = message.0;
                    let source_component = message.1;
                    let props = message.3;
                    let message = message.2;

                    let state = app.user_tree.user_state.storage.get_mut(&source_component).unwrap().as_mut();
                    update_fn(state, &mut app.global_state, props, Event::new(&Message::UserMessage(message)), &mut app.window_context);
                    app.window.as_ref().unwrap().request_redraw();
                }
                InternalMessage::ResourceEvent(resource_event) => {
                    let resource_manager = &mut app.resource_manager;

                    match resource_event {
                        ResourceEvent::Loaded(resource_identifier, resource_type, resource) => {
                            if resource_type == ResourceType::Font {
                                if let Some(_text_context) = app.text_context.as_mut() {
                                    if resource.data().is_some() {
                                        // Todo: Load the font into the text context.
                                        resource_manager.resources.insert(resource_identifier.clone(), Arc::new(resource));
                                    }
                                }

                                app.reload_fonts = true;
                                app.window.as_ref().unwrap().request_redraw();
                            } else if resource_type == ResourceType::Image || resource_type == ResourceType::TinyVg {
                                resource_manager.resources.insert(resource_identifier, Arc::new(resource));
                                app.window.as_ref().unwrap().request_redraw();
                            }
                        }
                        ResourceEvent::UnLoaded(_) => {}
                    }
                }
                InternalMessage::KeyboardInput(keyboard_input) => {
                    on_keyboard_input(&mut app, keyboard_input).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::ModifiersChanged(modifiers) => {
                    on_modifiers_input(&mut app, modifiers).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
            }
        }
    }
}

fn on_process_user_events(
    window: Option<Arc<dyn Window>>,
    app_sender: &mut Sender<AppMessage>,
    reactive_tree: &mut ReactiveTree,
) {
    if reactive_tree.update_queue.is_empty() {
        return;
    }

    for event in reactive_tree.update_queue.drain(..) {
        let app_sender_copy = app_sender.clone();
        let window_clone = window.clone().unwrap();
        let f = async move {
            let update_result = event.update_result.future.unwrap();
            let res = update_result.await;
            app_sender_copy
                .send(AppMessage::new(
                    0,
                    InternalMessage::GotUserMessage((event.update_function, event.source_component, res, event.props)),
                ))
                .await
                .expect("send failed");
            window_clone.request_redraw();
        };
        CraftRuntime::native_spawn(f);
    }
}

async fn on_pointer_moved(app: &mut Box<App>, mouse_moved: PointerMoved) {
    app.mouse_position = Some(Point::new(mouse_moved.position.x, mouse_moved.position.y));

    let scale_factor = get_scale_factor(&app.window);
    let logical_mouse_position: LogicalPosition<f32> = LogicalPosition::from_physical(PhysicalPosition::new(mouse_moved.position.x, mouse_moved.position.y), scale_factor);
    app.window_context.mouse_position = Some(Point::new(logical_mouse_position.x, logical_mouse_position.y));
    
    let message = Message::CraftMessage(CraftMessage::PointerMovedEvent(mouse_moved));

    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.user_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        true,
    );

    #[cfg(feature = "dev_tools")]
    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.dev_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        true,
    );

    if let Some(window) = app.window.as_ref() {
        window.request_redraw();
    }
}

async fn on_mouse_wheel(app: &mut Box<App>, mouse_wheel: MouseWheel) {
    let event = CraftMessage::MouseWheelEvent(mouse_wheel);
    let message = Message::CraftMessage(event);

    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.user_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        false,
    );

    #[cfg(feature = "dev_tools")]
    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.dev_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        false,
    );

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_ime(app: &mut Box<App>, ime: Ime) {
    let event = CraftMessage::ImeEvent(ime);
    let message = Message::CraftMessage(event);

    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.user_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        false,
    );

    #[cfg(feature = "dev_tools")]
    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.dev_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        false,
    );

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_modifiers_input(app: &mut Box<App>, modifiers: Modifiers) {
    let modifiers_event = CraftMessage::ModifiersChangedEvent(modifiers);
    let message = Message::CraftMessage(modifiers_event);
    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.user_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        false,
    );

    #[cfg(feature = "dev_tools")]
    {
        dispatch_event(
            &message,
            EventDispatchType::Bubbling,
            &mut app.resource_manager,
            app.mouse_position,
            &mut app.dev_tree,
            &mut app.global_state,
            &mut app.text_context,
            &mut app.window_context,
            false,
        );
    }
    app.window.as_ref().unwrap().request_redraw();
}

async fn on_keyboard_input(app: &mut Box<App>, keyboard_input: KeyboardInput) {
    let keyboard_event = CraftMessage::KeyboardInputEvent(keyboard_input.clone());
    let message = Message::CraftMessage(keyboard_event);

    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.user_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        false,
    );

    #[cfg(feature = "dev_tools")]
    {
        dispatch_event(
            &message,
            EventDispatchType::Bubbling,
            &mut app.resource_manager,
            app.mouse_position,
            &mut app.dev_tree,
            &mut app.global_state,
            &mut app.text_context,
            &mut app.window_context,
            false,
        );

        let logical_key = keyboard_input.event.logical_key;
        let key_state = keyboard_input.event.state;

        if key_state.is_pressed() {
            if let Key::Named(NamedKey::F12) = logical_key {
                app.is_dev_tools_open = !app.is_dev_tools_open;
            }
        }
    }
    app.window.as_ref().unwrap().request_redraw();
}

async fn on_resize(app: &mut Box<App>, new_size: PhysicalSize<u32>) {
    let scale_factor = get_scale_factor(&app.window);
    let window_size: LogicalSize<f32> = new_size.to_logical(scale_factor);
    app.window_context.window_size = Size::new(window_size.width, window_size.height);
    if let Some(renderer) = app.renderer.as_mut() {
        renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);
    }

    // On macOS the window needs to be redrawn manually after resizing
    #[cfg(target_os = "macos")]
    {
        app.window.as_ref().unwrap().request_redraw();
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_event(
    event: &Message,
    dispatch_type: EventDispatchType,
    _resource_manager: &mut Arc<ResourceManager>,
    mouse_position: Option<Point>,
    reactive_tree: &mut ReactiveTree,
    global_state: &mut GlobalState,
    text_context: &mut Option<TextContext>,
    window_context: &mut WindowContext,
    is_style: bool
) {
    let mut effects: Vec<(EventDispatchType, Message)> = Vec::new();

    let current_element_tree = if let Some(current_element_tree) = reactive_tree.element_tree.as_ref() {
        current_element_tree
    } else {
        return;
    };

    let fiber: FiberNode = FiberNode {
        element: Some(current_element_tree.as_ref()),
        component: Some(reactive_tree.component_tree.as_ref().unwrap()),
    };

    let is_pointer_event = matches!(
        event,
        Message::CraftMessage(CraftMessage::PointerMovedEvent(_))
            | Message::CraftMessage(CraftMessage::PointerButtonEvent(_))
    );
    let is_ime_event = matches!(
        event,
        Message::CraftMessage(CraftMessage::ImeEvent(Ime::Enabled))
            | Message::CraftMessage(CraftMessage::ImeEvent(Ime::Disabled))
    );

    #[derive(Clone)]
    struct Target {
        component_id: ComponentId,
        element_id: Option<String>,
        layout_order: usize,
        overlay_depth: usize,
    }

    match dispatch_type {
        EventDispatchType::Bubbling => {
            let mut targets: VecDeque<Target> = VecDeque::new();
            let mut target_components: VecDeque<&ComponentTreeNode> = VecDeque::new();


            /////////////////////////////////////////
            // A,0                                 //
            //   /////////////////////////         //
            //   // B,1                 //         //
            //   //   ///////////       //         //
            //   //   //       //       //         //
            //   //   //  C,2  //       //         //
            //   //   //       //       //         //
            //   //   ///////////       //         //
            //   //                     //         //
            //   /////////////////////////         //
            //                                     //
            /////////////////////////////////////////

            // Collect all possible target elements in reverse order.
            // Nodes added last are usually on top, so these elements are in visual order.


            for (fiber_node, overlay_depth) in fiber.dfs_with_overlay_depth() {

                if let Some(element) = fiber_node.element {
                    let in_bounds = mouse_position.is_some() && element.in_bounds(mouse_position.unwrap());
                    let mut should_pass_hit_test = in_bounds;

                    // Bypass the hit test result if pointer capture is turned on for the current element.
                    if is_pointer_event || is_ime_event {
                        if let Some(element_id) = reactive_tree.pointer_captures.get(&DUMMY_DEVICE_ID) {
                            if *element_id == element.component_id() {
                                should_pass_hit_test = true;
                            }
                        }
                    }

                    if should_pass_hit_test {
                        // println!("CID: {:?}, depth: {:?}", element.component_id(), overlay_depth);

                        targets.push_back(Target {
                            component_id: element.component_id(),
                            element_id: element.get_id().clone(),
                            layout_order: element.element_data().layout_order as usize,
                            overlay_depth,
                        })
                    } else {
                        //println!("Not in bounds, Element: {:?}", element.get_id());
                    }
                }
            }

            // The targets should be [(2, Some(c)), (1, Some(b)), (0, Some(a))].

            if targets.is_empty() {
                return;
            }

            // The target is always the first node (2, Some(c)).
            let mut tmp_targets: Vec<Target> = targets.clone().into_iter().collect();
            tmp_targets.sort_by(|a, b| b.layout_order.cmp(&a.layout_order)); // Sort using the layout order. (u32)
            tmp_targets.sort_by(|a, b| b.overlay_depth.cmp(&a.overlay_depth)); // Sort using the overlay depth order. (u32)
            targets = VecDeque::from(tmp_targets);

            let target = targets[0].clone();
            let mut propagate = true;
            let mut prevent_defaults = false;
            for current_target in targets.iter() {
                if !propagate {
                    break;
                }

                // Get the element's component tree node.
                let current_target_component = reactive_tree
                    .component_tree
                    .as_ref()
                    .unwrap()
                    .pre_order_iter()
                    .find(|node| node.id == current_target.component_id)
                    .unwrap();

                // Search for the closest non-element ancestor.
                let mut closest_ancestor_component: Option<&ComponentTreeNode> = None;

                let mut to_visit = Some(current_target_component);
                while let Some(node) = to_visit {
                    if !to_visit.unwrap().is_element {
                        closest_ancestor_component = Some(node);
                        to_visit = None;
                    } else if node.parent_id.is_none() {
                        to_visit = None;
                    } else {
                        let parent_id = node.parent_id.unwrap();
                        to_visit = reactive_tree
                            .component_tree
                            .as_ref()
                            .unwrap()
                            .pre_order_iter()
                            .find(|node2| node2.id == parent_id);
                    }
                }

                // Dispatch the event to the element's component.
                if let Some(node) = closest_ancestor_component {
                    target_components.push_back(node);

                    let state = reactive_tree.user_state.storage.get_mut(&node.id).unwrap().as_mut();
                    let mut res = (node.update)(
                        state,
                        global_state,
                        node.props.clone(),
                        Event::new(event)
                            .current_target(current_target.element_id.clone())
                            .target(target.element_id.clone()),
                        window_context
                    );
                    effects.append(&mut res.effects);
                    propagate = propagate && res.propagate;
                    let element_state =
                        &mut reactive_tree.element_state.storage.get_mut(&current_target.component_id).unwrap().base;
                    match res.pointer_capture {
                        PointerCapture::None => {}
                        PointerCapture::Set => {
                            element_state.pointer_capture.insert(DUMMY_DEVICE_ID, true);
                        }
                        PointerCapture::Unset => {
                            element_state.pointer_capture.remove(&DUMMY_DEVICE_ID);
                        }
                    }
                    prevent_defaults = prevent_defaults || res.prevent_defaults;
                    if res.future.is_some() {
                        reactive_tree.update_queue.push_back(UpdateQueueEntry::new(
                            node.id,
                            node.update,
                            res,
                            node.props.clone(),
                        ));
                    }
                }
            }

            let mut element_events: VecDeque<(CraftMessage, Option<String>)> = VecDeque::new();

            for element_state in reactive_tree.element_state.storage.values_mut() {
                if let Message::CraftMessage(message) = &event {
                    match message {
                        CraftMessage::PointerMovedEvent(..) => {
                            element_state.base.hovered = false;
                        }
                        CraftMessage::PointerButtonEvent(pointer_button) => {
                            if pointer_button.button.mouse_button() == MouseButton::Left && pointer_button.state == ElementState::Released {
                                element_state.base.active = false;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Handle element events if prevent defaults was not set to true.
            //let mut first_element = true;
            if !prevent_defaults {
                for target in targets.iter() {
                    let mut propagate = true;
                    let mut prevent_defaults = false;

                    for element in current_element_tree.pre_order_iter().collect::<Vec<&dyn Element>>().iter().rev() {
                        if !propagate {
                            break;
                        }
                        if element.component_id() == target.component_id {
                            if let Message::CraftMessage(event) = event {
                                let res = element.on_event(
                                    event,
                                    &mut reactive_tree.element_state,
                                    text_context.as_mut().unwrap(),
                                    // first_element && is_style. For only the first element.
                                    is_style,
                                );
                                //first_element = false;

                                if let Some(result_message) = res.result_message {
                                    element_events.push_back((result_message, element.get_id().clone()));
                                }

                                propagate = propagate && res.propagate;
                                prevent_defaults = prevent_defaults || res.prevent_defaults;
                            }
                        }
                    }
                }
            }

            for (event, target_element_id) in element_events.iter() {
                let mut propagate = true;
                let mut prevent_defaults = false;
                for node in target_components.iter() {
                    if !propagate {
                        break;
                    }

                    let state = reactive_tree.user_state.storage.get_mut(&node.id).unwrap().as_mut();
                    let mut res = (node.update)(
                        state,
                        global_state,
                        node.props.clone(),
                        Event::new(&Message::CraftMessage(event.clone())).current_target(target_element_id.clone()),
                        window_context
                    );
                    effects.append(&mut res.effects);
                    propagate = propagate && res.propagate;
                    prevent_defaults = prevent_defaults || res.prevent_defaults;
                    if res.future.is_some() {
                        reactive_tree.update_queue.push_back(UpdateQueueEntry::new(
                            node.id,
                            node.update,
                            res,
                            node.props.clone(),
                        ));
                    }
                }
            }
        }
        EventDispatchType::Direct(id) => {
            for node in fiber.pre_order_iter().collect::<Vec<FiberNode>>().iter() {
                if let Some(element) = node.element {
                    if element.component_id() == id {
                        if let Message::CraftMessage(event) = event {
                            let mut res = element.on_event(
                                event,
                                &mut reactive_tree.element_state,
                                text_context.as_mut().unwrap(),
                                false,
                            );

                            effects.append(&mut res.effects);
                        }

                        return;
                    }
                }
                if let Some(component) = node.component {
                    if component.id == id {
                        let state = reactive_tree.user_state.storage.get_mut(&component.id).unwrap().as_mut();
                        let mut res = (component.update)(
                            state,
                            global_state,
                            component.props.clone(),
                            Event::new(event).current_target(None).target(None),
                            window_context
                        );
                        effects.append(&mut res.effects);
                        if res.future.is_some() {
                            reactive_tree.update_queue.push_back(UpdateQueueEntry::new(
                                component.id,
                                component.update,
                                res,
                                component.props.clone(),
                            ));
                        }

                        return;
                    }
                }
            }
        }
    }

    // Handle effects.
    for (dispatch_type, message) in effects.iter() {
        dispatch_event(
            message,
            *dispatch_type,
            _resource_manager,
            mouse_position,
            reactive_tree,
            global_state,
            text_context,
            window_context,
            false
        );
    }
}

async fn on_pointer_button(app: &mut Box<App>, pointer_button: PointerButton) {
    let event = CraftMessage::PointerButtonEvent(pointer_button);
    let message = Message::CraftMessage(event);

    app.mouse_position = Some(Point::new(pointer_button.position.x, pointer_button.position.y));

    let scale_factor = get_scale_factor(&app.window);
    let logical_mouse_position: LogicalPosition<f32> = LogicalPosition::from_physical(PhysicalPosition::new(pointer_button.position.x, pointer_button.position.y), scale_factor);
    app.window_context.mouse_position = Some(Point::new(logical_mouse_position.x, logical_mouse_position.y));
    
    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.user_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        true,
    );

    #[cfg(feature = "dev_tools")]
    dispatch_event(
        &message,
        EventDispatchType::Bubbling,
        &mut app.resource_manager,
        app.mouse_position,
        &mut app.dev_tree,
        &mut app.global_state,
        &mut app.text_context,
        &mut app.window_context,
        false,
    );

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_resume(app: &mut App, window: Arc<dyn Window>, renderer: Option<Box<dyn Renderer + Send>>) {
    if app.user_tree.element_tree.is_none() {
        reset_unique_element_id();
        //let new_view = app.app.view();
        //app.element_tree = Some(new_view);
    }

    app.setup_text_context();
    if renderer.is_some() {
        app.renderer = renderer;

        // We can't guarantee the order of events on wasm.
        // This ensures a resize is not missed if the renderer was not finished creating when resize is called.
        #[cfg(target_arch = "wasm32")]
        app.renderer
            .as_mut()
            .unwrap()
            .resize_surface(window.surface_size().width as f32, window.surface_size().height as f32);
    }

    app.window = Some(window.clone());
}

#[allow(clippy::too_many_arguments)]
async fn update_reactive_tree(
    component_spec_to_generate_tree: ComponentSpecification,
    reactive_tree: &mut ReactiveTree,
    global_state: &mut GlobalState,
    resource_manager: Arc<ResourceManager>,
    should_reload_fonts: &mut bool,
    text_context: &mut TextContext,
    scaling_factor: f64,
    resources_collected: &mut HashMap<ResourceIdentifier, bool>,
    window_context: &mut WindowContext
) {
    let window_element = Container::new().into();
    let old_component_tree = reactive_tree.component_tree.as_ref();

    let new_tree = {
        let span = span!(Level::INFO, "reactive tree diffing");
        let _enter = span.enter();
        diff_trees(
            component_spec_to_generate_tree.clone(),
            window_element,
            old_component_tree,
            &mut reactive_tree.user_state,
            global_state,
            &mut reactive_tree.element_state,
            *should_reload_fonts,
            text_context,
            scaling_factor,
            window_context,
        )
    };

    *should_reload_fonts = false;

    scan_view_for_resources(
        new_tree.element_tree.internal.as_ref(),
        &new_tree.component_tree,
        resource_manager.clone(),
        resources_collected,
    )
    .await;
    reactive_tree.element_tree = Some(new_tree.element_tree.internal);
    reactive_tree.component_tree = Some(new_tree.component_tree);
    reactive_tree.component_ids = new_tree.component_ids;
    reactive_tree.element_ids = new_tree.element_ids;
    reactive_tree.pointer_captures = new_tree.pointer_captures;
}

#[allow(clippy::too_many_arguments)]
async fn draw_reactive_tree(
    reactive_tree: &mut ReactiveTree,
    resource_manager: Arc<ResourceManager>,
    renderer: &mut Box<dyn Renderer + Send>,
    viewport_size: Size<f32>,
    origin: Point,
    text_context: &mut TextContext,
    scale_factor: f64,
    mouse_position: Option<Point>,
    window: Option<Arc<dyn Window>>,
) {
    let root = reactive_tree.element_tree.as_mut().unwrap();

    let mut root_size = viewport_size;

    // When we lay out the root element it scales up the values by the scale factor, so we need to scale it down here.
    // We do not want to scale the window size.
    {
        root_size.width /= scale_factor as f32;
        root_size.height /= scale_factor as f32;
    }

    style_root_element(root, root_size);

    let (mut taffy_tree, taffy_root) = {
        let span = span!(Level::INFO, "layout");
        let _enter = span.enter();
        layout(
            &mut reactive_tree.element_state,
            root_size.width,
            root_size.height,
            text_context,
            root.as_mut(),
            origin,
            resource_manager.clone(),
            scale_factor,
            mouse_position,
        )
    };

    let renderer = renderer.as_mut();

    {
        let span = span!(Level::INFO, "render");
        let _enter = span.enter();
        let mut render_list = RenderList::new();
        root.draw(
            &mut render_list,
            text_context,
            &mut taffy_tree,
            taffy_root,
            &mut reactive_tree.element_state,
            mouse_position,
            window,
        );
        renderer.sort_and_cull_render_list(&mut render_list);

        let window = Rectangle {
            x: 0.0,
            y: 0.0,
            width: renderer.surface_width(),
            height: renderer.surface_height(),
        };
        renderer.prepare_render_list(render_list, resource_manager, window);
    }
}

async fn on_request_redraw(app: &mut App, scale_factor: f64, surface_size: Size<f32>) {
    if app.text_context.is_none() {
        app.setup_text_context();
    }
    let text_context = app.text_context.as_mut().unwrap();

    let old_element_ids = app.user_tree.element_ids.clone();
    let old_component_ids = app.user_tree.component_ids.clone();
    update_reactive_tree(
        app.app.clone(),
        &mut app.user_tree,
        &mut app.global_state,
        app.resource_manager.clone(),
        &mut app.reload_fonts,
        text_context,
        scale_factor,
        &mut app.resources_collected,
        &mut app.window_context
    )
    .await;

    let window_context = &mut app.window_context;

    // Handle window requests:
    if let Some(window) = app.window.clone() {
        if let Some(requested_cursor) = &window_context.requested_cursor {
            window.set_cursor(requested_cursor.clone());
        };
        
        if let Some(requested_window_width) = window_context.requested_window_width {
            let _ = window.request_surface_size(winit::dpi::Size::Logical(LogicalSize::new(requested_window_width as f64, window_context.window_size.height as f64)));
        };
        
        if let Some(requested_window_height) = window_context.requested_window_height {
            let _ = window.request_surface_size(winit::dpi::Size::Logical(LogicalSize::new(window_context.window_size.width as f64, requested_window_height as f64)));
        };
        
        if let Some(requested_mouse_position_x) = window_context.requested_mouse_position_x {
            let mouse_y = window_context.requested_mouse_position_y.unwrap_or_default() as f64;
            let _ = window.set_cursor_position(winit::dpi::Position::Logical(LogicalPosition::new(requested_mouse_position_x as f64, mouse_y)));
        };
        
        if let Some(requested_mouse_position_y) = window_context.requested_mouse_position_y {
            let mouse_x = window_context.requested_mouse_position_x.unwrap_or_default() as f64;
            let _ = window.set_cursor_position(winit::dpi::Position::Logical(LogicalPosition::new(mouse_x, requested_mouse_position_y as f64)));
        };   
    }
    
    // Reset the requested values:
    window_context.reset();

    // Cleanup unmounted components and elements.
    app.user_tree.user_state.remove_unused_state(&old_component_ids, &app.user_tree.component_ids);
    app.user_tree.element_state.remove_unused_state(&old_element_ids, &app.user_tree.element_ids);

    if app.renderer.is_none() {
        return;
    }

    let renderer = app.renderer.as_mut().unwrap();

    cfg_if! {
        if #[cfg(feature = "dev_tools")] {
            let mut root_size = surface_size;
        } else {
            let root_size = surface_size;
        }
    }

    renderer.surface_set_clear_color(Color::WHITE);

    #[cfg(feature = "dev_tools")]
    {
        if app.is_dev_tools_open {
            let dev_tools_size = Size::new(350.0, root_size.height);
            root_size.width -= dev_tools_size.width;
        }
    }

    draw_reactive_tree(
        &mut app.user_tree,
        app.resource_manager.clone(),
        renderer,
        root_size,
        Point::new(0.0, 0.0),
        text_context,
        scale_factor,
        app.mouse_position,
        app.window.clone(),
    )
    .await;

    #[cfg(feature = "dev_tools")]
    {
        if app.is_dev_tools_open {
            update_reactive_tree(
                dev_tools_view(app.user_tree.element_tree.clone().unwrap()),
                &mut app.dev_tree,
                &mut app.global_state,
                app.resource_manager.clone(),
                &mut app.reload_fonts,
                text_context,
                scale_factor,
                &mut app.resources_collected,
                &mut app.window_context
            )
            .await;

            draw_reactive_tree(
                &mut app.dev_tree,
                app.resource_manager.clone(),
                renderer,
                Size::new(surface_size.width - root_size.width, root_size.height),
                Point::new(root_size.width, 0.0),
                text_context,
                scale_factor,
                app.mouse_position,
                app.window.clone(),
            )
            .await;
        }
    }

    renderer.submit(app.resource_manager.clone());
}

fn style_root_element(root: &mut Box<dyn Element>, root_size: Size<f32>) {
    *root.style_mut().width_mut() = Unit::Px(root_size.width);
    *root.style_mut().wrap_mut() = Wrap::Wrap;
    *root.style_mut().display_mut() = Display::Block;

    let is_user_root_height_auto = {
        let root_children = root.children_mut();
        root_children[0].internal.style().height().is_auto()
    };

    *root.style_mut().width_mut() = Unit::Px(root_size.width);
    *root.style_mut().wrap_mut() = Wrap::Wrap;
    *root.style_mut().display_mut() = Display::Block;

    if is_user_root_height_auto {
        *root.style_mut().height_mut() = Unit::Auto;
    } else {
        *root.style_mut().height_mut() = Unit::Px(root_size.height);
        *root.style_mut().height_mut() = Unit::Px(root_size.height);
    }
}

#[allow(clippy::too_many_arguments)]
fn layout(
    element_state: &mut ElementStateStore,
    window_width: f32,
    window_height: f32,
    text_context: &mut TextContext,
    root_element: &mut dyn Element,
    origin: Point,
    resource_manager: Arc<ResourceManager>,
    scale_factor: f64,
    pointer: Option<Point>,
) -> (TaffyTree<LayoutContext>, NodeId) {
    let mut taffy_tree: TaffyTree<LayoutContext> = TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, element_state, scale_factor).unwrap();

    let available_space: taffy::Size<AvailableSpace> = taffy::Size {
        width: AvailableSpace::Definite(window_width),
        height: AvailableSpace::Definite(window_height),
    };

    taffy_tree
        .compute_layout_with_measure(
            root_node,
            available_space,
            |known_dimensions, available_space, _node_id, node_context, style| {
                measure_content(
                    element_state,
                    known_dimensions,
                    available_space,
                    node_context,
                    text_context,
                    resource_manager.clone(),
                    style,
                )
            },
        )
        .unwrap();

    let transform = glam::Mat4::IDENTITY;

    let mut layout_order: u32 = 0;
    root_element.finalize_layout(
        &mut taffy_tree,
        root_node,
        origin,
        &mut layout_order,
        transform,
        element_state,
        pointer,
        text_context,
    );

    // root_element.print_tree();
    // taffy_tree.print_tree(root_node);

    (taffy_tree, root_node)
}
