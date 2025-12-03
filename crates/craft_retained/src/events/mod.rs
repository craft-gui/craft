mod mouse_wheel;

pub mod internal;
mod event_dispatch;
mod pointer_capture;
mod helpers;
//#[cfg(test)]
//mod tests;

pub use mouse_wheel::MouseWheel;
pub use winit::event::ElementState;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
pub use ui_events;
use ui_events::keyboard::KeyboardEvent;
use ui_events::pointer::{PointerButtonEvent, PointerScrollEvent, PointerUpdate};
pub use winit::event::Ime;
pub use winit::event::Modifiers;
pub use winit::event::MouseButton;
use crate::PinnedFutureAny;
use crate::utils::cloneable_any::CloneableAny;

pub(crate) use event_dispatch::EventDispatcher;
use crate::elements::Element;

pub type PointerEventHandler = Rc<dyn Fn(&mut Event, &PointerButtonEvent)>;
pub type PointerCaptureHandler = Rc<dyn Fn(&mut Event)>;
pub type PointerEnterHandler = Rc<dyn Fn(&mut Event)>;
pub type PointerLeaveHandler = Rc<dyn Fn(&mut Event)>;

pub type PointerUpdateHandler = Rc<dyn Fn(&mut Event, &PointerUpdate)>;

pub type KeyboardInputHandler = Rc<dyn Fn(&mut Event, &KeyboardEvent)>;

#[derive(Clone)]
pub enum EventDispatchType {
    Bubbling,
}

#[derive(Clone)]
pub enum CraftMessage {
    GotPointerCapture(),
    LostPointerCapture(),
    PointerEnter(),
    PointerLeave(),
    PointerButtonUp(PointerButtonEvent),
    PointerButtonDown(PointerButtonEvent),
    KeyboardInputEvent(KeyboardEvent),
    PointerMovedEvent(PointerUpdate),
    PointerScroll(PointerScrollEvent),
    ImeEvent(Ime),
    TextInputChanged(String),
    LinkClicked(String),
    /// Generated when a dropdown is opened or closed. The boolean is the status of is_open after the event has occurred.
    DropdownToggled(bool),
    /// The index of the item selected in the list.
    /// For example, if you select the first item the index will be 0.
    DropdownItemSelected(usize),
    /// Generated when a switch is toggled. The boolean is the status of toggled after the event has occurred.
    SwitchToggled(bool),
    SliderValueChanged(f64),
    ElementMessage(Arc<UserMessage>),
}

impl CraftMessage {
    pub(super) fn is_pointer_event(&self) -> bool {
        matches!(
            self,
            CraftMessage::PointerMovedEvent(_)
            | CraftMessage::PointerButtonUp(_)
            | CraftMessage::PointerButtonDown(_)
            | CraftMessage::GotPointerCapture()
            | CraftMessage::LostPointerCapture()
            | CraftMessage::PointerScroll(_)
        )
    }

    pub(super) fn is_got_or_lost_pointer_capture(&self) -> bool {
        matches!(
            self,
            CraftMessage::GotPointerCapture()
            | CraftMessage::LostPointerCapture()
        )
    }

    pub fn new_element_message<T>(data: T) -> CraftMessage
    where
        T: Any + Send + Sync + Clone,
    {
        Self::ElementMessage(Arc::new(data))
    }
}
pub type UserMessage = dyn CloneableAny;


/// The result of an update.
pub struct Event {
    pub target: Rc<RefCell<dyn Element>>,
    /// Propagate craft_events to the next element. True by default.
    pub propagate: bool,
    /// A future that will produce a message when complete. The message will be sent to the origin component.
    pub future: Option<PinnedFutureAny>,
    /// Prevent default event handlers from running when an craft_event is not explicitly handled.
    /// False by default.
    pub prevent_defaults: bool,
}


#[derive(Debug, Clone, Copy, Default)]
pub enum FocusAction {
    #[default]
    None,
    Set(u64),
    Unset,
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
}

impl Event {
    fn new(target: Rc<RefCell<dyn Element>>) -> Self {
        Event {
            target,
            propagate: true,
            future: None,
            prevent_defaults: false,
        }
    }
}

impl Event {
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

}
