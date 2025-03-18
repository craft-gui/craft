use crate::events::OkuMessage;
use crate::PinnedFutureAny;
use std::any::Any;

/// The result of an update.
pub struct UpdateResult {
    /// Propagate oku_events to the next element. True by default.
    pub propagate: bool,
    /// A future that will produce a message when complete. The message will be sent to the origin component.
    pub future: Option<PinnedFutureAny>,
    /// Prevent default event handlers from running when an oku_event is not explicitly handled.
    /// False by default.
    pub prevent_defaults: bool,
    pub(crate) result_message: Option<OkuMessage>,
}

impl UpdateResult {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn async_result<T: Send + 'static>(t: T) -> Box<dyn Any + Send + 'static> {
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
    pub fn future<F: Future<Output = Box<dyn Any + Send>> + 'static + Send>(mut self, future: F) -> Self {
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

    pub(crate) fn result_message(mut self, message: OkuMessage) -> Self {
        self.result_message = Some(message);
        self
    }
}
