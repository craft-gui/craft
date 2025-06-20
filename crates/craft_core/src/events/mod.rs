mod mouse_wheel;

pub(crate) mod event_dispatch;
pub mod internal;
pub(crate) mod resource_event;
pub mod update_queue_entry;
//#[cfg(test)]
//mod tests;

pub use mouse_wheel::MouseWheel;
pub use winit::event::ElementState;

use crate::components::ComponentId;
use crate::elements::Element;
use crate::events::CraftMessage::PointerButtonUp;
use std::any::Any;
use std::sync::Arc;
pub use ui_events;
use ui_events::keyboard::KeyboardEvent;
use ui_events::pointer::{PointerButtonUpdate, PointerScrollUpdate, PointerUpdate};
pub use winit::event::Ime;
pub use winit::event::Modifiers;
pub use winit::event::MouseButton;

#[derive(Clone)]
pub enum EventDispatchType {
    Bubbling,
    Direct(ComponentId),
    /// Sends the message to all elements that satisfy the given predicate function.
    /// The predicate should return `true` for an element to receive the message.
    DirectToMatchingElements(Arc<dyn Fn(&dyn Element) -> bool + Send + Sync + 'static>),
    Accesskit(ComponentId),
}

#[derive(Clone, Debug)]
pub enum CraftMessage {
    Initialized,
    PointerButtonUp(PointerButtonUpdate),
    PointerButtonDown(PointerButtonUpdate),
    KeyboardInputEvent(KeyboardEvent),
    PointerMovedEvent(PointerUpdate),
    PointerScroll(PointerScrollUpdate),
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
    pub fn clicked(&self) -> bool {
        if let PointerButtonUp(pointer_button) = self {
            if pointer_button.is_primary() {
                return true;
            }
        }

        false
    }

    pub fn new_element_message<T>(data: T) -> CraftMessage
    where
        T: Any + Send + Sync,
    {
        Self::ElementMessage(Arc::new(data))
    }
}

#[cfg(target_arch = "wasm32")]
pub type UserMessage = dyn Any;
#[cfg(not(target_arch = "wasm32"))]
pub type UserMessage = dyn Any + Send + Sync;

pub enum Message {
    CraftMessage(CraftMessage),
    #[cfg(target_arch = "wasm32")]
    UserMessage(Box<UserMessage>),
    #[cfg(not(target_arch = "wasm32"))]
    UserMessage(Box<UserMessage>),
}

impl Message {
    pub fn clicked(&self) -> bool {
        if let Message::CraftMessage(message) = self {
            return message.clicked();
        }

        false
    }
}
