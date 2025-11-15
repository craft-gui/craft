use crate::events::{KeyboardInputHandler, PointerCaptureHandler, PointerEventHandler, PointerUpdateHandler};
use crate::style::Style;
use craft_primitives::geometry::ElementBox;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use kurbo::Point;
use ui_events::pointer::PointerId;
use crate::app::DOCUMENTS;
use crate::elements::core::ElementData;

/// The element trait for end-users.
pub trait Element : ElementData + crate::elements::core::ElementInternals {
    fn on_got_pointer_capture(&mut self, on_got_pointer_capture: PointerCaptureHandler) {
        self.element_data_mut().on_got_pointer_capture.push(on_got_pointer_capture);
    }

    fn on_lost_pointer_capture(&mut self, on_lost_pointer_capture: PointerCaptureHandler) {
        self.element_data_mut().on_lost_pointer_capture.push(on_lost_pointer_capture);
    }

    fn on_pointer_button_down(&mut self, on_pointer_button_down: PointerEventHandler) {
        self.element_data_mut().on_pointer_button_down.push(on_pointer_button_down);
    }

    fn on_pointer_button_up(&mut self, on_pointer_button_up: PointerEventHandler) {
        self.element_data_mut().on_pointer_button_up.push(on_pointer_button_up);
    }

    fn on_pointer_moved(&mut self, on_pointer_moved: PointerUpdateHandler) {
        self.element_data_mut().on_pointer_moved.push(on_pointer_moved);
    }

    fn on_keyboard_input(&mut self, on_keyboard_input: KeyboardInputHandler) {
        self.element_data_mut().on_keyboard_input.push(on_keyboard_input);
    }

    /// Returns the element's [`ElementBox`].
    fn computed_box_transformed(&self) -> ElementBox {
        self.element_data().layout_item.computed_box_transformed
    }

    /// Returns a shared reference to the element's [`Style`].
    fn style(&self) -> &Style {
        &self.element_data().style
    }

    /// Returns a mutable reference to the element's [`Style`].
    fn style_mut(&mut self) -> &mut Style {
        &mut self.element_data_mut().style
    }

    /// Determines if a point is within the bound of the element.
    ///
    /// Visual order and visibility shall not be accounted for.
    fn in_bounds(&self, point: Point) -> bool {
        let element_data = self.element_data();
        let rect = element_data.layout_item.computed_box_transformed.border_rectangle();

        if let Some(clip) = element_data.layout_item.clip_bounds {
            match rect.intersection(&clip) {
                Some(bounds) => bounds.contains(&point),
                None => false,
            }
        } else {
            rect.contains(&point)
        }
    }

    fn set_pointer_capture(&self, pointer_id: PointerId) {
        // 9.2 Setting pointer capture
        // https://w3c.github.io/pointerevents/#setting-pointer-capture

        DOCUMENTS.with_borrow_mut(|docs| {
            let current_doc = docs.get_current_document();

            // 1. If the pointerId provided as the method's argument does not match any of the active pointers, then throw a "NotFoundError" DOMException.
            // TODO (POINTER CAPTURE)
            // 2. Let the pointer be the active pointer specified by the given pointerId.
            // 3. If the element is not connected [DOM], throw an "InvalidStateError" DOMException.
            // TODO (POINTER CAPTURE)
            // 4. If this method is invoked while the element's node document [DOM] has a locked element ([PointerLock] pointerLockElement), throw an "InvalidStateError" DOMException.
            // TODO (POINTER CAPTURE)
            // 5. If the pointer is not in the active buttons state or the element's node document is not the active document of the pointer, then terminate these steps.
            // TODO (POINTER CAPTURE)
            // 6. For the specified pointerId, set the pending pointer capture target override to the Element on which this method was invoked.
            current_doc.pending_pointer_captures.insert(pointer_id, self.id());
        });
    }

    fn release_pointer_capture(&self, pointer_id: PointerId) {
        // 9.3 Releasing pointer capture
        // https://w3c.github.io/pointerevents/#releasing-pointer-capture
        let has_pointer_capture = self.has_pointer_capture(pointer_id);
        DOCUMENTS.with_borrow_mut(|docs| {
            let current_doc = docs.get_current_document();

            // 1. If the pointerId provided as the method's argument does not match any of the active pointers and these steps are not being invoked as a result of the implicit release of pointer capture, then throw a "NotFoundError" DOMException.
            // TODO (POINTER CAPTURE)
            // 2. If hasPointerCapture is false for the Element with the specified pointerId, then terminate these steps.
            if !has_pointer_capture {
                return;
            }
            // 3. For the specified pointerId, clear the pending pointer capture target override, if set.
            let _ = current_doc.pending_pointer_captures.remove(&pointer_id);
        });
    }

    fn has_pointer_capture(&self, pointer_id: PointerId) -> bool {
        // https://w3c.github.io/pointerevents/#dom-element-haspointercapture
        DOCUMENTS.with_borrow_mut(|docs| {
            let current_doc = docs.get_current_document();
            current_doc.pending_pointer_captures.get(&pointer_id).cloned() == Some(self.id())
        })
    }

}

