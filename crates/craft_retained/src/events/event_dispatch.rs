use crate::elements::Element;
use crate::events::{CraftMessage, Event, FocusAction};

use crate::events::helpers::{
    call_default_element_event_handler, call_user_event_handlers, find_target, freeze_target_list,
};
use crate::events::pointer_capture::{maybe_handle_implicit_pointer_capture_release};
use crate::text::text_context::TextContext;
use craft_logging::{span, Level};
use craft_primitives::geometry::Point;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::{Rc, Weak};

pub(super) fn dispatch_capturing_event(
    _message: &CraftMessage,
    _targets: &mut VecDeque<Rc<RefCell<dyn Element>>>,
) {
}

/// Dispatches 1 event to many elements.
/// The first dispatch happens at the top-most visual element.
pub(super) fn dispatch_bubbling_event(
    message: &CraftMessage,
    targets: &mut VecDeque<Rc<RefCell<dyn Element>>>,
) -> Event {
    let target = targets[0].clone();
    let mut base_event = Event::new(target.clone());

    // Call the callback handlers.
    for current_target in targets.iter() {
        call_user_event_handlers(&mut base_event, current_target, message);
        if !base_event.propagate {
            break;
        }
    }

    base_event
}

/// Responsible for dispatching events.
pub(crate) struct EventDispatcher {
    /// A "frozen" target list used to diff against the current target list.
    /// This is useful for pointer enter, leave, etc.
    previous_targets: VecDeque<Weak<RefCell<dyn Element>>>,
}

impl EventDispatcher {
    /// Creates an event dispatcher and zeros out the previous target list.
    pub fn new() -> Self {
        Self {
            previous_targets: Default::default(),
        }
    }

    /// Dispatches 1 event to 1 element.
    /// NOTE: This calls the user callbacks + the default event handler (if prevent_default() is not called).
    pub(super) fn dispatch_once(
        &self,
        message: &CraftMessage,
        text_context: &mut Option<TextContext>,
        target: &Rc<RefCell<dyn Element>>,
    ) {
        let mut base_event = Event::new(target.clone());

        // Call the callback handlers.
        call_user_event_handlers(&mut base_event, target, message);

        if !base_event.prevent_defaults {
            // Call the default on_event element functions.
            call_default_element_event_handler(&mut base_event, target, target, text_context, message);
        }
    }

    /// Diffs the current and previous target lists and dispatches
    /// `pointer_leave` to any element that was present in the previous list
    /// but is not present in the current one.
    ///
    /// Note: This event does not bubble.
    pub(super) fn maybe_dispatch_pointer_leave(
        &self,
        text_context: &mut Option<TextContext>,
        targets: &VecDeque<Rc<RefCell<dyn Element>>>,
    ) {
        for prev_target in self.previous_targets.iter() {
            let mut found = false;

            let prev_target = prev_target.upgrade();
            if prev_target.is_none() {
                continue;
            }
            let prev_target = prev_target.unwrap();

            let prev_target_id = prev_target.borrow().id();

            for target in targets.iter() {
                let target_id = target.borrow().id();

                if prev_target_id == target_id {
                    found = true;
                    break;
                }
            }

            // We had a prev target, but we don't in the new list. (PointerLeave)
            if !found {
                self.dispatch_once(&CraftMessage::PointerLeave(), text_context, &prev_target.clone());
            }
        }
    }

    /// Diffs the current and previous target lists and dispatches
    /// `pointer_enter` to any element that exists in the current list
    /// but not in the previous one.
    ///
    /// Note: This event does not bubble.
    pub(super) fn maybe_dispatch_pointer_enter(
        &self,
        text_context: &mut Option<TextContext>,
        targets: &VecDeque<Rc<RefCell<dyn Element>>>,
    ) {
        for target in targets.iter().rev() {
            let mut found = false;
            let target_id = target.borrow().id();

            for prev_target in self.previous_targets.iter().rev() {
                let prev_target = prev_target.upgrade();
                if prev_target.is_none() {
                    continue;
                }
                let prev_target = prev_target.unwrap();
                let prev_target_id = prev_target.borrow().id();

                if prev_target_id == target_id {
                    found = true;
                    break;
                }
            }

            // We weren't in the prev target list, but we are in the new list. (PointerEnter)
            if !found {
                self.dispatch_once(&CraftMessage::PointerEnter(), text_context, &target.clone());
            }
        }
    }

    /// Dispatches events.
    /// May emit multiple events from a single message (pointer enter, leave, etc.).
    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_event(
        &mut self,
        message: &CraftMessage,
        mouse_position: Option<Point>,
        root: Rc<RefCell<dyn Element>>,
        text_context: &mut Option<TextContext>,
    ) {
        let mut _focus = FocusAction::None;
        let span = span!(Level::INFO, "dispatch event");
        let _enter = span.enter();

        // Find the target and freeze the list, so the same set of elements are visited across sub event dispatches.
        let target: Rc<RefCell<dyn Element>> = find_target(&root, mouse_position, message);
        let mut targets: VecDeque<Rc<RefCell<dyn Element>>> = freeze_target_list(target);

        self.maybe_dispatch_pointer_leave(text_context, &targets);
        self.maybe_dispatch_pointer_enter(text_context, &targets);

        // Handle capturing
        dispatch_capturing_event(message, &mut targets);

        // Handle bubbling
        let mut base_event = dispatch_bubbling_event(message, &mut targets);
        let target = targets[0].clone();

        // NOTE: Only certain events will trigger default behavior.
        // We don't currently check for this, but we should.
        if !base_event.prevent_defaults {
            // Call the default on_event element functions.
            for current_target in targets.iter() {
                call_default_element_event_handler(&mut base_event, current_target, &target, text_context, message);
                if !base_event.propagate {
                    break;
                }
            }
        }

        // NOTE: May dispatch gotpointercapture and lostpointercapture. Handles capturing and bubbling.
        // The event dispatch flow looks like this:
        // - pointer_event(capture), pointer_event(bubble) (Executed above)
        // - lostpointercapture(capture), lostpointercapture(bubble)
        // - gotpointercapture(capture), gotpointercapture(bubble)
        maybe_handle_implicit_pointer_capture_release(message);

        self.previous_targets = targets.iter().map(Rc::downgrade).collect();
    }
}

pub fn dispatch_event(event: Event, craft_message: CraftMessage) {
    let mut targets: VecDeque<Rc<RefCell<dyn Element>>> = freeze_target_list(event.target);
    // Handle capturing
    dispatch_capturing_event(&craft_message, &mut targets);

    // Handle bubbling
    dispatch_bubbling_event(&craft_message, &mut targets);
}
