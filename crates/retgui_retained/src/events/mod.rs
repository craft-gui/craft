use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub use ui_events;

pub use winit::event::{ElementState, Ime, Modifiers, MouseButton};

pub use crate::events::mouse_wheel::MouseWheel;

pub(crate) use event_dispatch::EventDispatcher;

use ui_events::keyboard::KeyboardEvent;
use ui_events::pointer::{PointerButtonEvent, PointerId, PointerScrollEvent, PointerUpdate};

use crate::PinnedFutureAny;
use crate::elements::ElementInternals;
use crate::utils::cloneable_any::CloneableAny;

pub mod internal;

pub(crate) mod pointer_capture;

mod event_dispatch;
mod helpers;
mod mouse_wheel;

pub type CheckboxToggledHandler = Rc<dyn Fn(&mut Event, CheckboxToggled)>;
pub type DropdownItemSelectedHandler = Rc<dyn Fn(&mut Event, usize)>;
pub type KeyboardInputHandler = Rc<dyn Fn(&mut Event, &KeyboardEvent)>;
pub type PointerEnterHandler = Rc<dyn Fn(&mut Event)>;
pub type PointerEventHandler = Rc<dyn Fn(&mut Event, &PointerButtonEvent)>;
pub type PointerLeaveHandler = Rc<dyn Fn(&mut Event)>;
pub type PointerUpdateHandler = Rc<dyn Fn(&mut Event, &PointerUpdate)>;
pub type ClickHandler = Rc<dyn Fn(&mut Event)>;
pub type PointerCaptureHandler = Rc<dyn Fn(&mut Event)>;
pub type RadioValueChangedHandler = Rc<dyn Fn(&mut Event, Rc<RefCell<String>>)>;
pub type ScrollHandler = Rc<dyn Fn(&mut Event)>;
pub type SliderValueChangedHandler = Rc<dyn Fn(&mut Event, f64)>;
pub type TextInputChangedHandler = Rc<dyn Fn(&mut Event, &TextInputChanged)>;
pub type UserMessage = dyn CloneableAny;

#[derive(Clone)]
pub enum EventDispatchType {
    Bubble,
    Capture,
}

#[derive(Clone)]
pub enum EventKind {
    GotPointerCapture(),
    LostPointerCapture(),
    PointerEnter(),
    PointerLeave(),
    PointerButtonUp(PointerButtonEvent),
    PointerButtonDown(PointerButtonEvent),
    Click(),
    KeyboardInputEvent(KeyboardEvent),
    PointerMovedEvent(PointerUpdate),
    PointerScroll(PointerScrollEvent),
    Scroll(),
    ImeEvent(Ime),
    TextInputChanged(TextInputChanged),
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
    RadioValueChanged(Rc<RefCell<String>>),
    CheckboxToggled(CheckboxToggled),
}

#[derive(Clone)]
pub struct CheckboxToggled {
    pub label: String,
    pub status: bool,
}

#[derive(Clone)]
pub struct TextInputChanged {
    pub value: String,
}

/// The result of an update.
pub struct Event {
    pub target: Rc<RefCell<dyn ElementInternals>>,
    pub current_target: Rc<RefCell<dyn ElementInternals>>,

    /// Propagate retgui_events to the next element. True by default.
    pub propagate: bool,
    /// A future that will produce a message when complete. The message will be sent to the origin component.
    pub future: Option<PinnedFutureAny>,
    /// Prevent default event handlers from running when an retgui_event is not explicitly handled.
    /// False by default.
    pub prevent_defaults: bool,
}

impl EventKind {
    pub(super) fn is_system_pointer_event(&self) -> bool {
        matches!(
            self,
            EventKind::PointerMovedEvent(_)
                | EventKind::PointerButtonUp(_)
                | EventKind::PointerButtonDown(_)
                | EventKind::PointerScroll(_)
        )
    }

    pub(super) fn pointer_id(&self) -> Option<PointerId> {
        match self {
            EventKind::PointerButtonUp(e) => e.pointer.pointer_id,
            EventKind::PointerButtonDown(e) => e.pointer.pointer_id,
            EventKind::PointerMovedEvent(e) => e.pointer.pointer_id,
            EventKind::PointerScroll(e) => e.pointer.pointer_id,
            _ => None,
        }
    }

    pub(super) fn is_keyboard_event(&self) -> bool {
        matches!(self, EventKind::KeyboardInputEvent(_) | EventKind::ImeEvent(_))
    }

    pub fn new_element_message<T>(data: T) -> EventKind
    where
        T: Any + Send + Sync + Clone,
    {
        Self::ElementMessage(Arc::new(data))
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

    pub fn new(target: Rc<RefCell<dyn ElementInternals>>) -> Self {
        Event {
            target: target.clone(),
            current_target: target,
            propagate: true,
            future: None,
            prevent_defaults: false,
        }
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
}
