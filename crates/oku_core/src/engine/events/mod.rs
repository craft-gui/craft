mod pointer_button;
mod pointer_moved;
mod keyboard_input;
mod mouse_wheel;

pub(crate) mod internal;
pub(crate) mod resource_event;
pub mod update_queue_entry;

pub use pointer_button::PointerButton;
pub use pointer_moved::PointerMoved;
pub use winit::event::ButtonSource;
pub use winit::event::ElementState;
pub use keyboard_input::KeyboardInput;
pub use mouse_wheel::MouseWheel;

pub use winit::event::MouseButton;
use std::any::Any;

pub enum EventResult {
    Stop,
    Continue,
}

#[derive(Clone, Debug)]
pub enum OkuEvent {
    Initialized,
    PointerButtonEvent(PointerButton),
    KeyboardInputEvent(KeyboardInput),
    PointerMovedEvent(PointerMoved),
    MouseWheelEvent(MouseWheel),
}

pub enum Message {
    OkuMessage(OkuEvent),
    UserMessage(Box<dyn Any>),
}
