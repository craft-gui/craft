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

#[cfg(feature = "dev_tools")]
pub(crate) mod devtools;
pub mod geometry;
pub mod layout;
pub mod resource_manager;
mod view_introspection;

pub use craft_runtime::CraftRuntime;
pub use options::CraftOptions;
pub use renderer::color::palette;
pub use renderer::color::Color;

#[cfg(target_os = "android")]
pub use winit::platform::android::activity::*;

use crate::events::{CraftMessage, EventDispatchType};
pub use crate::options::RendererType;
use crate::reactive::element_state_store::ElementStateStore;
use crate::style::{Display, Unit, Wrap};
use components::component::{ComponentId, ComponentSpecification};
use elements::container::Container;
use elements::element::Element;
use events::internal::InternalMessage;
use events::resource_event::ResourceEvent;
use events::update_queue_entry::UpdateQueueEntry;
use events::Message;
use layout::layout_context::{measure_content, LayoutContext};
use reactive::element_id::reset_unique_element_id;
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

use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;
pub use winit::window::{Cursor, CursorIcon};

use std::any::Any;
use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use accesskit::{Role, TreeUpdate};
use cfg_if::cfg_if;
use craft_logging::{info, span, Level};
#[cfg(not(target_arch = "wasm32"))]
use std::time;
use accesskit_winit::Adapter;
use ui_events::keyboard::{KeyState, KeyboardEvent};
use ui_events::pointer::{PointerButtonUpdate, PointerScrollUpdate, PointerUpdate};
use winit::event::Ime;
#[cfg(target_os = "android")]
use {winit::event_loop::EventLoopBuilder, winit::platform::android::EventLoopBuilderExtAndroid};

#[cfg(target_arch = "wasm32")]
use {parley::GenericFamily, peniko::Blob};

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

#[derive(Debug, Clone)]
/// User-level API to get and set common window properties.
/// All values are in logical pixels.
pub struct WindowContext {
    scale_factor: f64,
    window_size: PhysicalSize<u32>,
    mouse_position: Option<Point>,
    cursor: Option<Cursor>,

    requested_window_width: Option<f32>,
    requested_window_height: Option<f32>,
    requested_mouse_position_x: Option<f32>,
    requested_mouse_position_y: Option<f32>,
    requested_cursor: Option<Cursor>,
}

impl WindowContext {
    pub(crate) fn apply_requests(&self, window: &Window) {
        if let Some(requested_cursor) = &self.requested_cursor {
            window.set_cursor(requested_cursor.clone());
        };

        if let Some(requested_window_width) = self.requested_window_width {
            let _ = window.request_inner_size(winit::dpi::Size::Logical(LogicalSize::new(
                requested_window_width as f64,
                self.window_size.height as f64,
            )));
        };

        if let Some(requested_window_height) = self.requested_window_height {
            let _ = window.request_inner_size(winit::dpi::Size::Logical(LogicalSize::new(
                self.window_size.width as f64,
                requested_window_height as f64,
            )));
        };

        if let Some(requested_mouse_position_x) = self.requested_mouse_position_x {
            let mouse_y = self.requested_mouse_position_y.unwrap_or_default() as f64;
            let _ = window.set_cursor_position(winit::dpi::Position::Logical(LogicalPosition::new(
                requested_mouse_position_x as f64,
                mouse_y,
            )));
        };

        if let Some(requested_mouse_position_y) = self.requested_mouse_position_y {
            let mouse_x = self.requested_mouse_position_x.unwrap_or_default() as f64;
            let _ = window.set_cursor_position(winit::dpi::Position::Logical(LogicalPosition::new(
                mouse_x,
                requested_mouse_position_y as f64,
            )));
        };
    }
}

impl WindowContext {
    pub(crate) fn new() -> WindowContext {
        Self {
            scale_factor: 1.0,
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

    pub fn cursor(&self) -> Option<&Cursor> {
        self.cursor.as_ref()
    }

    pub fn window_width(&self) -> f32 {
        self.window_size.to_logical(self.scale_factor).width
    }
    pub fn window_height(&self) -> f32 {
        self.window_size.to_logical(self.scale_factor).height
    }

    pub fn mouse_position_x(&self) -> Option<f32> {
        self.mouse_position.map(|pos| pos.x as f32)
    }

    pub fn mouse_position_y(&self) -> Option<f32> {
        self.mouse_position.map(|pos| pos.y as f32)
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
        self.requested_window_width = None;
        self.requested_window_height = None;
        self.requested_mouse_position_x = None;
        self.requested_mouse_position_y = None;
        self.requested_cursor = None;
    }
}

struct App {
    /// The user's view specification. This is lazily evaluated and will be called each time the view is redrawn.
    app: ComponentSpecification,
    /// The global state is used to store global data that can be accessed from anywhere in the user's application.
    global_state: GlobalState,
    /// A winit window. This is only valid between resume and pause.
    window: Option<Arc<Window>>,
    /// The text context is used to manage fonts and text rendering. It is only valid between resume and pause.
    text_context: Option<TextContext>,
    /// The renderer is used to draw the view. It is only valid between resume and pause.
    renderer: Option<RendererBox>,
    mouse_position: Option<Point>,
    reload_fonts: bool,
    /// The resource manager is used to manage resources such as images and fonts.
    ///
    /// The resource manager is responsible for loading, caching, and providing access to resources.
    resource_manager: Arc<ResourceManager>,
    /// Resources that have already been collected.
    /// We use this in view_introspection, so that we don't request the download
    /// of a resource too many times.
    resources_collected: HashMap<ResourceIdentifier, bool>,
    // The user's reactive tree.
    user_tree: ReactiveTree,
    /// Provides a way for the user to get and set common window properties during view and update.
    window_context: WindowContext,

    #[cfg(feature = "dev_tools")]
    is_dev_tools_open: bool,

    /// The dev tools tree is used to display the reactive tree in the dev tools.
    #[cfg(feature = "dev_tools")]
    dev_tree: ReactiveTree,
    app_sender: Sender<InternalMessage>,
    accesskit_adapter: Option<Adapter>,
    runtime: CraftRuntimeHandle,
}

#[cfg(not(target_arch = "wasm32"))]
type RendererBox = Box<dyn Renderer + Send>;
#[cfg(target_arch = "wasm32")]
type RendererBox = Box<dyn Renderer>;

impl App {

    fn on_close_requested(&mut self) {
        info!("Craft application is closing.");
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        self.window_context.scale_factor = scale_factor;
        self.on_resize(self.window.as_ref().unwrap().inner_size());
    }

    fn on_process_user_events(&mut self, is_dev_tree: bool) {
        let reactive_tree = if !is_dev_tree { &mut self.user_tree } else { &mut self.dev_tree };

        if reactive_tree.update_queue.is_empty() {
            return;
        }

        for event in reactive_tree.update_queue.drain(..) {
            let app_sender_copy = self.app_sender.clone();
            let f = async move {
                let update_result = event.update_result.unwrap();
                let res = update_result.await;
                app_sender_copy
                    .send(InternalMessage::GotUserMessage(InternalUserMessage {
                        update_fn: event.update_function,
                        source_component_id: event.source_component,
                        message: res,
                        props: event.props,
                    }))
                    .await
                    .expect("Failed to send user message");
            };
            self.runtime.spawn(f);
        }
    }

    fn on_resume(&mut self, window: Arc<Window>, renderer: RendererBox, event_loop: &ActiveEventLoop) {
        window.set_ime_allowed(true);

        if self.user_tree.element_tree.is_none() {
            reset_unique_element_id();
        }

        self.setup_text_context();
        self.renderer = Some(renderer);

        self.window = Some(window.clone());
        
        #[cfg(not(target_arch = "wasm32"))]
        let action_handler = CraftAccessHandler {
            runtime_handle: self.runtime.clone(),
            app_sender: self.app_sender.clone(),
        };
        let deactivation_handler = CraftDeactivationHandler::new();

        let scale_factor = window.scale_factor();

        self.window = Some(window.clone());
        self.window_context.scale_factor = scale_factor;
        self.on_resize(window.inner_size());
        let tree_update = self.on_request_redraw();

        let craft_activation_handler = CraftActivationHandler::new(tree_update);

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.accesskit_adapter = Some(Adapter::with_direct_handlers(
                event_loop,
                &window,
                craft_activation_handler,
                action_handler,
                deactivation_handler,
            ));
        }

        window.set_visible(true);
    }

    /// Handles the window resize event.
    fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_context.window_size = new_size;
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);
        }
        // On macOS the window needs to be redrawn manually after resizing
        #[cfg(target_os = "macos")]
        {
            self.wait_for_redraw();
        }
    }

    /// Initialize any data needed to layout/render text.
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
                text_context
                    .font_context
                    .collection
                    .append_generic_families(GenericFamily::SystemUi, fonts.iter().map(|f| f.0));
            }

            self.text_context = Some(text_context);
        }
    }

    /// Updates the view by applying the latest changes to the reactive tree.
    pub(crate) fn update_view(&mut self) {
        self.setup_text_context();
        let text_context = self.text_context.as_mut().unwrap();

        let old_element_ids = self.user_tree.element_ids.clone();
        let old_component_ids = self.user_tree.component_ids.clone();
        update_reactive_tree(
            self.app.clone(),
            &mut self.user_tree,
            &mut self.global_state,
            &mut self.reload_fonts,
            text_context,
            self.window_context.scale_factor,
            &mut self.window_context,
        );

        // Cleanup unmounted components and elements.
        self.user_tree.user_state.remove_unused_state(&old_component_ids, &self.user_tree.component_ids);
        self.user_tree.element_state.remove_unused_state(&old_element_ids, &self.user_tree.element_ids);
    }

    /// Updates the reactive tree, layouts the elements, and draws the view.
    fn on_request_redraw(&mut self) -> Option<TreeUpdate> {
        self.window.as_ref()?;

        let window = self.window.as_mut().unwrap().clone();

        let surface_size = Size::new(window.inner_size().width as f32, window.inner_size().height as f32);

        self.setup_text_context();

        self.update_view();

        cfg_if! {
            if #[cfg(feature = "dev_tools")] {
                let mut root_size = surface_size;
            } else {
                let root_size = surface_size;
            }
        }

        if self.renderer.is_some() {
            self.renderer.as_mut().unwrap().surface_set_clear_color(Color::WHITE);
        }

        #[cfg(feature = "dev_tools")]
        {
            if self.is_dev_tools_open {
                let dev_tools_size = Size::new(350.0, root_size.height);
                root_size.width -= dev_tools_size.width;
            }
        }

        {
            self.layout_tree(false, root_size, Point::new(0.0, 0.0), self.window_context.scale_factor, self.mouse_position);

            if self.renderer.is_some() {
                self.draw_reactive_tree(false, self.mouse_position, self.window.clone());
            }
        }

        #[cfg(feature = "dev_tools")]
        {
            if self.is_dev_tools_open {
                update_reactive_tree(
                    dev_tools_view(self.user_tree.element_tree.clone().unwrap()),
                    &mut self.dev_tree,
                    &mut self.global_state,
                    &mut self.reload_fonts,
                    self.text_context.as_mut().unwrap(),
                    self.window_context.scale_factor,
                    &mut self.window_context,
                );

                self.layout_tree(
                    true,
                    Size::new(surface_size.width - root_size.width, root_size.height),
                    Point::new(root_size.width as f64, 0.0),
                    self.window_context.scale_factor,
                    self.mouse_position,
                );

                if self.renderer.is_some() {
                    self.draw_reactive_tree(true, self.mouse_position, self.window.clone());
                }
            }
        }

        if self.renderer.is_some() {
            self.renderer.as_mut().unwrap().submit(self.resource_manager.clone());
        }

        if let Some(window) = &self.window {
            self.window_context.apply_requests(window);
            self.window_context.reset();
        }

        self.on_process_user_events(false);
        #[cfg(feature = "dev_tools")]
        {
            self.on_process_user_events(true);
        }

        self.view_introspection();

        let tree_update = self.compute_accessibility_tree();
        if let Some(accesskit_adapter) = &mut self.accesskit_adapter {
            accesskit_adapter.update_if_active(|| tree_update);
            window.pre_present_notify();
            None
        } else {
            window.pre_present_notify();
            Some(tree_update)
        }
    }

    fn on_pointer_scroll(&mut self, pointer_scroll_update: PointerScrollUpdate) {
        let event = CraftMessage::PointerScroll(pointer_scroll_update);
        let message = Message::CraftMessage(event);

        self.dispatch_event(&message, EventDispatchType::Bubbling, false);
        self.request_redraw();
    }

    fn on_pointer_button(&mut self, pointer_event: PointerButtonUpdate, is_up: bool, dispatch_type: EventDispatchType) {
        let cursor_position = pointer_event.state.position;

        let event = if is_up {
            CraftMessage::PointerButtonUp(pointer_event)
        } else {
            CraftMessage::PointerButtonDown(pointer_event)
        };
        let message = Message::CraftMessage(event);

        self.window_context.mouse_position = Some(Point::new(cursor_position.x, cursor_position.y));

        if let EventDispatchType::Direct(component) = dispatch_type {
            self.dispatch_event(&message, EventDispatchType::Direct(component), false);
        } else {
            self.dispatch_event(&message, EventDispatchType::Bubbling, true);
        }

        self.request_redraw();
    }

    fn on_pointer_moved(&mut self, mouse_moved: PointerUpdate) {
        self.mouse_position = Some(mouse_moved.current.position);
        self.window_context.mouse_position = Some(mouse_moved.current.position);

        let message = Message::CraftMessage(CraftMessage::PointerMovedEvent(mouse_moved));

        self.dispatch_event(&message, EventDispatchType::Bubbling, true);

        self.request_redraw();
    }

    fn on_ime(&mut self, ime: Ime) {
        let event = CraftMessage::ImeEvent(ime);
        let message = Message::CraftMessage(event);

        self.dispatch_event(&message, EventDispatchType::Bubbling, false);

        self.request_redraw();
    }

    /// Dispatch messages to the reactive tree.
    fn dispatch_event(&mut self, message: &Message, dispatch_type: EventDispatchType, is_style: bool) {
        dispatch_event(
            message,
            dispatch_type.clone(),
            &mut self.resource_manager,
            self.mouse_position,
            &mut self.user_tree,
            &mut self.global_state,
            &mut self.text_context,
            &mut self.window_context,
            is_style,
        );

        #[cfg(feature = "dev_tools")]
        dispatch_event(
            message,
            dispatch_type,
            &mut self.resource_manager,
            self.mouse_position,
            &mut self.dev_tree,
            &mut self.global_state,
            &mut self.text_context,
            &mut self.window_context,
            is_style,
        );
    }

    fn on_keyboard_input(&mut self, keyboard_input: KeyboardEvent) {
        let keyboard_event = CraftMessage::KeyboardInputEvent(keyboard_input.clone());
        let message = Message::CraftMessage(keyboard_event);

        self.dispatch_event(&message, EventDispatchType::Bubbling, false);

        #[cfg(feature = "dev_tools")]
        {
            let logical_key = keyboard_input.key;
            let key_state = keyboard_input.state;

            if KeyState::Down == key_state {
                if let ui_events::keyboard::Key::Named(ui_events::keyboard::NamedKey::F12) = logical_key {
                    self.is_dev_tools_open = !self.is_dev_tools_open;
                }
            }
        }

        self.request_redraw();
    }

    /// Processes async messages sent from the user.
    fn on_user_message(&mut self, message: InternalUserMessage) {
        let state = self.user_tree.user_state.storage.get_mut(&message.source_component_id).unwrap().as_mut();

        let mut event = Event::with_window_context(self.window_context.clone());

        (message.update_fn)(
            state,
            &mut self.global_state,
            message.props,
            &mut event,
            &Message::UserMessage(message.message),
        );
        self.window_context = event.window;
    }

    fn on_resource_event(&mut self, resource_event: ResourceEvent) {
        match resource_event {
            ResourceEvent::Loaded(resource_identifier, resource_type, resource) => {
                if resource_type == ResourceType::Font {
                    if let Some(_text_context) = self.text_context.as_mut() {
                        if resource.data().is_some() {
                            // Todo: Load the font into the text context.
                            self.resource_manager
                                .resources
                                .insert(resource_identifier.clone(), Arc::new(resource));
                        }
                    }

                    self.reload_fonts = true;
                } else if resource_type == ResourceType::Image || resource_type == ResourceType::TinyVg {
                    self.resource_manager.resources.insert(resource_identifier, Arc::new(resource));
                }
            }
            ResourceEvent::UnLoaded(_) => {}
        }
    }

    fn view_introspection(&mut self) {
        scan_view_for_resources(
            self.user_tree.element_tree.as_ref().unwrap().as_ref(),
            self.user_tree.component_tree.as_ref().unwrap(),
            self.resource_manager.clone(),
            &mut self.resources_collected,
        );
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn layout_tree(
        &mut self,
        is_dev_tree: bool,
        viewport_size: Size<f32>,
        origin: Point,
        scale_factor: f64,
        mouse_position: Option<Point>,
    ) {
        let reactive_tree = if !is_dev_tree { &mut self.user_tree } else { &mut self.dev_tree };
        let root_element = reactive_tree.element_tree.as_mut().unwrap();

        let mut root_size = viewport_size;

        // When we lay out the root element it scales up the values by the scale factor, so we need to scale it down here.
        // We do not want to scale the window size.
        {
            root_size.width /= scale_factor as f32;
            root_size.height /= scale_factor as f32;
        }

        style_root_element(root_element, root_size);
        let text_context = self.text_context.as_mut().unwrap();

        {
            let span = span!(Level::INFO, "layout");
            let _enter = span.enter();
            layout(
                &mut reactive_tree.element_state,
                root_size.width,
                root_size.height,
                text_context,
                root_element.as_mut(),
                origin,
                self.resource_manager.clone(),
                scale_factor,
                mouse_position,
            )
        };
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_reactive_tree(&mut self, is_dev_tree: bool, mouse_position: Option<Point>, window: Option<Arc<Window>>) {
        let reactive_tree = if !is_dev_tree { &mut self.user_tree } else { &mut self.dev_tree };
        let root_element = reactive_tree.element_tree.as_mut().unwrap();

        let text_context = self.text_context.as_mut().unwrap();
        {
            let span = span!(Level::INFO, "render");
            let _enter = span.enter();
            let mut render_list = RenderList::new();
            root_element.draw(&mut render_list, text_context, &mut reactive_tree.element_state, mouse_position, window);

            let renderer = self.renderer.as_mut().unwrap();
            renderer.sort_and_cull_render_list(&mut render_list);

            let window = Rectangle {
                x: 0.0,
                y: 0.0,
                width: renderer.surface_width(),
                height: renderer.surface_height(),
            };
            renderer.prepare_render_list(render_list, self.resource_manager.clone(), window);
        }
    }

    fn compute_accessibility_tree(&mut self) -> TreeUpdate {
        let tree = accesskit::Tree {
            root: accesskit::NodeId(0),
            toolkit_name: Some("Craft".to_string()),
            toolkit_version: None,
        };

        let mut tree_update = TreeUpdate {
            nodes: vec![],
            tree: Some(tree),
            focus: accesskit::NodeId(0),
        };

        let state = &mut self.user_tree.element_state;

        self.user_tree.element_tree.as_mut().unwrap().compute_accessibility_tree(&mut tree_update, None, state);
        tree_update.nodes[0].1.set_role(Role::Window);

        tree_update
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

use crate::components::Event;
use crate::craft_runtime::CraftRuntimeHandle;
use crate::events::event_dispatch::dispatch_event;
use crate::events::internal::InternalUserMessage;
use crate::geometry::{Point, Rectangle, Size};
use crate::reactive::state_store::{StateStore, StateStoreItem};
use crate::renderer::renderer::RenderList;
use crate::resource_manager::resource_type::ResourceType;
use crate::resource_manager::ResourceIdentifier;
use crate::text::text_context::TextContext;
use crate::view_introspection::scan_view_for_resources;
use craft_winit_state::CraftWinitState;
use crate::accessibility::access_handler::CraftAccessHandler;
use crate::accessibility::activation_handler::CraftActivationHandler;
use crate::accessibility::deactivation_handler::CraftDeactivationHandler;

#[cfg(target_arch = "wasm32")]
use crate::resource_manager::wasm_queue::WASM_QUEUE;

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
        runtime: runtime_copy,
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
                    winit_sender.send(InternalMessage::GotUserMessage(message)).await.expect("Failed to send user message");
                }
                InternalMessage::ResourceEvent(resource_event) => {
                    winit_sender.send(InternalMessage::ResourceEvent(resource_event)).await.expect("Failed to send resource event");
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_reactive_tree(
    component_spec_to_generate_tree: ComponentSpecification,
    reactive_tree: &mut ReactiveTree,
    global_state: &mut GlobalState,
    should_reload_fonts: &mut bool,
    text_context: &mut TextContext,
    scaling_factor: f64,
    window_context: &mut WindowContext,
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
            &mut reactive_tree.update_queue,
        )
    };

    *should_reload_fonts = false;

    reactive_tree.element_tree = Some(new_tree.element_tree.internal);
    reactive_tree.component_tree = Some(new_tree.component_tree);
    reactive_tree.component_ids = new_tree.component_ids;
    reactive_tree.element_ids = new_tree.element_ids;
    reactive_tree.pointer_captures = new_tree.pointer_captures;
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
        None,
    );

    // root_element.print_tree();
    // taffy_tree.print_tree(root_node);

    (taffy_tree, root_node)
}

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba8(r, g, b, a)
}
