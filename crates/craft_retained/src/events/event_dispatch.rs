use crate::elements::Element;
use crate::events::{CraftMessage, Event, EventDispatchType, FocusAction};

use crate::app::DOCUMENTS;
use crate::events::pointer_capture::{maybe_handle_implicit_pointer_capture_release, process_pending_pointer_capture};
use crate::text::text_context::TextContext;
use craft_logging::{span, Level};
use craft_primitives::geometry::Point;
use std::cell::RefCell;
use std::collections::{VecDeque};
use std::rc::Rc;
use ui_events::pointer::PointerId;
use crate::events::helpers::{call_default_element_event_handler, call_user_event_handlers, find_target, freeze_target_list};

pub struct EventDispatcher {
    previous_targets: VecDeque<Rc<RefCell<dyn Element>>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            previous_targets: Default::default(),
        }
    }

    pub fn dispatch_once(
        &self,
        message: &CraftMessage,
        text_context: &mut Option<TextContext>,
        current_target: &Rc<RefCell<dyn Element>>,
        target: &Rc<RefCell<dyn Element>>,
    ) {
        // Call the callback handlers.
        call_user_event_handlers(current_target, message);

        // Call the default on_event element functions.
        call_default_element_event_handler(current_target, target, text_context, message);
    }

    pub fn dispatch_capturing_event(
        &self,
        _message: &CraftMessage,
        _text_context: &mut Option<TextContext>,
        _targets: &mut VecDeque<Rc<RefCell<dyn Element>>>,
    ) {

    }

    pub fn dispatch_bubbling_event(
        &self,
        message: &CraftMessage,
        text_context: &mut Option<TextContext>,
        targets: &mut VecDeque<Rc<RefCell<dyn Element>>>,
    ) {
        let target = targets[0].clone();
        let mut propagate = true;

        // Call the callback handlers.
        for current_target in targets.iter() {
            call_user_event_handlers(current_target, message);
            if !propagate {
                break;
            }
        }

        // Call the default on_event element functions.
        for current_target in targets.iter() {
            call_default_element_event_handler(current_target, &target, text_context, message);
            if !propagate {
                break;
            }
        }
    }

    pub(super) fn maybe_dispatch_pointer_leave(
        &self,
        text_context: &mut Option<TextContext>,
        targets: &VecDeque<Rc<RefCell<dyn Element>>>,
    ) {
        for prev_target in self.previous_targets.iter() {
            let mut found = false;
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
                self.dispatch_once(&CraftMessage::PointerLeave(), text_context, &prev_target.clone(), &prev_target.clone());
            }
        }
    }

    pub(super) fn maybe_dispatch_pointer_enter(
        &self,
        text_context: &mut Option<TextContext>,
        targets: &VecDeque<Rc<RefCell<dyn Element>>>,
    ) {
        for target in targets.iter().rev() {
            let mut found = false;
            let target_id = target.borrow().id();

            for prev_target in self.previous_targets.iter().rev() {
                let prev_target_id = prev_target.borrow().id();

                if prev_target_id == target_id {
                    found = true;
                    break;
                }
            }

            // We weren't in the prev target list, but we are in the new list. (PointerEnter)
            if !found {
                self.dispatch_once(&CraftMessage::PointerEnter(), text_context, &target.clone(), &target.clone());
            }
        }
    }

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
        self.dispatch_capturing_event(message, text_context, &mut targets);

        // Handle bubbling
        self.dispatch_bubbling_event(message, text_context, &mut targets);

        // NOTE: May dispatch gotpointercapture and lostpointercapture. Handles capturing and bubbling.
        // The event dispatch flow looks like this:
        // - pointer_event(capture), pointer_event(bubble) (Executed above)
        // - lostpointercapture(capture), lostpointercapture(bubble)
        // - gotpointercapture(capture), gotpointercapture(bubble)
        maybe_handle_implicit_pointer_capture_release(self, message, root, text_context);

        self.previous_targets = targets;
    }

}