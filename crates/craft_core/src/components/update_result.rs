use crate::elements::Element;
use crate::events::{CraftMessage, EventDispatchType, Message};
use crate::geometry::Rectangle;
use crate::window_context::WindowContext;
use crate::PinnedFutureAny;
use std::any::Any;
use crate::components::ComponentId;

#[derive(Debug, Clone, Copy, Default)]
pub enum PointerCapture {
    #[default]
    None,
    Set,
    Unset,
}

/// The result of an update.
pub struct Event<'a> {
    /// Propagate craft_events to the next element. True by default.
    pub propagate: bool,
    /// A future that will produce a message when complete. The message will be sent to the origin component.
    pub future: Option<PinnedFutureAny>,
    /// Prevent default event handlers from running when an craft_event is not explicitly handled.
    /// False by default.
    pub prevent_defaults: bool,
    pub(crate) result_message: Option<CraftMessage>,
    /// Redirect future pointer events to this component. None by default.
    pub(crate) pointer_capture: PointerCapture,
    pub(crate) effects: Vec<(EventDispatchType, Message)>,
    pub(crate) ime: ImeAction,
    pub focus: FocusAction,

    pub target: Option<&'a dyn Element>,
    pub window: WindowContext,
    pub current_target: Option<&'a dyn Element>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ImeAction {
    #[default]
    None,
    Set(Rectangle),
    Unset,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FocusAction {
    #[default]
    None,
    Set(ComponentId),
    Unset,
}

impl FocusAction {

    pub(crate) fn merge(&self, other: FocusAction) -> FocusAction {
        match other {
            FocusAction::None => *self,
            FocusAction::Set(id) => FocusAction::Set(id),
            FocusAction::Unset => FocusAction::Unset,
        }
    }

}

impl<'a> Event<'a> {
    pub fn with_window_context(window: WindowContext) -> Self {
        Event {
            window,
            ..Default::default()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn async_result<T: Send + Sync + 'static>(t: T) -> Box<dyn Any + Send + Sync + 'static> {
        Box::new(t)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn async_result<T: 'static>(t: T) -> Box<dyn Any + 'static> {
        Box::new(t)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn async_no_result() -> Box<dyn Any + Send + 'static> {
        Box::new(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn async_no_result() -> Box<dyn Any + 'static> {
        Box::new(())
    }

    pub fn ime_action(&mut self, action: ImeAction) {
        self.ime = action;
    }

    pub fn focus_action(&mut self, action: FocusAction) {
        self.focus = action;
    }
}

impl<'a> Default for Event<'a> {
    fn default() -> Self {
        Event {
            propagate: true,
            future: None,
            prevent_defaults: false,
            result_message: None,
            pointer_capture: Default::default(),
            effects: Vec::new(),
            ime: ImeAction::None,
            focus: FocusAction::None,
            target: None,
            current_target: None,
            window: WindowContext::new(),
        }
    }
}

impl<'a> Event<'a> {
    pub fn new() -> Event<'a> {
        Event::default()
    }

    pub fn pinned_future(&mut self, future: PinnedFutureAny) {
        self.future = Some(future);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn future<F: Future<Output = Box<dyn Any + Send + Sync>> + 'static + Send>(&mut self, future: F) {
        self.future = Some(Box::pin(future));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn future<F: Future<Output = Box<dyn Any>> + 'static>(&mut self, future: F) {
        self.future = Some(Box::pin(future));
    }

    pub fn prevent_defaults(&mut self) {
        self.prevent_defaults = true;
    }

    pub fn prevent_propagate(&mut self) {
        self.propagate = false;
    }

    pub fn result_message(&mut self, message: CraftMessage) {
        self.result_message = Some(message);
    }

    pub fn pointer_capture(&mut self, pointer_capture: PointerCapture) {
        self.pointer_capture = pointer_capture;
    }

    pub fn add_effect(&mut self, event_dispatch_type: EventDispatchType, message: Message) {
        self.effects.push((event_dispatch_type, message));
    }
}
