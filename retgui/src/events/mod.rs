//! Typed events and event dispatch controls.
//!
//! Import [`Event`] to use the behavior shared by every concrete event type,
//! such as [`Event::stop_propagation`] and [`Event::prevent_default`].

pub use crate::events::mouse_wheel::MouseWheel;
pub(crate) use event_dispatch::EventDispatcher;
pub use ui_events;
pub use ui_events::keyboard::KeyboardEvent as UiKeyboardEvent;
pub use ui_events::pointer::{PointerButtonEvent as UiPointerButtonEvent, PointerScrollEvent as UiPointerScrollEvent};
pub use winit::event::{ElementState, Ime, Modifiers, MouseButton};

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use retgui_primitives::geometry::Point;
use ui_events::ScrollDelta;
use ui_events::keyboard::{Code, Key, KeyState, Location, Modifiers as KeyboardModifiers};
use ui_events::pointer::{PointerButton, PointerId, PointerInfo, PointerState, PointerUpdate};

use crate::elements::ElementInternals;

pub mod internal;

pub(crate) mod pointer_capture;

mod event_dispatch;
mod helpers;
mod mouse_wheel;

pub type EventTarget = Rc<RefCell<dyn ElementInternals>>;

/// State shared by every RetGui event.
#[derive(Clone)]
pub struct BaseEvent {
    target: EventTarget,
    current_target: EventTarget,
    propagation_stopped: bool,
    default_prevented: bool,
}

impl BaseEvent {
    pub fn new(target: EventTarget) -> Self {
        Self {
            target: target.clone(),
            current_target: target,
            propagation_stopped: false,
            default_prevented: false,
        }
    }

    fn retarget(&mut self, target: EventTarget) {
        self.target = target.clone();
        self.current_target = target;
    }
}

/// Behavior shared by every concrete RetGui event.
///
/// This trait must be in scope to call its methods on a concrete event.
pub trait Event {
    #[doc(hidden)]
    fn base(&self) -> &BaseEvent;

    #[doc(hidden)]
    fn base_mut(&mut self) -> &mut BaseEvent;

    /// Returns the element at which the event was originally dispatched.
    fn target(&self) -> EventTarget {
        self.base().target.clone()
    }

    /// Returns the element whose handlers are currently being invoked.
    fn current_target(&self) -> EventTarget {
        self.base().current_target.clone()
    }

    /// Stops the event from reaching any remaining elements.
    fn stop_propagation(&mut self) {
        self.base_mut().propagation_stopped = true;
    }

    /// Prevents RetGui's default handling for the event.
    fn prevent_default(&mut self) {
        self.base_mut().default_prevented = true;
    }

    /// Returns whether propagation has been stopped.
    fn is_propagation_stopped(&self) -> bool {
        self.base().propagation_stopped
    }

    /// Returns whether default handling has been prevented.
    fn is_default_prevented(&self) -> bool {
        self.base().default_prevented
    }
}

/// What caused a click event.
#[derive(Clone, Debug)]
pub enum ClickTrigger {
    Pointer {
        button: Option<PointerButton>,
        position: Point,
    },
    Keyboard {
        key: Key,
    },
    Accessibility,
    Programmatic,
}

#[derive(Clone)]
pub struct ClickEvent {
    base: BaseEvent,
    pub trigger: ClickTrigger,
}

impl ClickEvent {
    pub fn new(target: EventTarget, trigger: ClickTrigger) -> Self {
        Self {
            base: BaseEvent::new(target),
            trigger,
        }
    }
}

#[derive(Clone)]
pub struct PointerButtonEvent {
    base: BaseEvent,
    pub button: Option<PointerButton>,
    pub position: Point,
    pub pointer: PointerInfo,
    pub state: PointerState,
}

impl PointerButtonEvent {
    pub fn new(target: EventTarget, event: UiPointerButtonEvent) -> Self {
        let UiPointerButtonEvent {
            button,
            pointer,
            state,
        } = event;
        let position = state.logical_point();

        Self {
            base: BaseEvent::new(target),
            button,
            position,
            pointer,
            state,
        }
    }
}

#[derive(Clone)]
pub struct KeyboardEvent {
    base: BaseEvent,
    pub state: KeyState,
    pub key: Key,
    pub code: Code,
    pub location: Location,
    pub modifiers: KeyboardModifiers,
    pub repeat: bool,
    pub is_composing: bool,
}

impl KeyboardEvent {
    pub fn new(target: EventTarget, event: UiKeyboardEvent) -> Self {
        let UiKeyboardEvent {
            state,
            key,
            code,
            location,
            modifiers,
            repeat,
            is_composing,
        } = event;

        Self {
            base: BaseEvent::new(target),
            state,
            key,
            code,
            location,
            modifiers,
            repeat,
            is_composing,
        }
    }
}

/// A semantic scroll notification.
#[derive(Clone)]
pub struct ScrollEvent {
    base: BaseEvent,
}

impl ScrollEvent {
    pub fn new(target: EventTarget) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

/// An element gaining focus.
#[derive(Clone)]
pub struct FocusEvent {
    base: BaseEvent,
}

impl FocusEvent {
    pub fn new(target: EventTarget) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

/// An element losing focus.
#[derive(Clone)]
pub struct UnfocusEvent {
    base: BaseEvent,
}

impl UnfocusEvent {
    pub fn new(target: EventTarget) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

/// A pointer entering an element.
#[derive(Clone)]
pub struct PointerEnterEvent {
    base: BaseEvent,
}

impl PointerEnterEvent {
    pub fn new(target: EventTarget) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

/// A pointer leaving an element.
#[derive(Clone)]
pub struct PointerLeaveEvent {
    base: BaseEvent,
}

impl PointerLeaveEvent {
    pub fn new(target: EventTarget) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

#[derive(Clone)]
pub struct PointerCaptureEvent {
    base: BaseEvent,
    pub pointer_id: PointerId,
}

impl PointerCaptureEvent {
    pub fn new(target: EventTarget, pointer_id: PointerId) -> Self {
        Self {
            base: BaseEvent::new(target),
            pointer_id,
        }
    }
}

#[derive(Clone)]
pub struct PointerMovedEvent {
    base: BaseEvent,
    pub pointer: PointerInfo,
    pub current: PointerState,
    pub coalesced: Vec<PointerState>,
    pub predicted: Vec<PointerState>,
}

impl PointerMovedEvent {
    pub fn new(target: EventTarget, event: PointerUpdate) -> Self {
        let PointerUpdate {
            pointer,
            current,
            coalesced,
            predicted,
        } = event;

        Self {
            base: BaseEvent::new(target),
            pointer,
            current,
            coalesced,
            predicted,
        }
    }
}

#[derive(Clone)]
pub struct PointerScrollEvent {
    base: BaseEvent,
    pub pointer: PointerInfo,
    pub delta: ScrollDelta,
    pub state: PointerState,
}

impl PointerScrollEvent {
    pub fn new(target: EventTarget, event: UiPointerScrollEvent) -> Self {
        let UiPointerScrollEvent {
            pointer,
            delta,
            state,
        } = event;

        Self {
            base: BaseEvent::new(target),
            pointer,
            delta,
            state,
        }
    }
}

#[derive(Clone)]
pub struct ImeEvent {
    base: BaseEvent,
    pub ime: Ime,
}

impl ImeEvent {
    pub fn new(target: EventTarget, ime: Ime) -> Self {
        Self {
            base: BaseEvent::new(target),
            ime,
        }
    }
}

#[derive(Clone)]
pub struct TextInputChangedEvent {
    base: BaseEvent,
    pub value: String,
}

impl TextInputChangedEvent {
    pub fn new(target: EventTarget, value: String) -> Self {
        Self {
            base: BaseEvent::new(target),
            value,
        }
    }
}

#[derive(Clone)]
pub struct LinkClickedEvent {
    base: BaseEvent,
    pub url: String,
}

impl LinkClickedEvent {
    pub fn new(target: EventTarget, url: String) -> Self {
        Self {
            base: BaseEvent::new(target),
            url,
        }
    }
}

#[derive(Clone)]
pub struct DropdownToggledEvent {
    base: BaseEvent,
    pub is_open: bool,
}

impl DropdownToggledEvent {
    pub fn new(target: EventTarget, is_open: bool) -> Self {
        Self {
            base: BaseEvent::new(target),
            is_open,
        }
    }
}

#[derive(Clone)]
pub struct DropdownItemSelectedEvent {
    base: BaseEvent,
    pub index: usize,
}

impl DropdownItemSelectedEvent {
    pub fn new(target: EventTarget, index: usize) -> Self {
        Self {
            base: BaseEvent::new(target),
            index,
        }
    }
}

#[derive(Clone)]
pub struct SwitchToggledEvent {
    base: BaseEvent,
    pub toggled: bool,
}

impl SwitchToggledEvent {
    pub fn new(target: EventTarget, toggled: bool) -> Self {
        Self {
            base: BaseEvent::new(target),
            toggled,
        }
    }
}

#[derive(Clone)]
pub struct SliderValueChangedEvent {
    base: BaseEvent,
    pub value: f64,
}

impl SliderValueChangedEvent {
    pub fn new(target: EventTarget, value: f64) -> Self {
        Self {
            base: BaseEvent::new(target),
            value,
        }
    }
}

#[derive(Clone)]
pub struct RadioValueChangedEvent {
    base: BaseEvent,
    pub value: Rc<RefCell<String>>,
}

impl RadioValueChangedEvent {
    pub fn new(target: EventTarget, value: Rc<RefCell<String>>) -> Self {
        Self {
            base: BaseEvent::new(target),
            value,
        }
    }
}

#[derive(Clone)]
pub struct CheckboxToggledEvent {
    base: BaseEvent,
    pub label: String,
    pub status: bool,
}

impl CheckboxToggledEvent {
    pub fn new(target: EventTarget, label: String, status: bool) -> Self {
        Self {
            base: BaseEvent::new(target),
            label,
            status,
        }
    }
}

/// A type-erased application-defined event.
#[derive(Clone)]
pub struct CustomEvent {
    base: BaseEvent,
    pub data: Arc<UserEventData>,
}

impl CustomEvent {
    pub fn new<T>(target: EventTarget, detail: T) -> Self
    where
        T: Any + 'static,
    {
        Self {
            base: BaseEvent::new(target),
            data: Arc::new(detail),
        }
    }

    pub fn from_arc(target: EventTarget, detail: Arc<UserEventData>) -> Self {
        Self {
            base: BaseEvent::new(target),
            data: detail,
        }
    }

    pub fn data<T: Any>(&self) -> Option<&T> {
        self.data.downcast_ref()
    }
}

impl Event for ClickEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerButtonEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for KeyboardEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for ScrollEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for FocusEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for UnfocusEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerCaptureEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerEnterEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerLeaveEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerMovedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerScrollEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for ImeEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for TextInputChangedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for LinkClickedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for DropdownToggledEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for DropdownItemSelectedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for SwitchToggledEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for SliderValueChangedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for RadioValueChangedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for CheckboxToggledEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for CustomEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

pub type CheckboxToggledHandler = Rc<dyn Fn(&mut CheckboxToggledEvent)>;
pub type ClickHandler = Rc<dyn Fn(&mut ClickEvent)>;
pub type CustomHandler = Rc<dyn Fn(&mut CustomEvent)>;
pub type DropdownItemSelectedHandler = Rc<dyn Fn(&mut DropdownItemSelectedEvent)>;
pub type FocusHandler = Rc<dyn Fn(&mut FocusEvent)>;
pub type KeyboardInputHandler = Rc<dyn Fn(&mut KeyboardEvent)>;
pub type PointerCaptureHandler = Rc<dyn Fn(&mut PointerCaptureEvent)>;
pub type PointerEnterHandler = Rc<dyn Fn(&mut PointerEnterEvent)>;
pub type PointerEventHandler = Rc<dyn Fn(&mut PointerButtonEvent)>;
pub type PointerLeaveHandler = Rc<dyn Fn(&mut PointerLeaveEvent)>;
pub type PointerMovedHandler = Rc<dyn Fn(&mut PointerMovedEvent)>;
pub type PointerUpdateHandler = PointerMovedHandler;
pub type RadioValueChangedHandler = Rc<dyn Fn(&mut RadioValueChangedEvent)>;
pub type ScrollHandler = Rc<dyn Fn(&mut ScrollEvent)>;
pub type SliderValueChangedHandler = Rc<dyn Fn(&mut SliderValueChangedEvent)>;
pub type TextInputChangedHandler = Rc<dyn Fn(&mut TextInputChangedEvent)>;
pub type UnfocusHandler = Rc<dyn Fn(&mut UnfocusEvent)>;
pub type UserEventData = dyn Any;

#[derive(Clone)]
pub enum EventCallbackKind {
    CheckboxToggled(CheckboxToggledHandler),
    Click(ClickHandler),
    Custom(CustomHandler),
    DropdownItemSelected(DropdownItemSelectedHandler),
    Focus(FocusHandler),
    GotPointerCapture(PointerCaptureHandler),
    KeyboardInput(KeyboardInputHandler),
    LostPointerCapture(PointerCaptureHandler),
    PointerButtonDown(PointerEventHandler),
    PointerButtonUp(PointerEventHandler),
    PointerEnter(PointerEnterHandler),
    PointerLeave(PointerLeaveHandler),
    PointerMoved(PointerMovedHandler),
    RadioValueChanged(RadioValueChangedHandler),
    Scroll(ScrollHandler),
    SliderValueChanged(SliderValueChangedHandler),
    TextInputChanged(TextInputChangedHandler),
    Unfocus(UnfocusHandler),
}

#[derive(Clone, Copy, Default)]
pub struct EventListenerOptions {
    pub capturing: bool,
}

#[derive(Clone)]
pub struct EventCallback {
    pub callback: EventCallbackKind,
    pub capturing: bool,
}

/// The inline event representation used by RetGui's dispatcher and queue.
///
/// Most users should use concrete event types through the typed callback APIs.
#[doc(hidden)]
#[derive(Clone)]
pub enum EventKind {
    GotPointerCapture(PointerCaptureEvent),
    LostPointerCapture(PointerCaptureEvent),
    PointerEnter(PointerEnterEvent),
    PointerLeave(PointerLeaveEvent),
    PointerUp(PointerButtonEvent),
    PointerDown(PointerButtonEvent),
    Click(ClickEvent),
    Focus(FocusEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    PointerMoved(PointerMovedEvent),
    PointerScroll(PointerScrollEvent),
    Scroll(ScrollEvent),
    Ime(ImeEvent),
    TextInputChanged(TextInputChangedEvent),
    LinkClicked(LinkClickedEvent),
    DropdownToggled(DropdownToggledEvent),
    DropdownItemSelected(DropdownItemSelectedEvent),
    SwitchToggled(SwitchToggledEvent),
    SliderValueChanged(SliderValueChangedEvent),
    Custom(CustomEvent),
    RadioValueChanged(RadioValueChangedEvent),
    Unfocus(UnfocusEvent),
    CheckboxToggled(CheckboxToggledEvent),
}

impl EventKind {
    pub(super) fn is_system_pointer_event(&self) -> bool {
        matches!(
            self,
            Self::PointerMoved(_) | Self::PointerUp(_) | Self::PointerDown(_) | Self::PointerScroll(_)
        )
    }

    pub(super) fn pointer_id(&self) -> Option<PointerId> {
        match self {
            Self::GotPointerCapture(event) | Self::LostPointerCapture(event) => Some(event.pointer_id),
            Self::PointerUp(event) | Self::PointerDown(event) => event.pointer.pointer_id,
            Self::PointerMoved(event) => event.pointer.pointer_id,
            Self::PointerScroll(event) => event.pointer.pointer_id,
            _ => None,
        }
    }

    pub(super) fn is_keyboard_event(&self) -> bool {
        matches!(self, Self::KeyDown(_) | Self::KeyUp(_) | Self::Ime(_))
    }

    pub(crate) fn retarget(&mut self, target: EventTarget) {
        self.base_mut().retarget(target);
    }
}

impl Event for EventKind {
    fn base(&self) -> &BaseEvent {
        match self {
            Self::GotPointerCapture(event) => event.base(),
            Self::LostPointerCapture(event) => event.base(),
            Self::PointerEnter(event) => event.base(),
            Self::PointerLeave(event) => event.base(),
            Self::PointerUp(event) => event.base(),
            Self::PointerDown(event) => event.base(),
            Self::Click(event) => event.base(),
            Self::Focus(event) => event.base(),
            Self::KeyDown(event) => event.base(),
            Self::KeyUp(event) => event.base(),
            Self::PointerMoved(event) => event.base(),
            Self::PointerScroll(event) => event.base(),
            Self::Scroll(event) => event.base(),
            Self::Ime(event) => event.base(),
            Self::TextInputChanged(event) => event.base(),
            Self::LinkClicked(event) => event.base(),
            Self::DropdownToggled(event) => event.base(),
            Self::DropdownItemSelected(event) => event.base(),
            Self::SwitchToggled(event) => event.base(),
            Self::SliderValueChanged(event) => event.base(),
            Self::Custom(event) => event.base(),
            Self::RadioValueChanged(event) => event.base(),
            Self::Unfocus(event) => event.base(),
            Self::CheckboxToggled(event) => event.base(),
        }
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        match self {
            Self::GotPointerCapture(event) => event.base_mut(),
            Self::LostPointerCapture(event) => event.base_mut(),
            Self::PointerEnter(event) => event.base_mut(),
            Self::PointerLeave(event) => event.base_mut(),
            Self::PointerUp(event) => event.base_mut(),
            Self::PointerDown(event) => event.base_mut(),
            Self::Click(event) => event.base_mut(),
            Self::Focus(event) => event.base_mut(),
            Self::KeyDown(event) => event.base_mut(),
            Self::KeyUp(event) => event.base_mut(),
            Self::PointerMoved(event) => event.base_mut(),
            Self::PointerScroll(event) => event.base_mut(),
            Self::Scroll(event) => event.base_mut(),
            Self::Ime(event) => event.base_mut(),
            Self::TextInputChanged(event) => event.base_mut(),
            Self::LinkClicked(event) => event.base_mut(),
            Self::DropdownToggled(event) => event.base_mut(),
            Self::DropdownItemSelected(event) => event.base_mut(),
            Self::SwitchToggled(event) => event.base_mut(),
            Self::SliderValueChanged(event) => event.base_mut(),
            Self::Custom(event) => event.base_mut(),
            Self::RadioValueChanged(event) => event.base_mut(),
            Self::Unfocus(event) => event.base_mut(),
            Self::CheckboxToggled(event) => event.base_mut(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::elements::Container;

    use super::{Event, EventKind, EventTarget, FocusEvent};

    fn event_target() -> EventTarget {
        Container::new().inner
    }

    #[test]
    fn event_controls_track_dispatch_state() {
        let target = event_target();
        let mut event = FocusEvent::new(target.clone());

        assert!(Rc::ptr_eq(&event.target(), &target));
        assert!(Rc::ptr_eq(&event.current_target(), &target));
        assert!(!event.is_propagation_stopped());
        assert!(!event.is_default_prevented());

        event.stop_propagation();
        event.prevent_default();

        assert!(event.is_propagation_stopped());
        assert!(event.is_default_prevented());
    }

    #[test]
    fn retarget_updates_both_targets() {
        let original = event_target();
        let replacement = event_target();
        let mut event = EventKind::Focus(FocusEvent::new(original));

        event.retarget(replacement.clone());

        assert!(Rc::ptr_eq(&event.target(), &replacement));
        assert!(Rc::ptr_eq(&event.current_target(), &replacement));
    }
}
