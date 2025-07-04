use crate::components::ComponentId;
use crate::events::{CraftMessage, EventDispatchType, Message};
use craft_primitives::geometry::Rectangle;
use crate::PinnedFutureAny;
use std::any::Any;
use crate::utils::cloneable_any::CloneableAny;

#[derive(Debug, Clone, Copy, Default)]
pub enum PointerCapture {
    #[default]
    None,
    Set,
    Unset,
}

/// The result of an update.
pub struct Event {
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

impl Event {

    #[cfg(not(target_arch = "wasm32"))]
    pub fn async_result<T: Send + Sync + 'static + Clone>(t: T) -> Box<dyn CloneableAny + Send + Sync + 'static> {
        Box::new(t)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn async_result<T: 'static + Clone>(t: T) -> Box<dyn CloneableAny + 'static> {
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

impl Default for Event {
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
        }
    }
}

impl Event {
    pub fn new() -> Event {
        Event::default()
    }

    pub fn pinned_future(&mut self, future: PinnedFutureAny) {
        self.future = Some(future);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn future<F: Future<Output = Box<dyn CloneableAny + Send + Sync>> + 'static + Send>(&mut self, future: F) {
        self.future = Some(Box::pin(future));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn future<F: Future<Output = Box<dyn CloneableAny>> + 'static>(&mut self, future: F) {
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
