use crate::elements::{ElementIdMap, Window};
use crate::events::internal::InternalMessage;
use crate::events::CraftMessage;
use crate::events::{Event, EventDispatcher};
use crate::style::{Display, Unit, Wrap};
use crate::text::text_context::TextContext;
use craft_logging::info;
use craft_primitives::geometry::{Size, Point};
use craft_resource_manager::{ResourceIdentifier, ResourceManager};
use craft_runtime::CraftRuntimeHandle;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use std::sync::Arc;
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use {
    crate::accessibility::access_handler::CraftAccessHandler,
    crate::accessibility::activation_handler::CraftActivationHandler,
    crate::accessibility::deactivation_handler::CraftDeactivationHandler,
};
#[cfg(feature = "accesskit")]
use {
    accesskit::{Role, TreeUpdate},
    accesskit_winit::Adapter,
};

#[cfg(target_arch = "wasm32")]
use web_time as time;

use crate::animations::animation::AnimationFlags;
use crate::document::DocumentManager;
use crate::elements::Element;
use craft_resource_manager::resource_event::ResourceEvent;
use craft_resource_manager::resource_type::ResourceType;
use craft_runtime::Sender;
use std::time::Duration;
use ui_events::keyboard::{KeyboardEvent, Modifiers, NamedKey};
use ui_events::pointer::{PointerButtonEvent, PointerScrollEvent, PointerUpdate};
use ui_events::ScrollDelta;
use ui_events::ScrollDelta::PixelDelta;
use winit::event::Ime;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowId};
use crate::elements::core::ElementInternals;
use crate::window_manager::WindowManager;

thread_local! {
    /// The most recently recorded window id. This is set every time a windows event occurs.
    pub static CURRENT_WINDOW_ID : Cell<Option<WindowId>> = const { Cell::new(None) };
    /// Records document-level state (focus, pointer captures, etc.) for internal use.
    pub static DOCUMENTS: RefCell<DocumentManager> = RefCell::new(DocumentManager::new());
    pub(crate) static ELEMENTS: RefCell<ElementIdMap> = RefCell::new(ElementIdMap::new());
    pub(crate) static PENDING_RESOURCES: RefCell<VecDeque<(ResourceIdentifier, ResourceType)>> = RefCell::new(VecDeque::new());
    pub(crate) static IN_PROGRESS_RESOURCES: RefCell<VecDeque<(ResourceIdentifier, ResourceType)>> = RefCell::new(VecDeque::new());
    pub(crate) static FOCUS: RefCell<Option<Weak<RefCell<dyn Element>>>> = RefCell::new(None);
    /// An event queue that users or elements can manipulate. Cleared at the start and end of every event dispatch.
    static EVENT_DISPATCH_QUEUE: RefCell<VecDeque<(Event, CraftMessage)>> = RefCell::new(VecDeque::with_capacity(10));

    pub(crate) static WINDOW_MANAGER: RefCell<WindowManager> = RefCell::new(WindowManager::new());
}

/// Enqueues an event at the back of the dispatch queue.
///
/// This does **not** invoke any element `on_event` handlers.
/// Only user-registered event callbacks will be dispatched.
pub fn queue_event(event: Event, message: CraftMessage) {
    EVENT_DISPATCH_QUEUE.with_borrow_mut(|event_queue| {
        return event_queue.push_back((event, message));
    });
}

/// Pops from the front of the event dispatch queue and returns the result.
pub(crate) fn dequeue_event() -> Option<(Event, CraftMessage)> {
    let event = EVENT_DISPATCH_QUEUE.with_borrow_mut(|event_queue| {
        return event_queue.pop_front();
    });

    event
}

pub struct App {
    pub(crate) event_dispatcher: EventDispatcher,
    /// The text context is used to manage fonts and text rendering. It is only valid between resume and pause.
    pub(crate) text_context: Option<TextContext>,
    pub(crate) reload_fonts: bool,
    /// The resource manager is used to manage resources such as images and fonts.
    ///
    /// The resource manager is responsible for loading, caching, and providing access to resources.
    pub(crate) resource_manager: Arc<ResourceManager>,

    pub(crate) app_sender: Sender<InternalMessage>,
    #[cfg(feature = "accesskit")]
    pub(crate) accesskit_adapter: Option<Adapter>,
    #[allow(dead_code)]
    pub(crate) runtime: CraftRuntimeHandle,
    pub(crate) modifiers: Modifiers,
    pub redraw_flags: RedrawFlags,

    pub(super) target_scratch: Vec<Rc<RefCell<dyn Element>>>,

    pub(crate) previous_animation_flags: AnimationFlags,

    #[allow(dead_code)]
    pub(crate) focus: Option<Weak<RefCell<dyn Element>>>,
}

#[derive(Debug)]
pub struct RedrawFlags {
    rebuild_layout: bool,
}

impl RedrawFlags {
    pub fn new(rebuild_layout: bool) -> Self {
        Self { rebuild_layout }
    }

    pub fn should_rebuild_layout(&self) -> bool {
        self.rebuild_layout
    }
}

impl App {
    pub fn on_close_requested(&mut self) {
        info!("Craft application is closing.");
    }

    pub fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        /*self.window_context.scale_factor = scale_factor;
        self.on_resize(self.window.as_ref().unwrap().inner_size());
        self.root.borrow_mut().scale_factor(self.window_context.effective_scale_factor());
        style_root_element(self.root.borrow_mut().deref_mut(), self.window_context.window_size());*/
    }

    #[allow(unused_variables)]
    pub fn on_resume(&mut self, event_loop: &ActiveEventLoop) {
        //window.set_ime_allowed(true);

        self.setup_text_context();

        #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        let action_handler = CraftAccessHandler {
            runtime_handle: self.runtime.clone(),
            app_sender: self.app_sender.clone(),
        };
        #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        let deactivation_handler = CraftDeactivationHandler::new();

        /*let scale_factor = window.scale_factor();

        self.window = Some(window.clone());
        self.window_context.scale_factor = scale_factor;
        self.on_resize(window.inner_size());
        let tree_update = self.on_request_redraw();*/

        /*#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        let craft_activation_handler = CraftActivationHandler::new(tree_update);*/

        //#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
        //{
        //    self.accesskit_adapter = Some(Adapter::with_direct_handlers(
        //        event_loop,
        //        &window,
        //        craft_activation_handler,
        //        #[cfg(feature = "accesskit")]
        //        action_handler,
        //        deactivation_handler,
        //    ));
        //}
    }

    /// Handles the window resize event.
    pub fn on_resize(&mut self, window: Rc<RefCell<Window>>, new_size: Size<f32>) {
        window.borrow_mut().on_resize(new_size);
    }

    /// Initialize any data needed to layout/render text.
    fn setup_text_context(&mut self) {
        if self.text_context.is_none() {
            #[cfg(any(target_arch = "wasm32", not(feature = "system_fonts")))]
            let mut text_context = TextContext::new();
            #[cfg(all(not(target_arch = "wasm32"), feature = "system_fonts"))]
            let text_context = TextContext::new();

            #[cfg(any(target_arch = "wasm32", not(feature = "system_fonts")))]
            {
                let regular = include_bytes!("../../../fonts/Roboto-Regular.ttf");
                let bold = include_bytes!("../../../fonts/Roboto-Bold.ttf");
                let semi_bold = include_bytes!("../../../fonts/Roboto-SemiBold.ttf");
                let medium = include_bytes!("../../../fonts/Roboto-Medium.ttf");

                fn register_and_append(font_data: &'static [u8], text_context: &mut TextContext) {
                    let blob = peniko::Blob::new(Arc::new(font_data));
                    let fonts = text_context.font_context.collection.register_fonts(blob, None);

                    // Register all the Roboto families under parley::GenericFamily::SystemUi.
                    // This will become the fallback font for platforms like WASM.
                    text_context
                        .font_context
                        .collection
                        .append_generic_families(parley::GenericFamily::SystemUi, fonts.iter().map(|f| f.0));
                }

                register_and_append(regular, &mut text_context);
                register_and_append(bold, &mut text_context);
                register_and_append(semi_bold, &mut text_context);
                register_and_append(medium, &mut text_context);
            }

            self.text_context = Some(text_context);
        }
    }

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(feature = "accesskit")]
    pub fn on_request_redraw(&mut self, window: Rc<RefCell<Window>>) -> Option<TreeUpdate> {
        self.on_request_redraw_internal(window.clone());
        let winit_window = window.borrow().winit_window().as_mut().unwrap().clone();

        let tree_update = self.compute_accessibility_tree();
        if let Some(accesskit_adapter) = &mut self.accesskit_adapter {
            accesskit_adapter.update_if_active(|| tree_update);
            winit_window.pre_present_notify();
            None
        } else {
            winit_window.pre_present_notify();
            Some(tree_update)
        }
    }

    /// Updates the reactive tree, layouts the elements, and draws the view.
    #[cfg(not(feature = "accesskit"))]
    pub fn on_request_redraw(&mut self, window: Rc<RefCell<Window>>) {
        self.on_request_redraw_internal(window);
    }

    //#[cfg(not(feature = "accesskit"))]

    fn on_request_redraw_internal(&mut self, window: Rc<RefCell<Window>>) {
        self.update_resources();

        window.borrow_mut().on_redraw(self.text_context.as_mut().unwrap(), self.resource_manager.clone());
    }

    pub fn on_pointer_scroll(&mut self, window: Rc<RefCell<Window>>, pointer_scroll_update: PointerScrollEvent) {
        if self.modifiers.ctrl() && pointer_scroll_update.pointer.pointer_type == ui_events::pointer::PointerType::Mouse
        {
            let y: f32 = match pointer_scroll_update.delta {
                ScrollDelta::PageDelta(_, y) => y,
                ScrollDelta::LineDelta(_, y) => y,
                PixelDelta(physical) => physical.y as f32,
            };
            if y < 0.0 {
                window.borrow_mut().zoom_out();
            } else {
                window.borrow_mut().zoom_in();
            }
            let scale_factor = window.borrow().effective_scale_factor();
            window.borrow_mut().scale_factor(scale_factor);
            window.borrow_mut().mark_dirty();
            //style_root_element(window.borrow_mut().deref_mut(), window.borrow().window_size());
            self.request_redraw(RedrawFlags::new(true));
            return;
        }

        let event = CraftMessage::PointerScroll(pointer_scroll_update);
        let message = event;

        self.dispatch_event(window, &message, false);
        self.request_redraw(RedrawFlags::new(true));
    }

    pub fn on_pointer_button(&mut self, window: Rc<RefCell<Window>>, pointer_event: PointerButtonEvent, is_up: bool) {
        let mut pointer_event = pointer_event;
        let zoom = window.borrow().zoom_scale_factor();
        pointer_event.state.position.x /= zoom;
        pointer_event.state.position.y /= zoom;

        let cursor_position = pointer_event.state.position;

        let event = if is_up {
            CraftMessage::PointerButtonUp(pointer_event)
        } else {
            CraftMessage::PointerButtonDown(pointer_event)
        };
        let message = event;
        window.borrow_mut().set_mouse_position(Some(Point::new(cursor_position.x, cursor_position.y)));

        self.dispatch_event(window.clone(), &message, true);

        self.request_redraw(RedrawFlags::new(true));
    }

    pub fn on_pointer_moved(&mut self, window: Rc<RefCell<Window>>, mouse_moved: PointerUpdate) {
        let mut mouse_moved = mouse_moved;
        let zoom = window.borrow().zoom_scale_factor();
        mouse_moved.current.position.x /= zoom;
        mouse_moved.current.position.y /= zoom;

        window.borrow_mut().set_mouse_position(Some(mouse_moved.current.logical_point()));

        let message = CraftMessage::PointerMovedEvent(mouse_moved);

        self.dispatch_event(window.clone(), &message, true);

        self.request_redraw(RedrawFlags::new(true));
    }

    pub fn on_ime(&mut self, window: Rc<RefCell<Window>>, ime: Ime) {
        let event = CraftMessage::ImeEvent(ime);
        let message = event;

        self.dispatch_event(window.clone(), &message, false);

        self.request_redraw(RedrawFlags::new(true));
    }

    fn dispatch_event(&mut self, window: Rc<RefCell<Window>>, message: &CraftMessage, _is_style: bool) {
        let mouse_pos = Some(window.borrow().mouse_position());
        let render_list = window.borrow().render_list.clone();
        self.event_dispatcher.dispatch_event(
            message,
            mouse_pos.unwrap_or_default(),
            window.clone(),
            &mut self.text_context,
            render_list.borrow_mut().deref_mut(),
            &mut self.target_scratch,
        );
        window.borrow().winit_window().unwrap().request_redraw();
    }

    pub fn on_keyboard_input(&mut self, window: Rc<RefCell<Window>>, keyboard_input: KeyboardEvent) {
        self.modifiers = keyboard_input.modifiers;
        if keyboard_input.key == ui_events::keyboard::Key::Named(NamedKey::Control) && keyboard_input.state.is_up() {
            self.modifiers.set(Modifiers::CONTROL, false);
        }
        if keyboard_input.modifiers.ctrl() {
            if keyboard_input.key == ui_events::keyboard::Key::Character("=".to_string()) {
                window.borrow_mut().zoom_in();
                self.request_redraw(RedrawFlags::new(true));
                return;
            } else if keyboard_input.key == ui_events::keyboard::Key::Character("-".to_string()) {
                window.borrow_mut().zoom_out();
                self.request_redraw(RedrawFlags::new(true));
                return;
            }
        }

        let keyboard_event = CraftMessage::KeyboardInputEvent(keyboard_input.clone());
        let message = keyboard_event;

        self.dispatch_event(window.clone(), &message, false);

        self.request_redraw(RedrawFlags::new(true));
    }

    pub fn on_resource_event(&mut self, resource_event: ResourceEvent) {
        match resource_event {
            ResourceEvent::Loaded(resource_identifier, resource_type, resource) => {
                IN_PROGRESS_RESOURCES.with_borrow_mut(|in_progress| {
                    in_progress.retain_mut(|(resource, _resource_type)| *resource != resource_identifier);
                });
                if let Some(_text_context) = self.text_context.as_mut()
                    && resource_type == ResourceType::Font
                    && resource.data().is_some()
                {
                    // Todo: Load the font into the text context.
                    self.resource_manager.insert(resource_identifier.clone(), Arc::new(resource));
                    self.reload_fonts = true;
                } else if resource_type == ResourceType::Image || resource_type == ResourceType::TinyVg {
                    self.resource_manager.insert(resource_identifier, Arc::new(resource));
                }
            }
            ResourceEvent::UnLoaded(_) => {}
        }
    }

    fn request_redraw(&mut self, redraw_flags: RedrawFlags) {
        self.redraw_flags = redraw_flags;
        /*if let Some(window) = &self.window {
            window.request_redraw();
        }*/
    }

    /// "Animates" a tree by calling `on_animation_frame` and changing an element's styles.
    /*#[allow(dead_code)]
    fn animate_tree(&mut self, delta_time: &Duration, layout_origin: Point, viewport_size: LogicalSize<f32>) {
        /*let span = span!(Level::INFO, "animate_tree");
        let _enter = span.enter();*/

        let old_has_active_animation = self.previous_animation_flags.has_active_animation();
        let root_element = self.root.clone();

        // Damage track across recursive calls to `on_animation_frame`.
        let mut animation_flags = AnimationFlags::default();
        root_element.borrow_mut().on_animation_frame(&mut animation_flags, *delta_time);
        self.previous_animation_flags = animation_flags;

        // Perform a relayout if an animation used any layout effecting style property.
        if animation_flags.needs_relayout() || old_has_active_animation {
            root_element.borrow_mut().reset_layout_item();

            self.layout_tree(
                viewport_size,
                layout_origin,
                self.window_context.effective_scale_factor(),
                self.window_context.mouse_position,
            );
        }

        // Request a redraw if there is at least one animation playing.
        // ControlFlow::Poll is set in `about_to_wait`.
        if animation_flags.has_active_animation() || old_has_active_animation {
            // Winit does not guarantee when a redraw event will happen, but that should be fine, at worst we redraw an extra time.
            self.request_redraw(RedrawFlags::new(old_has_active_animation));
        }
    }*/

    #[cfg(feature = "accesskit")]
    fn compute_accessibility_tree(&mut self) -> TreeUpdate {
        let tree = accesskit::Tree {
            root: accesskit::NodeId(0),
            toolkit_name: Some("Craft".to_string()),
            toolkit_version: None,
        };

        let focus_id = self.focus.clone().map(|node| node.upgrade().unwrap().borrow().id()).unwrap_or(0);
        let mut tree_update = TreeUpdate {
            nodes: vec![],
            tree: Some(tree),
            focus: accesskit::NodeId(focus_id),
        };

        /*self.root.borrow_mut().compute_accessibility_tree(
            &mut tree_update,
            None,
            self.window_context.effective_scale_factor(),
        );*/
        tree_update.nodes[0].1.set_role(Role::Window);

        tree_update
    }

    fn update_resources(&mut self) {
        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            IN_PROGRESS_RESOURCES.with_borrow_mut(|in_progress| {
                for (resource, resource_type) in pending_resources.drain(..) {
                    if self.resource_manager.contains(&resource)
                        || in_progress.contains(&(resource.clone(), resource_type))
                    {
                        continue;
                    }
                    self.resource_manager.async_download_resource_and_send_message_on_finish(
                        self.app_sender.clone(),
                        resource.clone(),
                        resource_type,
                    );
                    in_progress.push_back((resource, resource_type));
                }
            });
        });
    }
}