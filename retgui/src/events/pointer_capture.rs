use std::collections::HashMap;

use crate::events::PointerId;

use crate::elements::{DynElement, Elements};
use crate::events::event_dispatch::dispatch_event;
use crate::events::{EventKind, PointerCaptureEvent};
use crate::text::text_context::TextContext;

/// Stores window specific information like pointer captures, focus (soon), etc.
#[derive(Default, Clone)]
pub struct PointerCapture {
    /// Tracks elements that are *currently* pointer captured.
    pub(crate) pointer_captures: HashMap<PointerId, DynElement>,
    /// Tracks elements that *should* be pointer captured.
    pub(crate) pending_pointer_captures: HashMap<PointerId, DynElement>,
}

impl PointerCapture {
    /// Remove an element from pointer capture.
    pub fn remove_element(&mut self, element: DynElement) {
        self.pointer_captures.retain(|_, value| *value != element);
        self.pending_pointer_captures.retain(|_, value| *value != element);
    }

    /// Returns the currently pointer captured element or None.
    pub(super) fn find_pointer_capture_target(
        &self,
        message: &EventKind,
        pointer_id: &PointerId,
    ) -> Option<DynElement> {
        // 9.4 Implicit pointer capture
        // https://w3c.github.io/pointerevents/#implicit-pointer-capture
        //
        let pointer_capture_element_id: Option<DynElement> = {
            if matches!(message, EventKind::GotPointerCapture(_)) {
                // Check pending (step 2):
                // https://w3c.github.io/pointerevents/#process-pending-pointer-capture
                self.pending_pointer_captures.get(pointer_id).cloned()
            } else {
                self.pointer_captures.get(pointer_id).cloned()
            }
        };

        pointer_capture_element_id
    }

    /// Checks if Got or Lost events need to be dispatched and updates the current pointer capture.
    pub(super) fn process_pending_pointer_capture(
        &mut self,
        elements: &mut Elements,
        text_context: &mut TextContext,
        pointer_id: &PointerId,
    ) -> bool {
        let mut did_pointer_capture_change = false;

        // 4.1.3.2 Process pending pointer capture
        let (pointer_capture_val, pending_pointer_capture_val) = {
            let pointer_capture_val = self.pointer_captures.get(pointer_id);
            let pending_pointer_capture_val = self.pending_pointer_captures.get(pointer_id);

            (pointer_capture_val.cloned(), pending_pointer_capture_val.cloned())
        };

        // 1. If the pointer capture target override for this pointer is set and is not equal to the pending pointer capture target override,
        // then fire a pointer event named lostpointercapture at the pointer capture target override node.
        if let Some(pointer_capture_val) = pointer_capture_val
            && Some(pointer_capture_val) != pending_pointer_capture_val
        {
            let targets = crate::events::helpers::freeze_target_list(pointer_capture_val, elements);
            let mut event = EventKind::LostPointerCapture(PointerCaptureEvent::new(pointer_capture_val, *pointer_id));
            dispatch_event(&mut event, &targets, text_context, elements);

            did_pointer_capture_change = true;
        }

        // 2. If the pending pointer capture target override for this pointer is set and is not equal to the pointer capture target override,
        // then fire a pointer event named gotpointercapture at the pending pointer capture target override.
        if let Some(pending_pointer_capture_val) = pending_pointer_capture_val
            && Some(pending_pointer_capture_val) != pointer_capture_val
        {
            let targets = crate::events::helpers::freeze_target_list(pending_pointer_capture_val, elements);
            let mut event =
                EventKind::GotPointerCapture(PointerCaptureEvent::new(pending_pointer_capture_val, *pointer_id));
            dispatch_event(&mut event, &targets, text_context, elements);

            did_pointer_capture_change = true;
        }

        // 3. Set the pointer capture target override to the pending pointer capture target override, if set.
        // Otherwise, clear the pointer capture target override.

        if let Some(pending_pointer_capture_val) = pending_pointer_capture_val {
            self.pointer_captures.insert(*pointer_id, pending_pointer_capture_val);
        } else {
            self.pointer_captures.remove(pointer_id);
        }

        did_pointer_capture_change
    }

    pub(super) fn maybe_handle_implicit_pointer_capture_release(
        &mut self,
        elements: &mut Elements,
        message: &EventKind,
        text_context: &mut TextContext,
        pointer_id: &PointerId,
    ) -> bool {
        // 9.5 Implicit release of pointer capture
        // https://w3c.github.io/pointerevents/#implicit-release-of-pointer-capture
        let is_pointer_up_event = matches!(message, EventKind::PointerUp(_));
        let mut did_pointer_capture_change = false;
        if is_pointer_up_event
        /* || is_pointer_canceled */
        {
            // Immediately after firing the pointerup or pointercancel events, the user agent MUST clear the pending pointer capture target override
            // for the pointerId of the pointerup or pointercancel event that was just dispatched
            let _ = self.pending_pointer_captures.remove(pointer_id);

            did_pointer_capture_change = self.process_pending_pointer_capture(elements, text_context, pointer_id);
        } else if message.is_system_pointer_event() {
            did_pointer_capture_change = self.process_pending_pointer_capture(elements, text_context, pointer_id);
        }

        did_pointer_capture_change
    }
}
