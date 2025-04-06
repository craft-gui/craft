use crate::events::{CraftMessage, EventDispatchType, Message};
use crate::PinnedFutureAny;
use std::any::Any;

#[derive(Debug, Clone, Copy, Default)]
pub enum PointerCapture {
    #[default]
    None,
    Set,
    Unset,
}

/// The result of an update.
pub struct UpdateResult {
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
}

impl UpdateResult {
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
}

impl Default for UpdateResult {
    fn default() -> Self {
        UpdateResult {
            propagate: true,
            future: None,
            prevent_defaults: false,
            result_message: None,
            pointer_capture: Default::default(),
            effects: Vec::new(),
        }
    }
}

impl UpdateResult {
    pub fn new() -> UpdateResult {
        UpdateResult::default()
    }

    pub fn pinned_future(mut self, future: PinnedFutureAny) -> Self {
        self.future = Some(future);
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn future<F: Future<Output = Box<dyn Any + Send + Sync>> + 'static + Send>(mut self, future: F) -> Self {
        self.future = Some(Box::pin(future));
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn future<F: Future<Output = Box<dyn Any>> + 'static>(mut self, future: F) -> Self {
        self.future = Some(Box::pin(future));
        self
    }

    pub fn prevent_defaults(mut self) -> Self {
        self.prevent_defaults = true;
        self
    }

    pub fn prevent_propagate(mut self) -> Self {
        self.propagate = false;
        self
    }

    pub(crate) fn result_message(mut self, message: CraftMessage) -> Self {
        self.result_message = Some(message);
        self
    }

    pub fn pointer_capture(mut self, pointer_capture: PointerCapture) -> Self {
        self.pointer_capture = pointer_capture;
        self
    }

    pub fn add_effect(mut self, event_dispatch_type: EventDispatchType, message: Message) -> Self {
        self.effects.push((event_dispatch_type, message));
        self
    }
}
