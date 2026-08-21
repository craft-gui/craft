use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::time::Instant as JobInstant;
#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
use std::time::{Duration, Instant};
#[cfg(all(feature = "audio", target_arch = "wasm32"))]
use web_time::{Duration, Instant};

use retgui_logging::info;

use retgui_primitives::geometry::{Point, Size};

use retgui_resource_manager::resource_event::ResourceEvent;
use retgui_resource_manager::resource_type::ResourceType;
use retgui_resource_manager::{ResourceId, ResourceManager};

use gummy::NodeId;
use issho::AccessEvent;

use retgui_runtime::{Job, Receiver, RetGuiRuntimeHandle, Sender, pop_gui_thread_work, push_gui_thread_work};

use ui_events::keyboard::KeyboardEvent;
use ui_events::pointer::{PointerButtonEvent, PointerEvent, PointerScrollEvent, PointerUpdate};
use ui_events_winit::{WindowEventReducer, WindowEventTranslation};

use winit::event::{Ime, WindowEvent};
use winit::event_loop::ActiveEventLoop;

use crate::RetGuiOptions;
#[cfg(feature = "audio")]
use crate::elements::{AUDIO_CONTEXT, AudioInner};
use crate::elements::{ElementIdMap, ElementInternals, Window, scrollable};
use crate::events::internal::InternalMessage;
use crate::events::{Event, EventDispatcher, EventKind};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;
#[cfg(target_arch = "wasm32")]
use crate::wasm_queue::{WASM_QUEUE, WasmQueue};
use crate::window_manager::WindowManager;

thread_local! {
    pub(crate) static ELEMENTS: RefCell<ElementIdMap> = RefCell::new(ElementIdMap::new());
    pub(crate) static PENDING_RESOURCES: RefCell<VecDeque<(ResourceId, ResourceType)>> = const { RefCell::new(VecDeque::new()) };
    pub(crate) static IN_PROGRESS_RESOURCES: RefCell<VecDeque<(ResourceId, ResourceType)>> = const { RefCell::new(VecDeque::new()) };
    pub(crate) static FOCUS: RefCell<Option<Weak<RefCell<dyn ElementInternals>>>> = RefCell::new(None);
    pub(crate) static WINDOW_MANAGER: RefCell<WindowManager> = RefCell::new(WindowManager::new());
    pub(crate) static GUMMY_TREE: RefCell<GummyTree> = RefCell::new(GummyTree::new());
    /// An event queue that users or elements can manipulate. Cleared at the start and end of every event dispatch.
    static EVENT_DISPATCH_QUEUE: RefCell<VecDeque<(Event, EventKind)>> = RefCell::new(VecDeque::with_capacity(10));
}

/// Update interval for audio elements.
#[cfg(feature = "audio")]
const AUDIO_UI_UPDATE_INTERVAL: Duration = Duration::from_millis(250);

#[cfg(feature = "audio")]
fn audio_ui_update_due(last_update: Option<Instant>, now: Instant) -> bool {
    last_update.is_none_or(|last_update| now.duration_since(last_update) >= AUDIO_UI_UPDATE_INTERVAL)
}

pub struct App {
    pub(crate) event_reducer: WindowEventReducer,
    pub(crate) event_dispatcher: Rc<RefCell<EventDispatcher>>,
    /// The text context is used to manage fonts and text rendering. It is only valid between resume and pause.
    pub(crate) text_context: Rc<RefCell<Option<TextContext>>>,
    pub(crate) reload_fonts: bool,
    /// The resource manager is used to manage resources such as images and fonts.
    ///
    /// The resource manager is responsible for loading, caching, and providing access to resources.
    pub(crate) resource_manager: Arc<ResourceManager>,

    pub(crate) app_sender: Sender<InternalMessage>,
    #[allow(dead_code)]
    pub(crate) event_receiver: Receiver<InternalMessage>,
    pub(crate) runtime: RetGuiRuntimeHandle,

    pub(super) target_scratch: Vec<Rc<RefCell<dyn ElementInternals>>>,
    #[allow(dead_code)]
    pub(crate) retgui_options: RetGuiOptions,

    /// True if the winit app is active.
    pub(crate) active: bool,
    pub(crate) wait_cancelled: bool,
    pub(crate) close_requested: bool,

    #[cfg(feature = "audio")]
    pub(crate) last_audio_ui_update: Option<Instant>,
}

pub(crate) enum WindowEventResult {
    Continue,
    #[allow(dead_code)]
    ExitRequested,
}

impl App {
    /// Handle window events.
    pub(crate) fn on_window_event(&mut self, window: Window, event: WindowEvent) -> WindowEventResult {
        if !matches!(
            event,
            WindowEvent::KeyboardInput {
                is_synthetic: true,
                ..
            }
        ) {
            match self.event_reducer.reduce(window.effective_scale_factor(), &event) {
                Some(WindowEventTranslation::Keyboard(keyboard_event)) => {
                    self.on_keyboard_input(window, keyboard_event);
                    return WindowEventResult::Continue;
                }
                Some(WindowEventTranslation::Pointer(pointer_event)) => {
                    match pointer_event {
                        PointerEvent::Down(event) => self.on_pointer_button(window, event, false),
                        PointerEvent::Up(event) => self.on_pointer_button(window, event, true),
                        PointerEvent::Move(event) => self.on_pointer_moved(window, event),
                        PointerEvent::Scroll(event) => self.on_pointer_scroll(window, event),
                        PointerEvent::Cancel(_) | PointerEvent::Enter(_) | PointerEvent::Leave(_) => {}
                        PointerEvent::Gesture(_) => {}
                    }
                    return WindowEventResult::Continue;
                }
                _ => {}
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                return WINDOW_MANAGER.with_borrow_mut(|window_manager| {
                    window_manager.close_window(&window);
                    if window_manager.is_empty() {
                        self.on_close_requested();
                        self.close_requested = true;
                    }
                    WindowEventResult::Continue
                });
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.on_scale_factor_changed(window, scale_factor);
            }
            WindowEvent::Resized(new_size) => {
                self.on_resize(window, Size::new(new_size.width as f32, new_size.height as f32));
            }
            WindowEvent::Ime(ime) => self.on_ime(window, ime),
            WindowEvent::RedrawRequested => self.on_request_redraw(window),
            WindowEvent::Moved(_) => self.on_move(window),
            WindowEvent::Focused(focused) => window.on_focused(focused),
            _ => {}
        }

        WindowEventResult::Continue
    }

    pub fn on_close_requested(&mut self) {
        info!("RetGui application is closing.");
    }

    pub fn on_scale_factor_changed(&mut self, window: Window, scale_factor: f64) {
        window.on_scale_factor_changed(scale_factor);
    }

    pub fn on_resume(&mut self, event_loop: Option<&ActiveEventLoop>) {
        self.active = true;
        self.setup_text_context();

        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.on_resume(self, event_loop);
        });
    }

    pub fn on_about_to_wait(&mut self, event_loop: Option<&ActiveEventLoop>) {
        self.runtime.update_local_set();
        self.process_messages();
        self.process_external_work();
        if let Some(text_context) = self.text_context.borrow_mut().as_mut() {
            self.event_dispatcher.borrow_mut().dispatch_queued_events(text_context);
        }

        #[cfg(feature = "audio")]
        self.update_audio_ui();

        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.on_about_to_wait(self, event_loop);
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn process_messages(&mut self) {
        while let Ok(message) = self.event_receiver.try_recv() {
            match message {
                InternalMessage::ResourceEvent(resource_event) => {
                    self.on_resource_event(resource_event);
                }
                #[cfg(target_arch = "wasm32")]
                InternalMessage::RendererCreated(_, _) => unreachable!(),
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn process_messages(&mut self) {
        WASM_QUEUE.with_borrow_mut(|wasm_queue: &mut WasmQueue| {
            wasm_queue.drain(|message| match message {
                InternalMessage::ResourceEvent(resource_event) => {
                    self.on_resource_event(resource_event);
                }
                InternalMessage::RendererCreated(winit_window, renderer) => {
                    WINDOW_MANAGER.with_borrow_mut(|window_manager| {
                        let window = window_manager.get_window_by_id(winit_window.id()).unwrap();
                        let size = Size::new(
                            winit_window.inner_size().width as f32,
                            winit_window.inner_size().height as f32,
                        );
                        window.inner.borrow_mut().on_renderer_created(renderer, size);
                    });
                }
            });
        });
    }

    fn process_external_work(&mut self) {
        // Elements changed by scheduled work request their own redraws.
        let mut timer_jobs: Vec<Job> = vec![];
        while let Some(mut work) = pop_gui_thread_work() {
            if work.interval.is_none() || work.last_run.elapsed() >= work.interval.unwrap() {
                (work.callback)();
                work.last_run = JobInstant::now();
            }

            if work.interval.is_some() {
                timer_jobs.push(work);
            }
        }

        for timer_job in timer_jobs {
            push_gui_thread_work(timer_job);
        }
    }

    #[cfg(feature = "audio")]
    fn update_audio_ui(&mut self) {
        use std::any::Any;
        use std::ops::DerefMut;
        let now = Instant::now();
        if !audio_ui_update_due(self.last_audio_ui_update, now) {
            return;
        }
        self.last_audio_ui_update = Some(now);

        AUDIO_CONTEXT.with(|audio_context| {
            if let Some(ctx) = audio_context.get() {
                for audio_element in &ctx.borrow().sounds {
                    if let Some(audio) = ELEMENTS.with(|elements| elements.borrow().get(*audio_element).cloned()) {
                        let audio = audio.upgrade().unwrap();
                        let mut audio = audio.borrow_mut();
                        let audio: &mut AudioInner = (audio.deref_mut() as &mut dyn Any)
                            .downcast_mut()
                            .expect("Failed to downcast");
                        audio.update();
                    }
                }
            }
        });
    }

    pub fn on_suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.active = false;
    }

    /// Handles the window resize event.
    pub fn on_resize(&mut self, window: Window, new_size: Size<f32>) {
        window.on_resize(new_size);
    }

    /// Updates the tree, layouts the elements, and draws the view.
    pub fn on_request_redraw(&mut self, window: Window) {
        self.on_request_redraw_internal(window);
    }

    pub fn on_move(&mut self, _window: Window) {}

    pub fn on_pointer_scroll(&mut self, window: Window, pointer_scroll_update: PointerScrollEvent) {
        if window.inner.borrow_mut().maybe_zoom(&pointer_scroll_update) {
            return;
        }
        self.dispatch_event(window, &EventKind::PointerScroll(pointer_scroll_update));
    }

    pub fn on_pointer_button(&mut self, window: Window, pointer_event: PointerButtonEvent, is_up: bool) {
        let cursor_position = pointer_event.state.logical_point();

        let event = if is_up {
            EventKind::PointerButtonUp(pointer_event)
        } else {
            EventKind::PointerButtonDown(pointer_event)
        };
        window.set_mouse_position(Some(Point::new(cursor_position.x, cursor_position.y)));

        self.dispatch_event(window.clone(), &event);
    }

    pub fn on_pointer_moved(&mut self, window: Window, mouse_moved: PointerUpdate) {
        window.set_mouse_position(Some(mouse_moved.current.logical_point()));
        self.dispatch_event(window.clone(), &EventKind::PointerMovedEvent(mouse_moved));
    }

    pub fn on_ime(&mut self, window: Window, ime: Ime) {
        self.dispatch_event(window.clone(), &EventKind::ImeEvent(ime));
    }

    pub fn on_keyboard_input(&mut self, window: Window, keyboard_input: KeyboardEvent) {
        window.inner.borrow_mut().update_modifiers(&keyboard_input);
        if window.inner.borrow_mut().maybe_toggle_perf_stats(&keyboard_input) {
            return;
        }
        if window.inner.borrow_mut().maybe_zoom_keyboard(&keyboard_input) {
            return;
        }
        let prevent_defaults =
            self.dispatch_event(window.clone(), &EventKind::KeyboardInputEvent(keyboard_input.clone()));
        if !prevent_defaults {
            let navigated = window.inner.borrow_mut().maybe_navigate_tab(&keyboard_input);
            if navigated {
                let focused = FOCUS.with(|focus| focus.borrow().as_ref().and_then(Weak::upgrade));
                if let Some(focused) = focused {
                    scrollable::handle_accessibility_scroll_event(
                        &mut *focused.borrow_mut(),
                        &AccessEvent::ScrollIntoView,
                    );
                }
                window.request_redraw();
            }
        }
    }

    pub fn on_resource_event(&mut self, resource_event: ResourceEvent) {
        match resource_event {
            ResourceEvent::Loaded(resource_id, resource_type, resource) => {
                IN_PROGRESS_RESOURCES.with_borrow_mut(|in_progress| {
                    in_progress.retain_mut(|(resource, _resource_type)| *resource != resource_id);
                });
                if let Some(_text_context) = self.text_context.borrow_mut().as_mut()
                    && resource_type == ResourceType::Font
                {
                    // Todo: Load the font into the text context.
                    self.resource_manager.insert(resource_id.clone(), Arc::new(resource));
                    self.reload_fonts = true;
                } else {
                    self.resource_manager.insert(resource_id, Arc::new(resource));
                }
                // TODO: Only mark dirty affected nodes.
                WINDOW_MANAGER.with_borrow_mut(|window_manager| {
                    window_manager.dirty_and_redraw_all_windows(self);
                });
            }
            ResourceEvent::UnLoaded(_) => {}
        }
    }

    fn on_request_redraw_internal(&mut self, window: Window) {
        self.update_resources();
        window.on_redraw(
            self.text_context.borrow_mut().as_mut().unwrap(),
            self.resource_manager.clone(),
        );
    }

    fn dispatch_event(&mut self, window: Window, message: &EventKind) -> bool {
        let mouse_pos = window.mouse_position();
        let binding = window.inner.borrow().renderer.clone();
        let renderer = &mut *binding.borrow_mut();
        self.event_dispatcher.borrow_mut().dispatch_event(
            message,
            mouse_pos,
            window.inner.clone(),
            self.text_context.borrow_mut().as_mut().unwrap(),
            renderer,
            &mut self.target_scratch,
        )
    }

    fn update_resources(&mut self) {
        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            IN_PROGRESS_RESOURCES.with_borrow_mut(|in_progress| {
                for (resource, resource_type) in pending_resources.drain(..) {
                    if self.resource_manager.contains(&resource)
                        || in_progress.contains(&(resource.clone(), resource_type.clone()))
                    {
                        continue;
                    }
                    self.resource_manager
                        .async_download_resource_and_send_message_on_finish(
                            self.app_sender.clone(),
                            resource.clone(),
                            &resource_type,
                        );
                    in_progress.push_back((resource, resource_type));
                }
            });
        });
    }

    /// Initialize any data needed to layout/render text.
    fn setup_text_context(&mut self) {
        if self.text_context.borrow().is_none() {
            #[cfg(any(target_arch = "wasm32", not(feature = "system_fonts")))]
            let mut text_context = TextContext::new();
            #[cfg(all(not(target_arch = "wasm32"), feature = "system_fonts"))]
            let text_context = TextContext::new();

            #[cfg(any(target_arch = "wasm32", not(feature = "system_fonts")))]
            {
                let regular = include_bytes!("../../assets/fonts/Roboto-Regular.ttf");
                let bold = include_bytes!("../../assets/fonts/Roboto-Bold.ttf");
                let semi_bold = include_bytes!("../../assets/fonts/Roboto-SemiBold.ttf");
                let medium = include_bytes!("../../assets/fonts/Roboto-Medium.ttf");

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

            *self.text_context.borrow_mut() = Some(text_context);
        }
    }
}

#[cfg(all(test, feature = "audio"))]
mod tests {
    use super::{AUDIO_UI_UPDATE_INTERVAL, Duration, Instant, audio_ui_update_due};

    #[test]
    fn audio_ui_updates_are_throttled() {
        let first_update = Instant::now();

        assert!(audio_ui_update_due(None, first_update));
        assert!(!audio_ui_update_due(
            Some(first_update),
            first_update + AUDIO_UI_UPDATE_INTERVAL - Duration::from_millis(1),
        ));
        assert!(audio_ui_update_due(
            Some(first_update),
            first_update + AUDIO_UI_UPDATE_INTERVAL,
        ));
    }
}

/// Enqueues an event at the back of the dispatch queue.
pub fn queue_event(event: Event, message: EventKind) {
    EVENT_DISPATCH_QUEUE.with_borrow_mut(|event_queue| {
        event_queue.push_back((event, message));
    });
}

/// Pops from the front of the event dispatch queue and returns the result.
pub(crate) fn dequeue_event() -> Option<(Event, EventKind)> {
    EVENT_DISPATCH_QUEUE.with_borrow_mut(|event_queue| event_queue.pop_front())
}

#[inline]
pub fn request_layout(gummy_node: NodeId) {
    GUMMY_TREE.with_borrow_mut(|gummy_tree| {
        gummy_tree.mark_dirty(gummy_node);
    });
}

#[inline]
pub fn request_apply_layout(node: NodeId) {
    GUMMY_TREE.with_borrow_mut(|gummy_tree| {
        gummy_tree.request_apply_layout(node);
    });
}
