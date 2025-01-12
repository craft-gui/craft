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

use cfg_if::cfg_if;

use cosmic_text::FontSystem;
use taffy::{AvailableSpace, NodeId, TaffyTree};

use tokio::sync::{RwLock, RwLockReadGuard};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::keyboard::{Key, NamedKey};



use std::any::Any;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::time;

#[cfg(target_os = "android")]
use {winit::event_loop::EventLoopBuilder, winit::platform::android::EventLoopBuilderExtAndroid};
use oku_logging::info;

const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

#[cfg(target_arch = "wasm32")]
pub type FutureAny = dyn Future<Output = Box<dyn Any>> + 'static;

#[cfg(not(target_arch = "wasm32"))]
pub type FutureAny = dyn Future<Output = Box<dyn Any + Send>> + 'static + Send;

pub type PinnedFutureAny = Pin<Box<FutureAny>>;

struct ReactiveTree {
    element_tree: Option<Box<dyn Element>>,
    component_tree: Option<ComponentTreeNode>,
    update_queue: VecDeque<UpdateQueueEntry>,
    user_state: StateStore,
    element_state: ElementStateStore,
}


struct App {
    app: ComponentSpecification,
    window: Option<Arc<dyn Window>>,
    font_system: Option<FontSystem>,
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

    fn setup_font_system(&mut self) {
        if self.font_system.is_none() {
            #[allow(unused_mut)]
            let mut font_system = FontSystem::new();
            
            #[cfg(target_arch = "wasm32")]
            {
                font_system.db_mut().load_font_data(include_bytes!("../../../fonts/FiraSans-Regular.ttf").to_vec());
                font_system.db_mut().load_font_data(include_bytes!("../../../fonts/FiraSans-Bold.ttf").to_vec());
                font_system.db_mut().load_font_data(include_bytes!("../../../fonts/FiraSans-Italic.ttf").to_vec());
            }

            #[cfg(target_os = "android")]
            {
                font_system.db_mut().load_fonts_dir("/system/fonts");
                font_system.db_mut().set_sans_serif_family("Roboto");
                font_system.db_mut().set_serif_family("Noto Serif");
                font_system.db_mut().set_monospace_family("Droid Sans Mono"); // Cutive Mono looks more printer-like
                font_system.db_mut().set_cursive_family("Dancing Script");
                font_system.db_mut().set_fantasy_family("Dancing Script");
            }

            self.font_system = Some(font_system);
        }
    }

}

#[cfg(target_os = "android")]
pub fn oku_main_with_options(application: ComponentSpecification, options: Option<OkuOptions>, app: AndroidApp) {
    info!("Oku started");

    info!("Created winit event loop");

    let event_loop =
        EventLoopBuilder::default().with_android_app(app).build().expect("Failed to create winit event loop.");
    oku_main_with_options_2(event_loop, application, options)
}

use crate::reactive::state_store::{StateStore, StateStoreItem};
use oku_winit_state::OkuWinitState;
use crate::devtools::dev_tools_component::dev_tools_view;
use crate::geometry::{Point, Size};
use crate::resource_manager::resource_type::ResourceType;
use crate::view_introspection::scan_view_for_resources;

#[cfg(not(target_os = "android"))]
pub fn oku_main_with_options(application: ComponentSpecification, options: Option<OkuOptions>) {
    info!("Oku started");

    info!("Creating winit event loop.");

    let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
    info!("Created winit event loop.");

    oku_main_with_options_2(event_loop, application, options)
}

fn oku_main_with_options_2(
    event_loop: EventLoop,
    application: ComponentSpecification,
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

    let future = async move {
        async_main(application, app_receiver, winit_sender, app_sender_copy, resource_manager_copy).await;
    };

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
           runtime.runtime_spawn(future);
        } else {
            runtime.runtime_spawn(future);
        }
    }

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
    application: ComponentSpecification,
    mut app_receiver: Receiver<AppMessage>,
    winit_sender: Sender<AppMessage>,
    mut app_sender: Sender<AppMessage>,
    resource_manager: Arc<RwLock<ResourceManager>>,
) {
    let mut user_state = StateStore::default();

    let dummy_root_value: Box<StateStoreItem> = Box::new(());
    user_state.storage.insert(0, dummy_root_value);

    let mut dev_tools_user_state = StateStore::default();
    dev_tools_user_state.storage.insert(0, Box::new(()));

    let mut app = Box::new(App {
        app: application,
        window: None,
        font_system: None,
        renderer: None,
        mouse_position: None,
        resource_manager,
        winit_sender: winit_sender.clone(),
        reload_fonts: false,
        user_tree: ReactiveTree {
            element_tree: None,
            component_tree: None,
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
                    update_fn(state, props, Event::new(Message::UserMessage(message)));
                    app.window.as_ref().unwrap().request_redraw();
                }
                InternalMessage::ResourceEvent(resource_event) => {
                    match resource_event {
                        ResourceEvent::Loaded(_resource_identifier, resource_type) => {
                            if resource_type == ResourceType::Font {
                                if let Some(renderer) = app.renderer.as_mut() {
                                    renderer.load_font(app.font_system.as_mut().unwrap());
                                }
                                app.reload_fonts = true;
                                app.window.as_ref().unwrap().request_redraw();
                            }

                            if resource_type == ResourceType::Image {
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
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(f);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            tokio::spawn(f);
        }
    }
}

async fn on_pointer_moved(app: &mut Box<App>, mouse_moved: PointerMoved) {
    app.mouse_position = Some(Point::new(mouse_moved.position.x as f32, mouse_moved.position.y as f32));
    dispatch_event(OkuMessage::PointerMovedEvent(mouse_moved.clone()), &mut app.resource_manager, &mut app.font_system, app.mouse_position, &mut app.user_tree).await;

    #[cfg(feature = "dev_tools")]
    dispatch_event(OkuMessage::PointerMovedEvent(mouse_moved), &mut app.resource_manager, &mut app.font_system, app.mouse_position, &mut app.dev_tree).await;

    if let Some(window) = app.window.as_ref() {
        window.request_redraw();
    }
}

async fn on_mouse_wheel(app: &mut Box<App>, mouse_wheel: MouseWheel) {
    let event = OkuMessage::MouseWheelEvent(mouse_wheel);

    dispatch_event(event.clone(), &mut app.resource_manager, &mut app.font_system, app.mouse_position, &mut app.user_tree).await;

    #[cfg(feature = "dev_tools")]
    dispatch_event(event, &mut app.resource_manager, &mut app.font_system, app.mouse_position, &mut app.dev_tree).await;

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_keyboard_input(app: &mut Box<App>, keyboard_input: KeyboardInput) {
    let keyboard_event = OkuMessage::KeyboardInputEvent(keyboard_input.clone());

    dispatch_event(keyboard_event.clone(), &mut app.resource_manager, &mut app.font_system, app.mouse_position, &mut app.user_tree).await;

    #[cfg(feature = "dev_tools")] {
        dispatch_event(keyboard_event.clone(), &mut app.resource_manager, &mut app.font_system, app.mouse_position, &mut app.dev_tree).await;

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

async fn dispatch_event(event: OkuMessage, _resource_manager: &mut Arc<RwLock<ResourceManager>>, font_system: &mut Option<FontSystem>, mouse_position: Option<Point>, reactive_tree: &mut ReactiveTree) {
    let current_element_tree = if let Some(current_element_tree) = reactive_tree.element_tree.as_ref() {
        current_element_tree
    } else {
        return;
    };

    let fiber: FiberNode = FiberNode {
        element: Some(current_element_tree.as_ref()),
        component: Some(reactive_tree.component_tree.as_ref().unwrap()),
    };

    // Dispatch some events globally to elements.
    // This is needed for things like scrolling while the mouse is not over an element.
    if matches!(event, OkuMessage::PointerMovedEvent(_) | OkuMessage::PointerButtonEvent(_)) {
        for fiber_node in fiber.level_order_iter().collect::<Vec<FiberNode>>().iter_mut().rev() {
            if let Some(element) = fiber_node.element {
                let res = element.on_event(event.clone(), &mut reactive_tree.element_state, font_system.as_mut().unwrap());

                if !res.propagate {
                    break;
                }
            }
        }
    }

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
            if in_bounds {
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
                        element.on_event(event.clone(), &mut reactive_tree.element_state, font_system.as_mut().unwrap());

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

    app.mouse_position = Some(Point::new(pointer_button.position.x as f32, pointer_button.position.y as f32));
    dispatch_event(event.clone(), &mut app.resource_manager, &mut app.font_system, app.mouse_position, &mut app.user_tree).await;

    #[cfg(feature = "dev_tools")]
    dispatch_event(event, &mut app.resource_manager, &mut app.font_system, app.mouse_position, &mut app.dev_tree).await;

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_resume(app: &mut App, window: Arc<dyn Window>, renderer: Option<Box<dyn Renderer + Send>>) {
    if app.user_tree.element_tree.is_none() {
        reset_unique_element_id();
        //let new_view = app.app.view();
        //app.element_tree = Some(new_view);
    }

    app.setup_font_system();
    if renderer.is_some() {
        app.renderer = renderer;
        app.renderer.as_mut().unwrap().load_font(app.font_system.as_mut().unwrap());

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
    resource_manager: Arc<RwLock<ResourceManager>>,
    font_system: &mut FontSystem,
    should_reload_fonts: &mut bool,
) {
    let window_element = Container::new().into();
    let old_component_tree = reactive_tree.component_tree.as_ref();

    let new_tree = diff_trees(
        component_spec_to_generate_tree.clone(),
        window_element,
        old_component_tree,
        &mut reactive_tree.user_state,
        &mut reactive_tree.element_state,
        font_system,
        *should_reload_fonts
    );

    *should_reload_fonts = false;


    scan_view_for_resources(new_tree.1.internal.as_ref(), &new_tree.0, resource_manager.clone(), font_system).await;
    reactive_tree.component_tree = Some(new_tree.0);
    reactive_tree.element_tree = Some(new_tree.1.internal);
}

async fn draw_reactive_tree(
    reactive_tree: &mut ReactiveTree,
    resource_manager: Arc<RwLock<ResourceManager>>,
    renderer: &mut Box<dyn Renderer + Send>,
    viewport_size: Size,
    origin: Point,
    font_system: &mut FontSystem,
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
    let (mut taffy_tree, taffy_root) = layout(
        &mut reactive_tree.element_state,
        root_size.width,
        root_size.height,
        font_system,
        root.as_mut(),
        origin,
        &resource_manager,
        scale_factor,
        mouse_position
    );

    let renderer = renderer.as_mut();
    root.draw(renderer, font_system, &mut taffy_tree, taffy_root, &reactive_tree.element_state, mouse_position);
    renderer.prepare(resource_manager, font_system, &reactive_tree.element_state);
}

async fn on_request_redraw(app: &mut App, scale_factor: f64, surface_size: Size) {
    if app.font_system.is_none() {
        app.setup_font_system();
    }
    let font_system = app.font_system.as_mut().unwrap();

    update_reactive_tree(
        app.app.clone(),
        &mut app.user_tree,
        app.resource_manager.clone(),
        font_system,
        &mut app.reload_fonts
    ).await;

    if app.renderer.is_none() {
        return;
    }

    let renderer = app.renderer.as_mut().unwrap();
    let mut root_size = surface_size;
    
    renderer.surface_set_clear_color(Color::rgba(255, 255, 255, 255));

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
        font_system,
        scale_factor,
        app.mouse_position
    ).await;

    #[cfg(feature = "dev_tools")] {
        update_reactive_tree(
            dev_tools_view(app.user_tree.element_tree.as_ref().unwrap()),
            &mut app.dev_tree,
            app.resource_manager.clone(),
            font_system,
            &mut app.reload_fonts
        ).await;

        if app.is_dev_tools_open {
            draw_reactive_tree(
                &mut app.dev_tree,
                app.resource_manager.clone(),
                renderer,
                Size::new(surface_size.width - root_size.width, root_size.height),
                Point::new(root_size.width, 0.0),
                font_system,
                scale_factor,
                app.mouse_position
            ).await;
        }
    }

    renderer.submit();

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
    font_system: &mut FontSystem,
    root_element: &mut dyn Element,
    origin: Point,
    resource_manager: &RwLockReadGuard<ResourceManager>,
    scale_factor: f64,
    pointer: Option<Point>,
) -> (TaffyTree<LayoutContext>, NodeId) {
    let mut taffy_tree: taffy::TaffyTree<LayoutContext> = taffy::TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, font_system, element_state, scale_factor).unwrap();

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
                    font_system,
                    resource_manager,
                    style
                )
            },
        )
        .unwrap();

    //taffy_tree.print_tree(root_node);

    let transform = glam::Mat4::IDENTITY;

    let mut layout_order: u32 = 0;
    root_element.finalize_layout(&mut taffy_tree, root_node, origin.x, origin.y, &mut layout_order, transform, font_system, element_state, pointer);

    // root_element.print_tree();
    // taffy_tree.print_tree(root_node);

    (taffy_tree, root_node)
}
