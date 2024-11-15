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

use std::any::Any;
pub use winit::event::MouseButton;

pub struct Event {
    pub target: Option<String>,
    pub message: Message,
}

impl Event {
    pub fn new(message: Message) -> Self {
        Self {
            target: None,
            message,
        }
    }

    pub fn target(mut self, target: Option<String>) -> Self {
        self.target = target;
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
    TextInputChanged(String),
}

pub enum Message {
    OkuMessage(OkuMessage),
    UserMessage(Box<dyn Any>),
}
