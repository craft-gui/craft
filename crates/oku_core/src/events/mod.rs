mod keyboard_input;
mod mouse_wheel;
mod pointer_button;
mod pointer_moved;

pub(crate) mod internal;
pub(crate) mod resource_event;
pub mod update_queue_entry;

pub use keyboard_input::KeyboardInput;
pub use mouse_wheel::MouseWheel;
pub use pointer_button::PointerButton;
pub use pointer_moved::PointerMoved;
pub use winit::event::ButtonSource;
pub use winit::event::ElementState;

use crate::events::OkuMessage::PointerButtonEvent;
use std::any::Any;
use winit::event::Ime;
pub use winit::event::MouseButton;

pub struct Event {
    /// The id of the element that triggered this event.
    pub target: Option<String>,
    /// The id of an element who is listening to this event.
    pub current_target: Option<String>,
    pub message: Message,
}

impl Event {
    pub fn new(message: Message) -> Self {
        Self {
            current_target: None,
            target: None,
            message,
        }
    }

    /// Set the event's target to the id of the element that triggered it.
    pub fn target(mut self, target: Option<String>) -> Self {
        self.target = target;
        self
    }

    /// Set the event's current target to the id of an element who is listening to this event.
    pub fn current_target(mut self, current_target: Option<String>) -> Self {
        self.current_target = current_target;
        self
    }
}

#[derive(Clone, Debug)]
pub enum OkuMessage {
    Initialized,
    PointerButtonEvent(PointerButton),
    KeyboardInputEvent(KeyboardInput),
    PointerMovedEvent(PointerMoved),
    MouseWheelEvent(MouseWheel),
    ImeEvent(Ime),
    TextInputChanged(String),
    /// Generated when a dropdown is opened or closed. The boolean is the status of is_open after the event has occurred.
    DropdownToggled(bool),
    /// The index of the item selected in the list.
    /// For example, if you select the first item the index will be 0.
    DropdownItemSelected(usize),
    /// Generated when a switch is toggled. The boolean is the status of toggled after the event has occurred.
    SwitchToggled(bool),
}

impl OkuMessage {
    pub fn clicked(&self) -> bool {
        if let PointerButtonEvent(pointer_button) = self {
            if pointer_button.button.mouse_button() == MouseButton::Left && pointer_button.state == ElementState::Released {
                return true;
            }
        }

        false
    }

}

pub enum Message {
    OkuMessage(OkuMessage),
    UserMessage(Box<dyn Any>),
}

impl Message {
    pub fn clicked(&self) -> bool {
        if let Message::OkuMessage(message) = self {
            return message.clicked();
        }

        false
    }
}