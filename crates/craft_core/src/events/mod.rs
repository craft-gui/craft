mod keyboard_input;
mod mouse_wheel;
mod pointer_button;
mod pointer_moved;

pub(crate) mod internal;
pub(crate) mod resource_event;
pub mod update_queue_entry;
pub(crate) mod event_dispatch;

//#[cfg(test)]
//mod tests;

pub use keyboard_input::KeyboardInput;
pub use mouse_wheel::MouseWheel;
pub use pointer_button::PointerButton;
pub use pointer_moved::PointerMoved;
pub use winit::event::ButtonSource;
pub use winit::event::ElementState;

use crate::components::ComponentId;
use crate::events::CraftMessage::PointerButtonEvent;
use std::any::Any;
use std::sync::Arc;
pub use winit::event::Modifiers;
pub use winit::event::Ime;
pub use winit::event::MouseButton;
use crate::elements::Element;

#[derive(Clone)]
pub enum EventDispatchType {
    Bubbling,
    Direct(ComponentId),
    /// Sends the message to all elements that satisfy the given predicate function.
    /// The predicate should return `true` for an element to receive the message.
    DirectToMatchingElements(Arc<dyn Fn(&dyn Element) -> bool + Send + Sync + 'static>),
}

#[derive(Clone, Debug)]
pub enum CraftMessage {
    Initialized,
    PointerButtonEvent(PointerButton),
    KeyboardInputEvent(KeyboardInput),
    PointerMovedEvent(PointerMoved),
    MouseWheelEvent(MouseWheel),
    ModifiersChangedEvent(winit::event::Modifiers),
    ImeEvent(Ime),
    TextInputChanged(String),
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
        if let PointerButtonEvent(pointer_button) = self {
            if pointer_button.button.mouse_button() == MouseButton::Left
                && pointer_button.state == ElementState::Released
            {
                return true;
            }
        }

        false
    }
}

impl PointerButton {
    pub fn clicked(&self) -> bool {
        self.button.mouse_button() == MouseButton::Left && self.state == ElementState::Released
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
