use crate::events::{KeyboardInputHandler, PointerEventHandler, PointerUpdateHandler};
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
        DOCUMENTS.with_borrow_mut(|docs| {
            let current_doc = docs.get_current_document();
            current_doc.pointer_captures.insert(pointer_id, self.id());
        });
    }

    fn release_pointer_capture(&self, pointer_id: PointerId) {
        DOCUMENTS.with_borrow_mut(|docs| {
            let current_doc = docs.get_current_document();
            let _ = current_doc.pointer_captures.remove(&pointer_id);
        });
    }

}

