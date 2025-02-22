pub mod accessibility;
pub mod components;
pub mod elements;
pub mod oku_runtime;
mod oku_winit_state;
mod options;
pub mod reactive;
pub mod style;
#[cfg(test)]
mod tests;
pub mod renderer;
pub mod events;

#[cfg(feature = "dev_tools")]
pub(crate) mod devtools;
pub mod app_message;
pub mod resource_manager;
pub mod geometry;
mod view_introspection;

pub use oku_runtime::OkuRuntime;
pub use renderer::color::Color;
pub use options::OkuOptions;

#[cfg(all(feature = "android", target_os = "android"))]
pub use winit::platform::android::activity::*;

use crate::reactive::element_state_store::ElementStateStore;
use crate::events::{Event, KeyboardInput, MouseWheel, OkuMessage, PointerButton, PointerMoved};
use reactive::element_id::reset_unique_element_id;
use reactive::fiber_node::FiberNode;
use events::update_queue_entry::UpdateQueueEntry;
use crate::style::{Display, Unit, Wrap};
use elements::container::Container;
use elements::element::Element;
use elements::layout_context::{measure_content, LayoutContext};
use events::Message;
use renderer::renderer::Renderer;
use reactive::tree::{diff_trees, ComponentTreeNode};
use components::component::{ComponentId, ComponentSpecification};
use app_message::AppMessage;
use events::resource_event::ResourceEvent;
pub use crate::options::RendererType;
use resource_manager::ResourceManager;
use events::internal::InternalMessage;

#[cfg(target_arch = "wasm32")]
use {std::cell::RefCell, web_time as time};

#[cfg(target_arch = "wasm32")]
thread_local! {
    pub static MESSAGE_QUEUE: RefCell<Vec<Message>> = RefCell::new(Vec::new());
}

type RendererBox = dyn Renderer;

use parley::FontContext;
use taffy::{AvailableSpace, NodeId, TaffyTree};

use tokio::sync::{RwLock, RwLockReadGuard};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::keyboard::{Key, NamedKey};



use std::any::Any;
use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::time;
use image::Rgba;
use peniko::Brush;
use peniko::color::palette;
use winit::event::DeviceId;
#[cfg(target_os = "android")]
use {winit::event_loop::EventLoopBuilder, winit::platform::android::EventLoopBuilderExtAndroid};
use oku_logging::{info, span, Level};

const WAIT_TIME: time::Duration = time::Duration::from_millis(15);

#[cfg(target_arch = "wasm32")]
pub type FutureAny = dyn Future<Output = Box<dyn Any>> + 'static;

#[cfg(not(target_arch = "wasm32"))]
pub type FutureAny = dyn Future<Output = Box<dyn Any + Send>> + 'static + Send;

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

struct App {
    app: ComponentSpecification,
    global_state: GlobalState,
    window: Option<Arc<dyn Window>>,
    font_context: Option<FontContext>,
    font_layout_context: Option<parley::LayoutContext<Brush>>,
    renderer: Option<Box<dyn Renderer + Send>>,
    mouse_position: Option<Point>,
    reload_fonts: bool,
    resource_manager: Arc<RwLock<ResourceManager>>,
    winit_sender: Sender<AppMessage>,

    user_tree: ReactiveTree,

    #[cfg(feature = "dev_tools")]
    is_dev_tools_open: bool,

    #[cfg(feature = "dev_tools")]
    dev_tree: ReactiveTree,
}

impl App {

    fn setup_font_context(&mut self) {
        if self.font_context.is_none() {
            #[allow(unused_mut)]
            let mut font_context = FontContext::new();
            self.font_context = Some(font_context);
            self.font_layout_context = Some(parley::LayoutContext::new());
        }
    }

}

#[cfg(target_os = "android")]
pub fn internal_oku_main_with_options(application: ComponentSpecification, global_state: Box<dyn Any>, options: Option<OkuOptions>, app: AndroidApp) {
    info!("Oku started");

    info!("Created winit event loop");

    let event_loop =
        EventLoopBuilder::default().with_android_app(app).build().expect("Failed to create winit event loop.");
    oku_main_with_options_2(event_loop, application, global_state, options)
}

#[cfg(feature = "dev_tools")]
use crate::devtools::dev_tools_component::dev_tools_view;

use crate::reactive::state_store::{StateStore, StateStoreItem};
use oku_winit_state::OkuWinitState;
use crate::elements::base_element_state::DUMMY_DEVICE_ID;
use crate::geometry::{Point, Size};
use crate::resource_manager::resource_type::ResourceType;
use crate::view_introspection::scan_view_for_resources;

pub type GlobalState = Box<dyn Any + Send + 'static>;

pub fn oku_main_with_options<T: Send + 'static>(application: ComponentSpecification, global_state: Box<T>, options: Option<OkuOptions>) {
    internal_oku_main_with_options(application, global_state, options);
}

#[cfg(not(target_os = "android"))]
pub fn internal_oku_main_with_options(application: ComponentSpecification, global_state: GlobalState, options: Option<OkuOptions>) {
    info!("Oku started");

    info!("Creating winit event loop.");

    let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
    info!("Created winit event loop.");

    oku_main_with_options_2(event_loop, application, global_state, options)
}

fn oku_main_with_options_2(
    event_loop: EventLoop,
    application: ComponentSpecification,
    global_state: GlobalState,
    oku_options: Option<OkuOptions>,
) {
    let oku_options = oku_options.unwrap_or_default();

    let runtime = OkuRuntime::new();
    info!("Created async runtime");

    let (app_sender, app_receiver) = channel::<AppMessage>(100);
    let (winit_sender, winit_receiver) = channel::<AppMessage>(100);
    let resource_manager = Arc::new(RwLock::new(ResourceManager::new(app_sender.clone())));

    let app_sender_copy = app_sender.clone();
    let resource_manager_copy = resource_manager.clone();

    let future = async_main(application, app_receiver, winit_sender, app_sender_copy, resource_manager_copy, global_state);

    runtime.runtime_spawn(future);

    let mut app = OkuWinitState::new(runtime, winit_receiver, app_sender, oku_options);

    event_loop.run_app(&mut app).expect("run_app failed");
}

async fn send_response(app_message: AppMessage, sender: &mut Sender<AppMessage>) {
    #[cfg(not(target_arch = "wasm32"))]
    if app_message.blocking {
        sender.send(AppMessage::new(app_message.id, InternalMessage::Confirmation)).await.expect("send failed");
    }
}

async fn async_main(
    component_spec_application: ComponentSpecification,
    mut app_receiver: Receiver<AppMessage>,
    winit_sender: Sender<AppMessage>,
    mut app_sender: Sender<AppMessage>,
    resource_manager: Arc<RwLock<ResourceManager>>,
    global_state: GlobalState
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
        font_context: None,
        font_layout_context: None,
        renderer: None,
        mouse_position: None,
        resource_manager,
        winit_sender: winit_sender.clone(),
        reload_fonts: false,
        user_tree: ReactiveTree {
            element_tree: None,
            component_tree: None,
            element_ids: Default::default(),
            component_ids: Default::default(),
            pointer_captures: Default::default(),
            update_queue: VecDeque::new(),
            user_state: user_state,
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
                    info!("Oku Closing");

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
                    update_fn(state, &mut app.global_state, props, Event::new(Message::UserMessage(message)));
                    app.window.as_ref().unwrap().request_redraw();
                }
                InternalMessage::ResourceEvent(resource_event) => {

                    let mut resource_manager = app.resource_manager.write().await;
                    
                    match resource_event {
                        ResourceEvent::Loaded(resource_identifier, resource_type, resource) => {
                            if resource_type == ResourceType::Font {
                                
                                    
                                if let Some(font_context) = app.font_context.as_mut() {
                                    if resource.data().is_some() {
                                        //font_context.db_mut().load_font_data(resource.data().unwrap().to_vec());
                                        resource_manager.resources.insert(resource_identifier.clone(), resource);
                                    }
                                }
                                
                                if let Some(renderer) = app.renderer.as_mut() {
                                    renderer.load_font(app.font_context.as_mut().unwrap());
                                }
                                
                                app.reload_fonts = true;
                                app.window.as_ref().unwrap().request_redraw();
                            } else if resource_type == ResourceType::Image {
                                resource_manager.resources.insert(resource_identifier, resource);
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
            }
        }
    }
}

fn on_process_user_events(window: Option<Arc<dyn Window>>, app_sender: &mut Sender<AppMessage>, reactive_tree: &mut ReactiveTree) {
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
        OkuRuntime::native_spawn(f);
    }
}

async fn on_pointer_moved(app: &mut Box<App>, mouse_moved: PointerMoved) {
    app.mouse_position = Some(Point::new(mouse_moved.position.x, mouse_moved.position.y));
    dispatch_event(OkuMessage::PointerMovedEvent(mouse_moved.clone()), &mut app.resource_manager, &mut app.font_context, app.mouse_position, &mut app.user_tree, &mut app.global_state).await;

    #[cfg(feature = "dev_tools")]
    dispatch_event(OkuMessage::PointerMovedEvent(mouse_moved), &mut app.resource_manager, &mut app.font_context, app.mouse_position, &mut app.dev_tree, &mut app.global_state).await;

    if let Some(window) = app.window.as_ref() {
        window.request_redraw();
    }
}

async fn on_mouse_wheel(app: &mut Box<App>, mouse_wheel: MouseWheel) {
    let event = OkuMessage::MouseWheelEvent(mouse_wheel);

    dispatch_event(event.clone(), &mut app.resource_manager, &mut app.font_context, app.mouse_position, &mut app.user_tree, &mut app.global_state).await;

    #[cfg(feature = "dev_tools")]
    dispatch_event(event, &mut app.resource_manager, &mut app.font_context, app.mouse_position, &mut app.dev_tree, &mut app.global_state).await;

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_keyboard_input(app: &mut Box<App>, keyboard_input: KeyboardInput) {
    let keyboard_event = OkuMessage::KeyboardInputEvent(keyboard_input.clone());

    dispatch_event(keyboard_event.clone(), &mut app.resource_manager, &mut app.font_context, app.mouse_position, &mut app.user_tree, &mut app.global_state).await;

    #[cfg(feature = "dev_tools")] {
        dispatch_event(keyboard_event.clone(), &mut app.resource_manager, &mut app.font_context, app.mouse_position, &mut app.dev_tree, &mut app.global_state).await;

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
    if let Some(renderer) = app.renderer.as_mut() {
        renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);
    }

    // On macOS the window needs to be redrawn manually after resizing
    #[cfg(target_os = "macos")]
    {
        app.window.as_ref().unwrap().request_redraw();
    }
}

async fn dispatch_event(event: OkuMessage, _resource_manager: &mut Arc<RwLock<ResourceManager>>, font_context: &mut Option<FontContext>, mouse_position: Option<Point>, reactive_tree: &mut ReactiveTree, global_state: &mut GlobalState) {
    let current_element_tree = if let Some(current_element_tree) = reactive_tree.element_tree.as_ref() {
        current_element_tree
    } else {
        return;
    };

    let fiber: FiberNode = FiberNode {
        element: Some(current_element_tree.as_ref()),
        component: Some(reactive_tree.component_tree.as_ref().unwrap()),
    };

    let is_pointer_event = matches!(event, OkuMessage::PointerMovedEvent(_) | OkuMessage::PointerButtonEvent(_));

    let mut targets: VecDeque<(ComponentId, Option<String>, u32)> = VecDeque::new();
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
    // Nodes added last are usually on top, so this elements in visual order.
    for fiber_node in fiber.level_order_iter().collect::<Vec<FiberNode>>().iter().rev() {
        if let Some(element) = fiber_node.element {
            let in_bounds = mouse_position.is_some() && element.in_bounds(mouse_position.unwrap());
            let mut should_pass_hit_test = in_bounds;
            
            // Bypass the hit test result if pointer capture is turned on for the current element.
            if is_pointer_event {
                if let Some(element_id) = reactive_tree.pointer_captures.get(&DUMMY_DEVICE_ID) {
                    if *element_id == element.component_id() {
                        should_pass_hit_test = true;
                    }
                }
            }
            
            if should_pass_hit_test {
                targets.push_back((element.component_id(), element.get_id().clone(), element.common_element_data().layout_order))
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

    let mut tmp_targets: Vec<(ComponentId, Option<String>, u32)> = targets.clone().into_iter().collect();
    tmp_targets.sort_by(|a, b| b.2.cmp(&a.2)); // Sort using the 3rd field (u32)
    targets = VecDeque::from(tmp_targets);

    let target = targets[0].clone();
    let (_target_component_id, target_element_id, _layout_order) = target.clone();
    let mut propagate = true;
    let mut prevent_defaults = false;
    for current_target in targets.iter() {
        if !propagate {
            break;
        }

        let (current_target_component_id, current_target_element_id, _layout_order) = current_target.clone();

        // Get the element's component tree node.
        let current_target_component =
            reactive_tree
            .component_tree
            .as_ref()
            .unwrap()
            .pre_order_iter()
            .find(|node| node.id == current_target_component_id)
            .unwrap();

        // Search for the closest non-element ancestor.
        let mut closest_ancestor_component: Option<&ComponentTreeNode> = None;

        let mut to_visit = Some(current_target_component);
        while let Some(node) = to_visit {
            if !to_visit.unwrap().is_element {
                closest_ancestor_component = Some(node);
                to_visit = None;
            } else {
                if node.parent_id.is_none() {
                    to_visit = None;
                } else {
                    let parent_id = node.parent_id.unwrap();
                    to_visit =
                        reactive_tree.component_tree.as_ref().unwrap().pre_order_iter().find(|node2| node2.id == parent_id);
                }
            }
        }

        // Dispatch the event to the element's component.
        if let Some(node) = closest_ancestor_component {
            target_components.push_back(node);

            let state = reactive_tree.user_state.storage.get_mut(&node.id).unwrap().as_mut();
            let res = (node.update)(
                state,
                global_state,
                node.props.clone(),
                Event::new(Message::OkuMessage(event.clone()))
                    .current_target(current_target_element_id.clone())
                    .target(target_element_id.clone()),
            );
            propagate = propagate && res.propagate;
            prevent_defaults = prevent_defaults || res.prevent_defaults;
            if res.future.is_some() {
                reactive_tree.update_queue.push_back(UpdateQueueEntry::new(node.id, node.update, res, node.props.clone()));
            }
        }
    }

    let mut element_events: VecDeque<(OkuMessage, Option<String>)> = VecDeque::new();

    // Handle element events if prevent defaults was not set to true.
    if !prevent_defaults {
        for target in targets.iter() {
            let (target_component_id, _target_element_id, _layout_order) = target.clone();

            let mut propagate = true;
            let mut prevent_defaults = false;

            for element in current_element_tree.pre_order_iter().collect::<Vec<&dyn Element>>().iter().rev() {
                if !propagate {
                    break;
                }
                if element.component_id() == target_component_id {
                    let res =
                        element.on_event(event.clone(), &mut reactive_tree.element_state, font_context.as_mut().unwrap());

                    if let Some(result_message) = res.result_message {
                        element_events.push_back((result_message, element.get_id().clone()));
                    }

                    propagate = propagate && res.propagate;
                    prevent_defaults = prevent_defaults || res.prevent_defaults;
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
            let res = (node.update)(
                state,
                global_state,
                node.props.clone(),
                Event::new(Message::OkuMessage(event.clone())).current_target(target_element_id.clone()),
            );
            propagate = propagate && res.propagate;
            prevent_defaults = prevent_defaults || res.prevent_defaults;
            if res.future.is_some() {
                reactive_tree.update_queue.push_back(UpdateQueueEntry::new(node.id, node.update, res, node.props.clone()));
            }
        }
    }
}

async fn on_pointer_button(app: &mut Box<App>, pointer_button: PointerButton) {
    let event = OkuMessage::PointerButtonEvent(pointer_button);

    app.mouse_position = Some(Point::new(pointer_button.position.x, pointer_button.position.y));
    dispatch_event(event.clone(), &mut app.resource_manager, &mut app.font_context, app.mouse_position, &mut app.user_tree, &mut app.global_state).await;

    #[cfg(feature = "dev_tools")]
    dispatch_event(event, &mut app.resource_manager, &mut app.font_context, app.mouse_position, &mut app.dev_tree, &mut app.global_state).await;

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_resume(app: &mut App, window: Arc<dyn Window>, renderer: Option<Box<dyn Renderer + Send>>) {
    if app.user_tree.element_tree.is_none() {
        reset_unique_element_id();
        //let new_view = app.app.view();
        //app.element_tree = Some(new_view);
    }

    app.setup_font_context();
    if renderer.is_some() {
        app.renderer = renderer;
        app.renderer.as_mut().unwrap().load_font(app.font_context.as_mut().unwrap());

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

async fn update_reactive_tree(
    component_spec_to_generate_tree: ComponentSpecification,
    reactive_tree: &mut ReactiveTree,
    global_state: &mut GlobalState,
    resource_manager: Arc<RwLock<ResourceManager>>,
    font_context: &mut FontContext,
    font_layout_context: &mut parley::LayoutContext<Brush>,
    should_reload_fonts: &mut bool
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
            font_context,
            *should_reload_fonts
        )
    };

    *should_reload_fonts = false;


    scan_view_for_resources(new_tree.element_tree.internal.as_ref(), &new_tree.component_tree, resource_manager.clone(), font_context).await;
    reactive_tree.element_tree = Some(new_tree.element_tree.internal);
    reactive_tree.component_tree = Some(new_tree.component_tree);
    reactive_tree.component_ids = new_tree.component_ids;
    reactive_tree.element_ids = new_tree.element_ids;
    reactive_tree.pointer_captures = new_tree.pointer_captures;
}

async fn draw_reactive_tree(
    reactive_tree: &mut ReactiveTree,
    resource_manager: Arc<RwLock<ResourceManager>>,
    renderer: &mut Box<dyn Renderer + Send>,
    viewport_size: Size,
    origin: Point,
    font_context: &mut FontContext,
    font_layout_context: &mut parley::LayoutContext<Brush>,
    scale_factor: f64,
    mouse_position: Option<Point>,
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

    let resource_manager = resource_manager.read().await;
    
    let (mut taffy_tree, taffy_root) = {
        let span = span!(Level::INFO, "layout");
        let _enter = span.enter();
        layout(
            &mut reactive_tree.element_state,
            root_size.width,
            root_size.height,
            font_context,
            font_layout_context,
            root.as_mut(),
            origin,
            &resource_manager,
            scale_factor,
            mouse_position
        )
    };

    let renderer = renderer.as_mut();

    {
        let span = span!(Level::INFO, "render");
        let _enter = span.enter();
        root.draw(renderer, font_context, &mut taffy_tree, taffy_root, &reactive_tree.element_state, mouse_position);
        renderer.prepare(resource_manager, font_context, &reactive_tree.element_state);
    }
}

async fn on_request_redraw(app: &mut App, scale_factor: f64, surface_size: Size) {
    if app.font_context.is_none() {
        app.setup_font_context();
    }
    let font_context = app.font_context.as_mut().unwrap();
    let font_layout_context = app.font_layout_context.as_mut().unwrap();

    let old_element_ids = app.user_tree.element_ids.clone();
    let old_component_ids = app.user_tree.component_ids.clone();
    update_reactive_tree(
        app.app.clone(),
        &mut app.user_tree,
        &mut app.global_state,
        app.resource_manager.clone(),
        font_context,
        font_layout_context,
        &mut app.reload_fonts
    ).await;

    // Cleanup unmounted components and elements.
    app.user_tree.user_state.remove_unused_state(&old_component_ids, &app.user_tree.component_ids);
    app.user_tree.element_state.remove_unused_state(&old_element_ids, &app.user_tree.element_ids);

    if app.renderer.is_none() {
        return;
    }

    let renderer = app.renderer.as_mut().unwrap();
    let mut root_size = surface_size;

    renderer.surface_set_clear_color(Color::WHITE);

    #[cfg(feature = "dev_tools")] {
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
        font_context,
        font_layout_context,
        scale_factor,
        app.mouse_position
    ).await;

    #[cfg(feature = "dev_tools")] {
        if app.is_dev_tools_open {
            update_reactive_tree(
                dev_tools_view(app.user_tree.element_tree.as_ref().unwrap()),
                &mut app.dev_tree,
                &mut app.global_state,
                app.resource_manager.clone(),
                font_context,
                font_layout_context,
                &mut app.reload_fonts
            ).await;
            
            draw_reactive_tree(
                &mut app.dev_tree,
                app.resource_manager.clone(),
                renderer,
                Size::new(surface_size.width - root_size.width, root_size.height),
                Point::new(root_size.width, 0.0),
                font_context,
                font_layout_context,
                scale_factor,
                app.mouse_position
            ).await;
        }
    }   
    
    let resource_manager = app.resource_manager.as_ref().read().await;
    renderer.submit(resource_manager);

}

fn style_root_element(root: &mut Box<dyn Element>, root_size: Size) {
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

fn layout<'a>(
    element_state: &mut ElementStateStore,
    _window_width: f32,
    _window_height: f32,
    font_context: &mut FontContext,
    font_layout_context: &mut parley::LayoutContext<Brush>,
    root_element: &mut dyn Element,
    origin: Point,
    resource_manager: &RwLockReadGuard<ResourceManager>,
    scale_factor: f64,
    pointer: Option<Point>,
) -> (TaffyTree<LayoutContext>, NodeId) {
    let mut taffy_tree: taffy::TaffyTree<LayoutContext> = taffy::TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, font_context, element_state, scale_factor).unwrap();

    let available_space: taffy::Size<taffy::AvailableSpace> = taffy::Size {
        width: AvailableSpace::Definite(_window_width),
        height: AvailableSpace::Definite(_window_height),
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
                    font_context,
                    font_layout_context,
                    resource_manager,
                    style
                )
            },
        )
        .unwrap();

    //taffy_tree.print_tree(root_node);

    let transform = glam::Mat4::IDENTITY;

    let mut layout_order: u32 = 0;
    root_element.finalize_layout(&mut taffy_tree, root_node, origin, &mut layout_order, transform, font_context, element_state, pointer);

    // root_element.print_tree();
    // taffy_tree.print_tree(root_node);

    (taffy_tree, root_node)
}
