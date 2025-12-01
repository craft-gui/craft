use crate::elements::Element;
use crate::events::{CraftMessage, Event, EventDispatchType, FocusAction};

use crate::app::DOCUMENTS;
use crate::events::pointer_capture::{processing_pending_pointer_capture};
use crate::text::text_context::TextContext;
use craft_logging::{span, Level};
use craft_primitives::geometry::Point;
use std::cell::RefCell;
use std::collections::{VecDeque};
use std::rc::Rc;
use ui_events::pointer::PointerId;
use crate::events::helpers::{call_default_element_event_handler, call_user_event_handlers, find_target};

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

    pub fn dispatch_bubbling_event(
        &self,
        message: &CraftMessage,
        dispatch_type: EventDispatchType,
        text_context: &mut Option<TextContext>,
        targets: &mut VecDeque<Rc<RefCell<dyn Element>>>,
    ) {
        match dispatch_type {
            EventDispatchType::Bubbling => {
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
        }
    }

    pub(super) fn maybe_dispatch_pointer_leave(
        &self,
        dispatch_type: EventDispatchType,
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
        dispatch_type: EventDispatchType,
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

        let target: Rc<RefCell<dyn Element>> = find_target(&root, mouse_position, message);
        let mut current_target = Some(Rc::clone(&target));

        // Gather and "freeze" the elements we will visit.
        let mut targets: VecDeque<Rc<RefCell<dyn Element>>> = VecDeque::new();
        while let Some(node) = current_target {
            targets.push_back(Rc::clone(&node));
            current_target = node.borrow().parent().as_ref().and_then(|p| p.upgrade());
        }

        // if message.is_pointer_event() {}
        self.maybe_dispatch_pointer_leave(EventDispatchType::Bubbling, text_context, &targets);
        self.maybe_dispatch_pointer_enter(EventDispatchType::Bubbling, text_context, &targets);

        self.dispatch_bubbling_event(message, EventDispatchType::Bubbling.clone(), text_context, &mut targets);

        // 9.5 Implicit release of pointer capture
        // https://w3c.github.io/pointerevents/#implicit-release-of-pointer-capture
        let is_pointer_up_event = matches!(message, CraftMessage::PointerButtonUp(_));
        if is_pointer_up_event
        /* || is_pointer_canceled */
        {
            // Immediately after firing the pointerup or pointercancel events, the user agent MUST clear the pending pointer capture target override
            // for the pointerId of the pointerup or pointercancel event that was just dispatched
            DOCUMENTS.with_borrow_mut(|docs| {
                let key = &PointerId::new(1).unwrap();
                let _ = docs.get_current_document().pending_pointer_captures.remove(key);
            });

            processing_pending_pointer_capture(self, EventDispatchType::Bubbling, root, text_context);
        } else if message.is_pointer_event() && !message.is_got_or_lost_pointer_capture() {
            processing_pending_pointer_capture(self, EventDispatchType::Bubbling, root, text_context);
        }

        self.previous_targets = targets;
    }

}