use crate::app::DOCUMENTS;
use crate::elements::Element;
use crate::events::event_dispatch::EventDispatcher;
use crate::events::helpers::collect_nodes;
use crate::events::CraftMessage;
use crate::text::text_context::TextContext;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use ui_events::pointer::PointerId;

/// Returns the currently pointer captured element or None.
pub(super) fn find_pointer_capture_target(
    nodes: &Vec<Rc<RefCell<dyn Element>>>,
    message: &CraftMessage,
) -> Option<Rc<RefCell<dyn Element>>> {
    // 9.4 Implicit pointer capture
    // https://w3c.github.io/pointerevents/#implicit-pointer-capture
    //
    let pointer_capture_element_id = DOCUMENTS.with_borrow_mut(|docs| {
        let key = &PointerId::new(1).unwrap();

        if matches!(message, CraftMessage::GotPointerCapture()) {
            // Check pending (step 2):
            // https://w3c.github.io/pointerevents/#process-pending-pointer-capture
            docs.get_current_document().pending_pointer_captures.get(key).copied()
        } else {
            docs.get_current_document().pointer_captures.get(key).copied()
        }
    });

    // Skip hit-testing if pointer capture is active AND it is a pointer event.
    if let Some(pointer_capture_element_id) = pointer_capture_element_id
        && message.is_pointer_event()
    /*|| is_ime_event)*/
    {
        for node in nodes {
            if node.borrow().id() == pointer_capture_element_id {
                return Some(Rc::clone(node));
            }
        }
    }

    None
}

/// Checks if Got or Lost events need to be dispatched and updates the current pointer capture.
pub(super) fn process_pending_pointer_capture(
    event_dispatcher: &mut EventDispatcher,
    root: Rc<RefCell<dyn Element>>,
    text_context: &mut Option<TextContext>,
) {
    // 4.1.3.2 Process pending pointer capture
    let key = &PointerId::new(1).unwrap();
    let (pointer_capture_val, pending_pointer_capture_val) = DOCUMENTS.with_borrow_mut(|docs| {
        let current_doc = docs.get_current_document();
        let pointer_capture_val = current_doc.pointer_captures.get(key);
        let pending_pointer_capture_val = current_doc.pending_pointer_captures.get(key);

        (pointer_capture_val.cloned(), pending_pointer_capture_val.cloned())
    });

    // 1. If the pointer capture target override for this pointer is set and is not equal to the pending pointer capture target override,
    // then fire a pointer event named lostpointercapture at the pointer capture target override node.
    if let Some(pointer_capture_val) = pointer_capture_val
        && Some(pointer_capture_val) != pending_pointer_capture_val
    {
        let msg = CraftMessage::LostPointerCapture();
        let nodes: Vec<Rc<RefCell<dyn Element>>> = collect_nodes(&root);
        let target = find_pointer_capture_target(&nodes, &msg);

        if let Some(target) = target {
            let mut targets: VecDeque<Rc<RefCell<dyn Element>>> = VecDeque::new();
            let mut current_target = Some(Rc::clone(&target));
            while let Some(node) = current_target {
                targets.push_back(Rc::clone(&node));
                current_target = node.borrow().parent().as_ref().and_then(|p| p.upgrade());
            }

            event_dispatcher.dispatch_capturing_event(&msg, text_context, &mut targets);
            event_dispatcher.dispatch_bubbling_event(&msg, text_context, &mut targets);
        }
    }

    // 2. If the pending pointer capture target override for this pointer is set and is not equal to the pointer capture target override,
    // then fire a pointer event named gotpointercapture at the pending pointer capture target override.
    if let Some(pending_pointer_capture_val) = pending_pointer_capture_val
        && Some(pending_pointer_capture_val) != pointer_capture_val
    {
        let msg = CraftMessage::GotPointerCapture();
        let nodes: Vec<Rc<RefCell<dyn Element>>> = collect_nodes(&root);
        let target = find_pointer_capture_target(&nodes, &msg);

        if let Some(target) = target {
            let mut targets: VecDeque<Rc<RefCell<dyn Element>>> = VecDeque::new();
            let mut current_target = Some(Rc::clone(&target));
            while let Some(node) = current_target {
                targets.push_back(Rc::clone(&node));
                current_target = node.borrow().parent().as_ref().and_then(|p| p.upgrade());
            }

            event_dispatcher.dispatch_capturing_event(&msg, text_context, &mut targets);
            event_dispatcher.dispatch_bubbling_event(&msg, text_context, &mut targets);
        }
    }

    // 3. Set the pointer capture target override to the pending pointer capture target override, if set.
    // Otherwise, clear the pointer capture target override.
    DOCUMENTS.with_borrow_mut(|docs| {
        let current_doc = docs.get_current_document();

        if let Some(pending_pointer_capture_val) = pending_pointer_capture_val {
            current_doc.pointer_captures.insert(*key, pending_pointer_capture_val);
        } else {
            current_doc.pointer_captures.remove(key);
        }
    });
}

pub(super) fn maybe_handle_implicit_pointer_capture_release(
    event_dispatcher: &mut EventDispatcher,
    message: &CraftMessage,
    root: Rc<RefCell<dyn Element>>,
    text_context: &mut Option<TextContext>,
) {
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

        process_pending_pointer_capture(event_dispatcher, root, text_context);
    } else if message.is_pointer_event() && !message.is_got_or_lost_pointer_capture() {
        process_pending_pointer_capture(event_dispatcher, root, text_context);
    }
}
