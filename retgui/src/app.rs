use std::collections::VecDeque;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use issho::AccessEvent;

use retgui_logging::info;

use retgui_primitives::geometry::{Point, Size};

#[cfg(target_arch = "wasm32")]
use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::resource_type::ResourceType;
use retgui_resource_manager::{ResourceId, ResourceManager};

use retgui_runtime::RetGuiRuntime;

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Ime, KeyEvent, PointerKind, WindowEvent};
use winit::event_loop::ActiveEventLoop;

use crate::elements::{AnimationSchedule, DynElement, Elements, Window, WindowElement, scrollable, set_focus_outline_visible};
use crate::events::{EventDispatcher, EventKind, ImeEvent, KeyboardEvent, PointerButtonEvent, PointerInfo, PointerMovedEvent, PointerScrollEvent, PointerState};
use crate::text::text_context::TextContext;

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
    pub(crate) elements: Elements,
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

impl App {
    pub(crate) fn new(mut elements: Elements) -> Self {
        let runtime = RetGuiRuntime::new();
        info!("Created async runtime");

        let resource_manager = elements.resource_manager().clone();
        let text_context = TextContext {
            font_context: elements.font_context.clone(),
            layout_context: Default::default(),
        };
        #[cfg(target_arch = "wasm32")]
        let (created_renderer_sender, created_renderer_receiver) = std::sync::mpsc::channel();

        Self {
            event_dispatcher: EventDispatcher::new(),
            text_context,
            elements,
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
                let scale_factor = window.effective_scale_factor(&self.elements);
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
                let scale_factor = window.effective_scale_factor(&self.elements);
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
                let scale_factor = window.effective_scale_factor(&self.elements);
                let logical_position = window.mouse_position(&self.elements).unwrap_or_default();
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
                self.elements.close_window(&window);
                if self.elements.window_manager.is_empty() {
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
                window.on_focused(&self.elements, focused);
            }
            _ => {}
        }

        WindowEventResult::Continue
    }

    pub fn on_close_requested(&mut self) {
        info!("RetGui application is closing.");
    }

    pub fn on_scale_factor_changed(&mut self, window: Window, scale_factor: f64) {
        window.on_scale_factor_changed(&mut self.elements, scale_factor);
    }

    pub fn on_resume(&mut self, event_loop: Option<&dyn ActiveEventLoop>) {
        self.active = true;

        let mut elements = std::mem::take(&mut self.elements);
        elements.on_resume(self, event_loop);
        self.elements = elements;
    }

    pub fn on_about_to_wait(&mut self, event_loop: Option<&dyn ActiveEventLoop>) -> Option<Duration> {
        #[cfg(not(target_arch = "wasm32"))]
        self.runtime.maybe_block_on(async {
            retgui_runtime::task::yield_now().await;
        });
        self.runtime.handle().update_local_set();
        self.elements.run_gui_actions();
        #[cfg(target_arch = "wasm32")]
        self.process_created_renderers();
        self.process_resources();
        self.process_accessibility_events();
        self.event_dispatcher
            .dispatch_queued_events(&mut self.text_context, &mut self.elements);

        let mut elements = std::mem::take(&mut self.elements);
        let next_animation_update = elements.on_about_to_wait(self, event_loop);
        self.elements = elements;
        next_animation_update
    }

    #[cfg(target_arch = "wasm32")]
    fn process_created_renderers(&mut self) {
        while let Ok(created) = self.created_renderer_receiver.try_recv() {
            if self.elements.contains(created.window) {
                let (gummy_tree, elements) = self.elements.disjoint_borrow_layout_and_elements();
                elements
                    .get_as_mut::<WindowElement>(created.window)
                    .on_renderer_created(gummy_tree, created.renderer, created.size);
            }
        }
    }

    fn process_accessibility_events(&mut self) {
        let tree = self.elements.access_tree.clone();
        while let Some((target, event)) = tree.pop_event() {
            if !self.elements.contains(target) {
                continue;
            }
            let _ = self.elements.dispatch_mut(target, |target, elements| {
                scrollable::handle_accessibility_scroll_event(elements, target, &event);
                target.on_access_event(elements, event)
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
            self.elements.dirty_and_redraw_all_windows(self.active);
        }
    }

    /// Suspends rendering until the application is resumed again.
    pub fn on_suspended(&mut self) {
        self.active = false;
    }

    /// Returns the window associated with a winit window ID.
    pub fn window_by_id(&self, window_id: winit::window::WindowId) -> Option<Window> {
        self.elements.window_manager.get_window_by_id(&self.elements, window_id)
    }

    /// Returns whether the driver should exit its event loop.
    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    /// Installs the driver callback used to wake a sleeping GUI event loop when
    /// asynchronous work schedules an element-store action.
    pub fn set_gui_waker(&self, waker: impl Fn() + Send + Sync + 'static) {
        self.elements.set_gui_waker(waker);
    }

    /// Handles the window resize event.
    pub fn on_resize(&mut self, window: Window, new_size: Size<f32>) {
        window.on_resize(&mut self.elements, new_size);
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
        delta: winit::event::MouseScrollDelta,
        state: PointerState,
    ) {
        let pointer_scroll_update = PointerScrollEvent::new(DynElement::new(window.inner), pointer, delta, state);
        let zoomed = self.elements.dispatch_mut(window.inner, |window, elements| {
            (window as &mut dyn std::any::Any)
                .downcast_mut::<WindowElement>()
                .unwrap()
                .maybe_zoom(elements, &pointer_scroll_update)
        });
        if zoomed {
            return;
        }
        self.dispatch_event(window, EventKind::PointerScroll(pointer_scroll_update));
    }

    pub fn on_pointer_button(
        &mut self,
        window: Window,
        button: Option<winit::event::MouseButton>,
        pointer: PointerInfo,
        state: PointerState,
        is_up: bool,
    ) {
        if !is_up {
            set_focus_outline_visible(&mut self.elements, false);
        }

        let cursor_position = state.logical_point();
        let pointer_event = PointerButtonEvent::new(DynElement::new(window.inner), button, pointer, state);

        let event = if is_up {
            EventKind::PointerUp(pointer_event)
        } else {
            EventKind::PointerDown(pointer_event)
        };
        window.set_mouse_position(
            &mut self.elements,
            Some(Point::new(cursor_position.x, cursor_position.y)),
        );

        self.dispatch_event(window, event);
    }

    pub fn on_pointer_moved(&mut self, window: Window, pointer: PointerInfo, state: PointerState) {
        window.set_mouse_position(&mut self.elements, Some(state.logical_point()));
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
            set_focus_outline_visible(&mut self.elements, true);
        }
        let event = KeyboardEvent::new(DynElement::new(window.inner), keyboard_input, modifiers, is_composing);
        let toggled = self.elements.dispatch_mut(window.inner, |window, elements| {
            (window as &mut dyn std::any::Any)
                .downcast_mut::<WindowElement>()
                .unwrap()
                .maybe_toggle_perf_stats(elements, &event)
        });
        if toggled {
            return;
        }
        let zoomed = self.elements.dispatch_mut(window.inner, |window, elements| {
            (window as &mut dyn std::any::Any)
                .downcast_mut::<WindowElement>()
                .unwrap()
                .maybe_zoom_keyboard(elements, &event)
        });
        if zoomed {
            return;
        }
        let navigation_target = self
            .elements
            .get_as::<WindowElement>(window.inner)
            .tab_navigation_target(&self.elements, &event);
        let event = match state {
            ElementState::Pressed => EventKind::KeyDown(event),
            ElementState::Released => EventKind::KeyUp(event),
        };
        let prevent_defaults = self.dispatch_event(window, event);
        if !prevent_defaults && let Some(target) = navigation_target {
            self.elements.dispatch_mut(target, |target, elements| {
                target.focus(elements);
                scrollable::handle_accessibility_scroll_event(elements, target, &AccessEvent::ScrollIntoView);
            });
            window.request_redraw(&self.elements);
        }
    }

    fn on_request_redraw_internal(&mut self, window: Window) {
        let animation_schedule = self.elements.animation_tick(&window);
        self.update_resources();
        window.on_redraw(
            &mut self.elements,
            &mut self.text_context,
            self.resource_manager.clone(),
        );

        if animation_schedule == AnimationSchedule::NextFrame && window.winit_window(&self.elements).is_some() {
            window.request_redraw(&self.elements);
        }
    }

    fn dispatch_event(&mut self, window: Window, mut event: EventKind) -> bool {
        let mouse_pos = window.mouse_position(&self.elements);
        let mut renderer = std::mem::replace(
            &mut self.elements.get_as_mut::<WindowElement>(window.inner).renderer,
            Box::new(retgui_renderer::blank_renderer::BlankRenderer::default()),
        );
        let prevented = self.event_dispatcher.dispatch_event(
            &mut event,
            mouse_pos,
            window.inner,
            &mut self.text_context,
            &mut *renderer,
            &mut self.target_scratch,
            &mut self.elements,
        );
        self.elements.get_as_mut::<WindowElement>(window.inner).renderer = renderer;
        prevented
    }

    fn update_resources(&mut self) {
        for (resource, resource_type) in self.elements.pending_resources.drain(..) {
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
}
