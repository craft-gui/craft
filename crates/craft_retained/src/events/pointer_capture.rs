use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::{Rc, Weak};

use ui_events::pointer::PointerId;

use crate::elements::ElementInternals;
use crate::events::EventKind;
use crate::events::event_dispatch::{dispatch_bubbling_event, dispatch_capturing_event};

/// Stores window specific information like pointer captures, focus (soon), etc.
#[derive(Default, Clone)]
pub struct PointerCapture {
    /// Tracks elements that are *currently* pointer captured.
    pub(crate) pointer_captures: HashMap<PointerId, Weak<RefCell<dyn ElementInternals>>>,
    /// Tracks elements that *should* be pointer captured.
    pub(crate) pending_pointer_captures: HashMap<PointerId, Weak<RefCell<dyn ElementInternals>>>,
}

impl PointerCapture {
    /// Remove an element from pointer capture.
    pub fn remove_element(&mut self, element: &Rc<RefCell<dyn ElementInternals>>) {
        let element_weak = element.borrow().element_data().me.clone();
        self.pointer_captures.retain(|_, v| !Weak::ptr_eq(v, &element_weak));
        self.pending_pointer_captures
            .retain(|_, v| !Weak::ptr_eq(v, &element_weak));
    }

    /// Returns the currently pointer captured element or None.
    pub(super) fn find_pointer_capture_target(&self, message: &EventKind) -> Option<Rc<RefCell<dyn ElementInternals>>> {
        // 9.4 Implicit pointer capture
        // https://w3c.github.io/pointerevents/#implicit-pointer-capture
        //
        let pointer_capture_element_id: Option<Weak<RefCell<dyn ElementInternals>>> = {
            let key = &PointerId::new(1).unwrap();
            if matches!(message, EventKind::GotPointerCapture()) {
                // Check pending (step 2):
                // https://w3c.github.io/pointerevents/#process-pending-pointer-capture
                self.pending_pointer_captures.get(key).cloned()
            } else {
                self.pointer_captures.get(key).cloned()
            }
        };

        pointer_capture_element_id.map(|element| element.upgrade().expect("Pointer captured element should exist."))
    }

    /// Checks if Got or Lost events need to be dispatched and updates the current pointer capture.
    pub(super) fn process_pending_pointer_capture(&mut self) {
        // 4.1.3.2 Process pending pointer capture
        let key = &PointerId::new(1).unwrap();
        let (pointer_capture_val, pending_pointer_capture_val) = {
            let pointer_capture_val = self.pointer_captures.get(key);
            let pending_pointer_capture_val = self.pending_pointer_captures.get(key);

            (pointer_capture_val.cloned(), pending_pointer_capture_val.cloned())
        };

        // 1. If the pointer capture target override for this pointer is set and is not equal to the pending pointer capture target override,
        // then fire a pointer event named lostpointercapture at the pointer capture target override node.
        if let Some(pointer_capture_val) = pointer_capture_val.clone()
            && Some(pointer_capture_val.as_ptr()) != pending_pointer_capture_val.clone().map(|w| w.as_ptr())
        {
            let msg = EventKind::LostPointerCapture();
            let target = self.find_pointer_capture_target(&msg);

            if let Some(target) = target {
                let mut targets: VecDeque<Rc<RefCell<dyn ElementInternals>>> = VecDeque::new();
                let mut current_target = Some(Rc::clone(&target));
                while let Some(node) = current_target {
                    targets.push_back(Rc::clone(&node));
                    current_target = node.borrow().parent().as_ref().and_then(|p| p.upgrade());
                }

                dispatch_capturing_event(&msg, &mut targets);
                dispatch_bubbling_event(&msg, &mut targets);
            }
        }

        // 2. If the pending pointer capture target override for this pointer is set and is not equal to the pointer capture target override,
        // then fire a pointer event named gotpointercapture at the pending pointer capture target override.
        if let Some(pending_pointer_capture_val) = pending_pointer_capture_val.clone()
            && Some(pending_pointer_capture_val.as_ptr()) != pointer_capture_val.map(|w| w.as_ptr())
        {
            let msg = EventKind::GotPointerCapture();
            let target = self.find_pointer_capture_target(&msg);

            if let Some(target) = target {
                let mut targets: VecDeque<Rc<RefCell<dyn ElementInternals>>> = VecDeque::new();
                let mut current_target = Some(Rc::clone(&target));
                while let Some(node) = current_target {
                    targets.push_back(Rc::clone(&node));
                    current_target = node.borrow().parent().as_ref().and_then(|p| p.upgrade());
                }

                dispatch_capturing_event(&msg, &mut targets);
                dispatch_bubbling_event(&msg, &mut targets);
            }
        }

        // 3. Set the pointer capture target override to the pending pointer capture target override, if set.
        // Otherwise, clear the pointer capture target override.

        if let Some(pending_pointer_capture_val) = pending_pointer_capture_val {
            self.pointer_captures.insert(*key, pending_pointer_capture_val);
        } else {
            self.pointer_captures.remove(key);
        }
    }

    pub(super) fn maybe_handle_implicit_pointer_capture_release(&mut self, message: &EventKind) {
        // 9.5 Implicit release of pointer capture
        // https://w3c.github.io/pointerevents/#implicit-release-of-pointer-capture
        let is_pointer_up_event = matches!(message, EventKind::PointerButtonUp(_));
        if is_pointer_up_event
        /* || is_pointer_canceled */
        {
            // Immediately after firing the pointerup or pointercancel events, the user agent MUST clear the pending pointer capture target override
            // for the pointerId of the pointerup or pointercancel event that was just dispatched
            let key = &PointerId::new(1).unwrap();
            let _ = self.pending_pointer_captures.remove(key);

            self.process_pending_pointer_capture();
        } else if message.is_pointer_event() && !message.is_got_or_lost_pointer_capture() {
            self.process_pending_pointer_capture();
        }
    }
}
