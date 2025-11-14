use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use kurbo::Point;
use ui_events::pointer::PointerId;
use craft_resource_manager::ResourceManager;
use crate::app::DOCUMENTS;
use crate::elements::Element;
use crate::events::{dispatch_event, CraftMessage, EventDispatchType};
use crate::text::text_context::TextContext;
use crate::WindowContext;

pub(super) fn processing_pending_pointer_capture(dispatch_type: EventDispatchType,
                                      _resource_manager: &mut Arc<ResourceManager>,
                                      mouse_position: Option<Point>,
                                      root: Rc<RefCell<dyn Element>>,
                                      text_context: &mut Option<TextContext>,
                                      window_context: &mut WindowContext,
                                      is_style: bool) {
    // 4.1.3.2 Process pending pointer capture
    let key = &PointerId::new(1).unwrap();
    let (pointer_capture_val, pending_pointer_capture_val) = DOCUMENTS.with_borrow_mut(|docs| {
    let current_doc = docs.get_current_document();
    let pointer_capture_val = current_doc.pointer_captures.get(key);
    let pending_pointer_capture_val = current_doc.pending_pointer_captures.get(key);

    return (pointer_capture_val.cloned(), pending_pointer_capture_val.cloned());
    });

    // 1. If the pointer capture target override for this pointer is set and is not equal to the pending pointer capture target override,
    // then fire a pointer event named lostpointercapture at the pointer capture target override node.
    if let Some(pointer_capture_val) = pointer_capture_val && Some(pointer_capture_val) != pending_pointer_capture_val {
        dispatch_event(&CraftMessage::LostPointerCapture(), dispatch_type.clone(), _resource_manager, mouse_position, Rc::clone(&root), text_context, window_context, is_style);
    }

    // 2. If the pending pointer capture target override for this pointer is set and is not equal to the pointer capture target override,
    // then fire a pointer event named gotpointercapture at the pending pointer capture target override.
    if let Some(pending_pointer_capture_val) = pending_pointer_capture_val && Some(pending_pointer_capture_val) != pointer_capture_val {
        dispatch_event(&CraftMessage::GotPointerCapture(), dispatch_type, _resource_manager, mouse_position, root, text_context, window_context, is_style);
    }

    // 3. Set the pointer capture target override to the pending pointer capture target override, if set.
    // Otherwise, clear the pointer capture target override.
    DOCUMENTS.with_borrow_mut(|docs| {
        let current_doc = docs.get_current_document();

        if let Some(pending_pointer_capture_val) = pending_pointer_capture_val {
            current_doc.pointer_captures.insert(*key, pending_pointer_capture_val);
        } else {
            let _ = current_doc.pointer_captures.remove(key);
        }
    });
}