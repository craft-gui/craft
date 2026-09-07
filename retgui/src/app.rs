use std::any::Any;
use std::collections::VecDeque;
use std::future::Future;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use issho::AccessEvent;

use parley::FontContext;

use peniko::Blob;

use retgui_logging::info;

use retgui_primitives::geometry::{Point, Size};

#[cfg(target_arch = "wasm32")]
use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::resource_type::ResourceType;
use retgui_resource_manager::{ResourceError, ResourceId, ResourceManager};

use retgui_runtime::RetGuiRuntime;
#[cfg(not(target_arch = "wasm32"))]
use retgui_runtime::task::yield_now;

use rustc_hash::FxHashMap;

use slotmap::{DefaultKey, SlotMap};

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, PointerKind, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::RetGuiError;
use crate::accessibility::RetGuiAccessTree;
#[cfg(feature = "audio")]
use crate::elements::audio::AudioContext;
use crate::elements::gui_actions::GuiActionQueue;
use crate::elements::internal_helpers::queue_animation_update;
use crate::elements::{AnimationSchedule, DynElement, ElementData, ElementInternals, RetainedElements, State, Window, WindowElement, scrollable, set_focus_outline_visible};
use crate::events::{EventDispatcher, EventKind, ImeEvent, KeyboardEvent, PointerButtonEvent, PointerInfo, PointerMovedEvent, PointerScrollEvent, PointerState};
use crate::layout::GummyTree;
use crate::text::text_context::{TextContext, create_font_context};
use crate::window_manager::WindowManager;

#[cfg(target_arch = "wasm32")]
pub(crate) struct CreatedRenderer {
    pub(crate) window: DynElement,
    pub(crate) renderer: Box<dyn Renderer>,
    pub(crate) size: Size<f32>,
}

pub struct App {
    pub(crate) event_dispatcher: EventDispatcher,
    /// Shared font and text-layout state.
    pub(crate) text_context: TextContext,
    pub(crate) font_context: FontContext,
    pub(crate) elements: RetainedElements,
    pub(crate) states: SlotMap<DefaultKey, Box<dyn Any>>,
    pub(crate) by_internal_id: FxHashMap<u64, DynElement>,
    pub(crate) access_tree: RetGuiAccessTree,
    pub(crate) gummy_tree: GummyTree,
    pub(crate) window_manager: WindowManager,
    pub(crate) pending_resources: VecDeque<(ResourceId, ResourceType)>,
    pub(crate) event_queue: VecDeque<EventKind>,
    pub(crate) pending_animation_updates: Vec<(DynElement, bool)>,
    pub(crate) focus: Option<DynElement>,
    pub(crate) focus_outline_visible: bool,
    #[cfg(feature = "audio")]
    pub(crate) audio_context: Option<AudioContext>,
    pub(crate) gui_actions: GuiActionQueue,
    in_progress_resources: VecDeque<(ResourceId, ResourceType)>,
    /// The resource manager is used to manage resources such as images and fonts.
    ///
    /// The resource manager is responsible for loading, caching, and providing access to resources.
    pub(crate) resource_manager: Arc<ResourceManager>,

    pub(crate) runtime: RetGuiRuntime,

    #[cfg(target_arch = "wasm32")]
    pub(crate) created_renderer_sender: Sender<CreatedRenderer>,
    #[cfg(target_arch = "wasm32")]
    created_renderer_receiver: Receiver<CreatedRenderer>,

    pub(super) target_scratch: Vec<DynElement>,

    /// True if the winit app is active.
    pub(crate) active: bool,
    pub(crate) close_requested: bool,
}

/// The action requested after dispatching a window event to [`App`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub enum WindowEventResult {
    Continue,
    ExitRequested,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let runtime = RetGuiRuntime::new();
        info!("Created async runtime");

        #[allow(clippy::arc_with_non_send_sync)]
        let resource_manager = Arc::new(ResourceManager::new());
        let font_context = create_font_context();
        let text_context = TextContext {
            font_context: font_context.clone(),
            layout_context: Default::default(),
        };
        #[cfg(target_arch = "wasm32")]
        let (created_renderer_sender, created_renderer_receiver) = channel();

        Self {
            event_dispatcher: EventDispatcher::new(),
            text_context,
            font_context,
            elements: RetainedElements::new(),
            states: SlotMap::with_key(),
            by_internal_id: FxHashMap::default(),
            access_tree: RetGuiAccessTree::new(),
            gummy_tree: GummyTree::new(),
            window_manager: WindowManager::new(),
            pending_resources: VecDeque::new(),
            event_queue: VecDeque::with_capacity(10),
            pending_animation_updates: Vec::new(),
            focus: None,
            focus_outline_visible: true,
            #[cfg(feature = "audio")]
            audio_context: None,
            gui_actions: GuiActionQueue::new(),
            in_progress_resources: VecDeque::new(),
            resource_manager,
            runtime,
            #[cfg(target_arch = "wasm32")]
            created_renderer_sender,
            #[cfg(target_arch = "wasm32")]
            created_renderer_receiver,
            target_scratch: Vec::new(),
            active: false,
            close_requested: false,
        }
    }

    /// Handle window events.
    pub fn on_window_event(&mut self, window: Window, event: WindowEvent) -> WindowEventResult {
        if matches!(
            &event,
            WindowEvent::KeyboardInput {
                is_synthetic: true,
                ..
            }
        ) {
            return WindowEventResult::Continue;
        }

        match event {
            WindowEvent::KeyboardInput { event, .. } => self.on_keyboard_input(window, event),
            WindowEvent::ModifiersChanged(modifiers) => {
                self.elements
                    .get_as_mut::<WindowElement>(window.inner)
                    .update_modifiers(modifiers.state());
            }
            WindowEvent::PointerMoved {
                position,
                primary,
                source,
                ..
            } => {
                let scale_factor = self
                    .elements
                    .get_as::<WindowElement>(window.inner)
                    .effective_scale_factor();
                let pointer = PointerInfo::from_source(&source, primary);
                self.on_pointer_moved(window, pointer, PointerState::new(position, scale_factor));
            }
            WindowEvent::PointerButton {
                state,
                position,
                primary,
                button,
                ..
            } => {
                let scale_factor = self
                    .elements
                    .get_as::<WindowElement>(window.inner)
                    .effective_scale_factor();
                let pointer = PointerInfo::from_button(&button, primary);
                self.on_pointer_button(
                    window,
                    button.clone().mouse_button(),
                    pointer,
                    PointerState::new(position, scale_factor),
                    state == ElementState::Released,
                );
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scale_factor = self
                    .elements
                    .get_as::<WindowElement>(window.inner)
                    .effective_scale_factor();
                let logical_position = self
                    .elements
                    .get_as::<WindowElement>(window.inner)
                    .mouse_position()
                    .unwrap_or_default();
                let physical_position =
                    PhysicalPosition::new(logical_position.x * scale_factor, logical_position.y * scale_factor);
                self.on_pointer_scroll(
                    window,
                    PointerInfo::new(PointerKind::Mouse, true),
                    delta,
                    PointerState::new(physical_position, scale_factor),
                );
            }
            WindowEvent::CloseRequested => {
                self.window_manager.close_window(&mut self.elements, &window);
                if self.window_manager.is_empty() {
                    self.on_close_requested();
                    self.close_requested = true;
                }
                return WindowEventResult::Continue;
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.on_scale_factor_changed(window, scale_factor);
            }
            WindowEvent::SurfaceResized(new_size) => {
                self.on_resize(window, Size::new(new_size.width as f32, new_size.height as f32));
            }
            WindowEvent::Ime(ime) => self.on_ime(window, ime),
            WindowEvent::RedrawRequested => self.on_request_redraw(window),
            WindowEvent::Moved(_) => self.on_move(window),
            WindowEvent::Focused(focused) => {
                if !focused {
                    self.elements.get_as_mut::<WindowElement>(window.inner).ime_composing = false;
                }
                self.elements
                    .get_as::<WindowElement>(window.inner)
                    .on_focused(&self.elements, self.focus, focused);
            }
            _ => {}
        }

        WindowEventResult::Continue
    }

    pub fn on_close_requested(&mut self) {
        info!("RetGui application is closing.");
    }

    pub fn on_scale_factor_changed(&mut self, window: Window, scale_factor: f64) {
        self.elements.dispatch_mut(window.inner, |window, elements| {
            (window as &mut dyn Any)
                .downcast_mut::<WindowElement>()
                .unwrap()
                .on_scale_factor_changed(elements, &mut self.gummy_tree, scale_factor);
        });
    }

    pub fn on_resume(&mut self, event_loop: Option<&dyn ActiveEventLoop>) {
        self.active = true;

        self.window_manager.on_resume(
            &mut self.elements,
            &mut self.gummy_tree,
            &self.states,
            &mut self.text_context,
            &self.resource_manager,
            &mut self.runtime,
            &mut self.pending_animation_updates,
            #[cfg(target_arch = "wasm32")]
            &self.created_renderer_sender,
            event_loop,
        );
    }

    pub fn on_about_to_wait(&mut self, event_loop: Option<&dyn ActiveEventLoop>) -> Option<Duration> {
        #[cfg(not(target_arch = "wasm32"))]
        self.block_on(async {
            yield_now().await;
        });
        let runtime_handle = self.runtime.handle();
        runtime_handle.update_local_set();
        {
            #[cfg(not(target_arch = "wasm32"))]
            let mut runtime_handle = runtime_handle;
            #[cfg(not(target_arch = "wasm32"))]
            let _runtime_context = runtime_handle.tokio_runtime_mut().enter();
            self.run_gui_actions();
        }
        #[cfg(target_arch = "wasm32")]
        self.process_created_renderers();
        self.process_resources();
        self.process_accessibility_events();
        EventDispatcher::dispatch_queued_events(self);
        self.window_manager.on_about_to_wait(
            &mut self.elements,
            &mut self.gummy_tree,
            &self.states,
            &mut self.text_context,
            &self.resource_manager,
            &mut self.runtime,
            &mut self.pending_animation_updates,
            #[cfg(target_arch = "wasm32")]
            &self.created_renderer_sender,
            self.active,
            event_loop,
        )
    }

    #[cfg(target_arch = "wasm32")]
    fn process_created_renderers(&mut self) {
        while let Ok(created) = self.created_renderer_receiver.try_recv() {
            if self.elements.contains(created.window) {
                self.elements
                    .get_as_mut::<WindowElement>(created.window)
                    .on_renderer_created(&mut self.gummy_tree, created.renderer, created.size);
            }
        }
    }

    fn process_accessibility_events(&mut self) {
        let tree = self.access_tree.clone();
        while let Some((target, event)) = tree.pop_event() {
            if !self.elements.contains(target) {
                continue;
            }
            let _ = self.elements.dispatch_mut(target, |target, elements| {
                scrollable::handle_accessibility_scroll_event(elements, &mut self.event_queue, target, &event);
                target.on_access_event(elements, &mut self.event_queue, &mut self.states, event)
            });
        }
    }

    fn process_resources(&mut self) {
        let mut resource_loaded = false;
        self.in_progress_resources.retain(|(resource_id, _resource_type)| {
            if !self.resource_manager.contains(resource_id) {
                return true;
            }

            resource_loaded = true;
            false
        });

        if resource_loaded {
            self.window_manager
                .dirty_and_redraw_all_windows(&self.elements, &mut self.gummy_tree, self.active);
        }
    }

    /// Suspends rendering until the application is resumed again.
    pub fn on_suspended(&mut self) {
        self.active = false;
    }

    /// Returns the window associated with a winit window ID.
    pub fn window_by_id(&self, window_id: WindowId) -> Option<Window> {
        self.window_manager.get_window_by_id(&self.elements, window_id)
    }

    /// Returns whether the driver should exit its event loop.
    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    /// Installs the driver callback used to wake a sleeping GUI event loop when
    /// asynchronous work schedules an element-store action.
    pub fn set_gui_waker(&self, waker: impl Fn() + Send + Sync + 'static) {
        self.gui_actions.set_waker(waker);
    }

    /// Handles the window resize event.
    pub fn on_resize(&mut self, window: Window, new_size: Size<f32>) {
        self.elements
            .get_as_mut::<WindowElement>(window.inner)
            .on_resize(&mut self.gummy_tree, new_size);
    }

    /// Updates the tree, layouts the elements, and draws the view.
    pub fn on_request_redraw(&mut self, window: Window) {
        self.on_request_redraw_internal(window);
    }

    pub fn on_move(&mut self, _window: Window) {}

    pub fn on_pointer_scroll(
        &mut self,
        window: Window,
        pointer: PointerInfo,
        delta: MouseScrollDelta,
        state: PointerState,
    ) {
        let pointer_scroll_update = PointerScrollEvent::new(DynElement::new(window.inner), pointer, delta, state);
        let zoomed = self.elements.dispatch_mut(window.inner, |window, elements| {
            (window as &mut dyn Any)
                .downcast_mut::<WindowElement>()
                .unwrap()
                .maybe_zoom(elements, &mut self.gummy_tree, &pointer_scroll_update)
        });
        if zoomed {
            return;
        }
        self.dispatch_event(window, EventKind::PointerScroll(pointer_scroll_update));
    }

    pub fn on_pointer_button(
        &mut self,
        window: Window,
        button: Option<MouseButton>,
        pointer: PointerInfo,
        state: PointerState,
        is_up: bool,
    ) {
        if !is_up {
            set_focus_outline_visible(&mut self.elements, self.focus, &mut self.focus_outline_visible, false);
        }

        let cursor_position = state.logical_point();
        let pointer_event = PointerButtonEvent::new(DynElement::new(window.inner), button, pointer, state);

        let event = if is_up {
            EventKind::PointerUp(pointer_event)
        } else {
            EventKind::PointerDown(pointer_event)
        };
        self.elements
            .get_as_mut::<WindowElement>(window.inner)
            .set_mouse_position(Some(Point::new(cursor_position.x, cursor_position.y)));

        self.dispatch_event(window, event);
    }

    pub fn on_pointer_moved(&mut self, window: Window, pointer: PointerInfo, state: PointerState) {
        self.elements
            .get_as_mut::<WindowElement>(window.inner)
            .set_mouse_position(Some(state.logical_point()));
        let event = PointerMovedEvent::new(DynElement::new(window.inner), pointer, state);
        self.dispatch_event(window, EventKind::PointerMoved(event));
    }

    pub fn on_ime(&mut self, window: Window, ime: Ime) {
        if let Some(is_composing) = match &ime {
            Ime::Preedit(text, _) => Some(!text.is_empty()),
            Ime::Enabled | Ime::Commit(_) | Ime::Disabled => Some(false),
            _ => None,
        } {
            self.elements.get_as_mut::<WindowElement>(window.inner).ime_composing = is_composing;
        }
        let event = ImeEvent::new(DynElement::new(window.inner), ime);
        self.dispatch_event(window, EventKind::Ime(event));
    }

    pub fn on_keyboard_input(&mut self, window: Window, keyboard_input: KeyEvent) {
        let (modifiers, is_composing) = {
            let window = self.elements.get_as::<WindowElement>(window.inner);
            (window.modifiers, window.ime_composing)
        };
        let state = keyboard_input.state;
        if state == ElementState::Pressed && !modifiers.control_key() && !modifiers.alt_key() && !modifiers.meta_key() {
            set_focus_outline_visible(&mut self.elements, self.focus, &mut self.focus_outline_visible, true);
        }
        let event = KeyboardEvent::new(DynElement::new(window.inner), keyboard_input, modifiers, is_composing);
        let toggled = self.elements.dispatch_mut(window.inner, |window, elements| {
            (window as &mut dyn Any)
                .downcast_mut::<WindowElement>()
                .unwrap()
                .maybe_toggle_perf_stats(elements, &mut self.gummy_tree, &event)
        });
        if toggled {
            return;
        }
        let zoomed = self.elements.dispatch_mut(window.inner, |window, elements| {
            (window as &mut dyn Any)
                .downcast_mut::<WindowElement>()
                .unwrap()
                .maybe_zoom_keyboard(elements, &mut self.gummy_tree, &event)
        });
        if zoomed {
            return;
        }
        let navigation_target = self
            .elements
            .get_as::<WindowElement>(window.inner)
            .tab_navigation_target(&self.elements, self.focus, &event);
        let event = match state {
            ElementState::Pressed => EventKind::KeyDown(event),
            ElementState::Released => EventKind::KeyUp(event),
        };
        let prevent_defaults = self.dispatch_event(window, event);
        if !prevent_defaults && let Some(target) = navigation_target {
            self.elements.dispatch_mut(target, |target, elements| {
                target.focus(
                    elements,
                    &mut self.event_queue,
                    &mut self.focus,
                    self.focus_outline_visible,
                );
                scrollable::handle_accessibility_scroll_event(
                    elements,
                    &mut self.event_queue,
                    target,
                    &AccessEvent::ScrollIntoView,
                );
            });
            self.elements.get_as::<WindowElement>(window.inner).request_redraw();
        }
    }

    fn on_request_redraw_internal(&mut self, window: Window) {
        let animation_schedule = self.window_manager.animation_tick(
            &mut self.elements,
            &mut self.gummy_tree,
            &mut self.states,
            &mut self.pending_resources,
            &mut self.pending_animation_updates,
            &window,
        );
        self.update_resources();
        WindowElement::on_request_redraw(
            &mut self.elements,
            &mut self.gummy_tree,
            &self.states,
            &mut self.text_context,
            self.resource_manager.clone(),
            window.inner,
        );

        if animation_schedule == AnimationSchedule::NextFrame
            && self
                .elements
                .get_as::<WindowElement>(window.inner)
                .winit_window()
                .is_some()
        {
            self.elements.get_as::<WindowElement>(window.inner).request_redraw();
        }
    }

    fn dispatch_event(&mut self, window: Window, mut event: EventKind) -> bool {
        let mouse_pos = self.elements.get_as::<WindowElement>(window.inner).mouse_position();
        EventDispatcher::dispatch_event(&mut event, mouse_pos, window.inner, self)
    }

    fn update_resources(&mut self) {
        for (resource, resource_type) in self.pending_resources.drain(..) {
            if self.resource_manager.contains(&resource)
                || self
                    .in_progress_resources
                    .iter()
                    .any(|(pending_resource, pending_type)| {
                        pending_resource == &resource && pending_type == &resource_type
                    })
            {
                continue;
            }
            self.resource_manager
                .async_download_resource(resource.clone(), &resource_type, &self.runtime.handle());
            self.in_progress_resources.push_back((resource, resource_type));
        }
    }

    /// Synchronously uploads resources.
    pub fn upload_resource(
        &mut self,
        resource_id: ResourceId,
        resource_type: ResourceType,
        bytes: impl Into<Vec<u8>>,
    ) -> Result<(), RetGuiError> {
        let bytes = bytes.into();
        if resource_type == ResourceType::Font {
            let fonts = self
                .font_context
                .collection
                .register_fonts(Blob::new(Arc::new(bytes)), None);
            if fonts.is_empty() {
                return Err(ResourceError::new(ResourceType::Font, "No fonts found in uploaded data").into());
            }
        } else {
            self.resource_manager.upload(resource_id, resource_type, bytes)?;
        }
        self.window_manager
            .dirty_and_redraw_all_windows(&self.elements, &mut self.gummy_tree, true);
        Ok(())
    }

    /// Inserts an element into this store and creates its layout node.
    pub fn insert_element<T: ElementInternals>(
        &mut self,
        is_scrollable: bool,
        create: impl FnOnce(ElementData) -> T,
    ) -> DynElement {
        let element = self
            .elements
            .insert_with(&self.access_tree, &mut self.by_internal_id, |me, access_tree| {
                Box::new(create(ElementData::new(me, is_scrollable, access_tree)))
            });
        self.elements
            .get_mut(element)
            .element_data_mut()
            .create_layout_node(&mut self.gummy_tree, None);
        element
    }

    /// Runs a local future and applies its result with exclusive access to this
    /// application on the GUI thread.
    pub fn spawn_local<F, O, C>(&mut self, future: F, on_complete: C)
    where
        F: Future<Output = O> + 'static,
        O: 'static,
        C: FnOnce(O, &mut App) + 'static,
    {
        self.gui_actions.spawn_local(future, on_complete);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        self.runtime.tokio_runtime_mut().block_on(future)
    }

    pub(crate) fn run_gui_actions(&mut self) {
        // Collect first so invoking an action never aliases the queue field with
        // the exclusive borrow of the complete store.
        let actions = self.gui_actions.drain();
        for action in actions {
            action(self);
        }
    }

    pub(crate) fn get(&self, element: DynElement) -> &dyn ElementInternals {
        self.elements.get(element)
    }

    /// Returns a retained element, or `None` when the handle is stale or belongs
    /// to another store.
    pub(crate) fn try_get(&self, element: DynElement) -> Option<&dyn ElementInternals> {
        self.elements.try_get(element)
    }

    /// Borrows a retained element as its concrete type.
    pub fn get_as<T: ElementInternals>(&self, element: DynElement) -> &T {
        (self.get(element) as &dyn Any)
            .downcast_ref()
            .expect("typed element handle changed type")
    }

    /// Borrows a retained element as its concrete type, returning `None`
    /// for a stale handle.
    pub(crate) fn try_get_as<T: ElementInternals>(&self, element: DynElement) -> Option<&T> {
        Some(
            (self.try_get(element)? as &dyn Any)
                .downcast_ref()
                .expect("typed element handle changed type"),
        )
    }

    /// Mutably borrows a retained element as its concrete type.
    ///
    /// The borrow is tied to this store borrow, just like the framework's own
    /// element-specific setters; no runtime borrow guard is involved.
    pub fn get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> &mut T {
        self.elements.get_as_mut(element)
    }

    /// Mutably borrows a retained element as its concrete type, returning
    /// `None` for a stale handle.
    pub(crate) fn try_get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> Option<&mut T> {
        self.elements.try_get_as_mut(element)
    }

    pub(crate) fn contains(&self, element: DynElement) -> bool {
        self.elements.contains(element)
    }

    /// Schedules an element to recompute when it next needs an animation update.
    pub fn schedule_animation_update(&mut self, element: DynElement) {
        queue_animation_update(&mut self.pending_animation_updates, element, false);
    }

    /// Stores application state and returns a typed handle to it.
    pub fn insert_state<T: 'static>(&mut self, value: T) -> State<T> {
        State::insert(&mut self.states, self.elements.store_id(), value)
    }

    pub fn state<T: 'static>(&self, state: State<T>) -> &T {
        state.read_from(&self.states, self.elements.store_id())
    }

    pub fn state_mut<T: 'static>(&mut self, state: State<T>) -> &mut T {
        state.write_to(&mut self.states, self.elements.store_id())
    }
}
