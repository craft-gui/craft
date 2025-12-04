use crate::app::{DOCUMENTS, FOCUS, SPATIAL_TREE, SPATIAL_TREE_MAP, TAFFY_TREE};
use crate::elements::core::ElementData;
use crate::events::{KeyboardInputHandler, PointerCaptureHandler, PointerEnterHandler, PointerEventHandler, PointerLeaveHandler, PointerUpdateHandler};
use crate::layout::layout_context::LayoutContext;
use crate::style::{
    AlignItems, Display, FlexDirection, FontFamily, FontStyle, JustifyContent, ScrollbarColor, Style, Underline, Unit,
    Weight, Wrap,
};
use crate::CraftError;
use craft_primitives::geometry::Point;
use craft_primitives::geometry::{ElementBox, TrblRectangle};
use craft_primitives::Color;
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use taffy::{BoxSizing, NodeId, Overflow, Position, TaffyResult, TaffyTree};
use ui_events::pointer::PointerId;

/// The element trait for end-users.
pub trait Element: ElementData + crate::elements::core::ElementInternals + Any {
    fn swap_child(
        &mut self,
        child_1: Rc<RefCell<dyn Element>>,
        child_2: Rc<RefCell<dyn Element>>,
    ) -> Result<(), CraftError> {
        let children = &mut self.element_data_mut().children;
        let position_1 = children.iter().position(|x| Rc::ptr_eq(x, &child_1)).ok_or(CraftError::ElementNotFound)?;

        let position_2 = children.iter().position(|x| Rc::ptr_eq(x, &child_2)).ok_or(CraftError::ElementNotFound)?;

        // Swap the children.
        self.element_data_mut().children.swap(position_1, position_2);

        // Swap the children's taffy nodes.
        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let parent_id = self.element_data().layout_item.taffy_node_id;
            let child_1_id = child_1.borrow().element_data().layout_item.taffy_node_id;
            let child_2_id = child_2.borrow().element_data().layout_item.taffy_node_id;

            if let Some(parent_id) = parent_id
                && let Some(child_1_id) = child_1_id
                && let Some(child_2_id) = child_2_id
            {
                // There isn't a swap API in the taffy tree. Instead swap the children and call set_children.
                let mut tchildren = taffy_tree.children(parent_id).expect("Failed to get taffy children").to_vec();

                let i1 = tchildren
                    .iter()
                    .position(|x| *x == child_1_id)
                    .ok_or(CraftError::ElementNotFound)
                    .expect("Failed to find taffy child");
                let i2 = tchildren
                    .iter()
                    .position(|x| *x == child_2_id)
                    .ok_or(CraftError::ElementNotFound)
                    .expect("Failed to find taffy child");

                tchildren.swap(i1, i2);

                taffy_tree.set_children(parent_id, &tchildren).expect("Failed set taffy children");
                taffy_tree.mark_dirty(parent_id).expect("Failed to mark taffy node dirty.");
            }
        });

        Ok(())
    }

    /// Removes a direct child of this element and returns the removed node.
    ///
    /// # Errors
    /// Returns [`CraftError::ElementNotFound`] if `child` is not an immediate child
    /// of this element.
    ///
    /// # Panics
    /// Panics if the corresponding Taffy layout nodes fail to be removed.
    fn remove_child(&mut self, child: Rc<RefCell<dyn Element>>) -> Result<Rc<RefCell<dyn Element>>, CraftError> {
        // Find the node.
        let children = &mut self.element_data_mut().children;
        let position = children.iter().position(|x| Rc::ptr_eq(x, &child)).ok_or(CraftError::ElementNotFound)?;

        let child = children[position].clone();

        // Remove the node from the element.

        children.remove(position);

        // Remove the parent reference.

        child.borrow_mut().element_data_mut().parent = None;

        // Remove the entire layout subtree.

        fn remove_subtree(taffy: &mut TaffyTree<LayoutContext>, node: NodeId) -> TaffyResult<()> {
            // Can we avoid this allocation?
            let children = taffy.children(node)?;

            for child in children {
                remove_subtree(taffy, child)?;
            }

            taffy.remove(node).map(|_| ())
        }

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let child_id = child.borrow().element_data().layout_item.taffy_node_id;

            if let Some(child_id) = child_id {
                remove_subtree(taffy_tree, child_id).expect("Failed to remove taffy element.");
            }

            let parent_id = self.element_data().layout_item.taffy_node_id;
            taffy_tree.mark_dirty(parent_id.unwrap()).expect("Failed to mark taffy node dirty.");
        });

        if let Some(spatial_node) = child.borrow().element_data().layout_item.spatial_node_id {
            SPATIAL_TREE.with_borrow_mut(|spatial_tree| {
                spatial_tree.remove(spatial_node);
            });
            SPATIAL_TREE_MAP.with_borrow_mut(|spatial_tree_map| {
               spatial_tree_map.remove(&spatial_node);
            });
        }

        Ok(child)
    }

    /// Appends a child to the element.
    fn push(&mut self, _child: Rc<RefCell<dyn Element>>) -> &mut Self
    where
        Self: Sized,
    {
        panic!("Pushing children is not supported.")
    }

    fn push_dyn(&mut self, _child: Rc<RefCell<dyn Element>>) {
        panic!("Pushing children is not supported.")
    }

    /// Appends multiple children to the element.
    fn extend(&mut self, _children: impl IntoIterator<Item = Rc<RefCell<dyn Element>>>) -> &mut Self
    where
        Self: Sized,
    {
        panic!("")
    }

    fn on_pointer_enter(&mut self, on_pointer_enter: PointerEnterHandler) -> &mut Self
    where
        Self: Sized,
    {
        self.element_data_mut().on_pointer_enter.push(on_pointer_enter);
        self
    }

    fn on_pointer_leave(&mut self, on_pointer_leave: PointerLeaveHandler) -> &mut Self
    where
        Self: Sized,
    {
        self.element_data_mut().on_pointer_leave.push(on_pointer_leave);
        self
    }

    fn on_got_pointer_capture(&mut self, on_got_pointer_capture: PointerCaptureHandler) -> &mut Self
    where
        Self: Sized,
    {
        self.element_data_mut().on_got_pointer_capture.push(on_got_pointer_capture);
        self
    }

    fn on_lost_pointer_capture(&mut self, on_lost_pointer_capture: PointerCaptureHandler) -> &mut Self
    where
        Self: Sized,
    {
        self.element_data_mut().on_lost_pointer_capture.push(on_lost_pointer_capture);
        self
    }

    fn on_pointer_button_down(&mut self, on_pointer_button_down: PointerEventHandler) -> &mut Self
    where
        Self: Sized,
    {
        self.element_data_mut().on_pointer_button_down.push(on_pointer_button_down);
        self
    }

    fn on_pointer_button_up(&mut self, on_pointer_button_up: PointerEventHandler) -> &mut Self
    where
        Self: Sized,
    {
        self.element_data_mut().on_pointer_button_up.push(on_pointer_button_up);
        self
    }

    fn on_pointer_moved(&mut self, on_pointer_moved: PointerUpdateHandler) -> &mut Self
    where
        Self: Sized,
    {
        self.element_data_mut().on_pointer_moved.push(on_pointer_moved);
        self
    }

    fn on_keyboard_input(&mut self, on_keyboard_input: KeyboardInputHandler) -> &mut Self
    where
        Self: Sized,
    {
        self.element_data_mut().on_keyboard_input.push(on_keyboard_input);
        self
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
            current_doc.pending_pointer_captures.insert(pointer_id, self.element_data().me.clone().unwrap());
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
            current_doc.pending_pointer_captures.get(&pointer_id).cloned().map(|w| w.as_ptr()) == self.element_data().me.clone().map(|w|w.as_ptr())
        })
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn display(&mut self, display: Display) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_display(display);
        self
    }

    fn box_sizing(&mut self, box_sizing: BoxSizing) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_box_sizing(box_sizing);
        self
    }

    fn position(&mut self, position: Position) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_position(position);
        self
    }

    fn margin(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_margin(TrblRectangle::new(top, right, bottom, left));
        self
    }

    fn padding(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_padding(TrblRectangle::new(top, right, bottom, left));
        self
    }

    fn gap(&mut self, row_gap: Unit, column_gap: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_gap([row_gap, column_gap]);
        self
    }

    fn inset(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_inset(TrblRectangle::new(top, right, bottom, left));
        self
    }

    fn min_width(&mut self, min_width: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_min_width(min_width);
        self
    }

    fn min_height(&mut self, min_height: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_min_height(min_height);
        self
    }

    fn width(&mut self, width: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_width(width);
        self
    }

    fn height(&mut self, height: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_height(height);
        self
    }

    fn max_width(&mut self, max_width: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_max_width(max_width);
        self
    }

    fn max_height(&mut self, max_height: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_max_height(max_height);
        self
    }

    fn wrap(&mut self, wrap: Wrap) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_wrap(wrap);
        self
    }

    fn align_items(&mut self, align_items: Option<AlignItems>) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_align_items(align_items);
        self
    }

    fn justify_content(&mut self, justify_content: Option<JustifyContent>) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_justify_content(justify_content);
        self
    }

    fn flex_direction(&mut self, flex_direction: FlexDirection) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_flex_direction(flex_direction);
        self
    }

    fn flex_grow(&mut self, flex_grow: f32) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_flex_grow(flex_grow);
        self
    }

    fn flex_shrink(&mut self, flex_shrink: f32) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_flex_shrink(flex_shrink);
        self
    }

    fn flex_basis(&mut self, flex_basis: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_flex_basis(flex_basis);
        self
    }

    fn font_family(&mut self, font_family: FontFamily) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_font_family(font_family);
        self
    }

    fn color(&mut self, color: Color) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_color(color);
        self
    }

    fn background_color(&mut self, color: Color) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_background(color);
        self
    }

    fn font_size(&mut self, font_size: f32) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_font_size(font_size);
        self
    }

    fn line_height(&mut self, line_height: f32) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_line_height(line_height);
        self
    }

    fn font_weight(&mut self, font_weight: Weight) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_font_weight(font_weight);
        self
    }

    fn font_style(&mut self, font_style: FontStyle) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_font_style(font_style);
        self
    }

    fn underline(&mut self, underline: Option<Underline>) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_underline(underline);
        self
    }

    fn overflow(&mut self, overflow_x: Overflow, overflow_y: Overflow) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_overflow([overflow_x, overflow_y]);
        self
    }

    fn border_color(&mut self, top: Color, right: Color, bottom: Color, left: Color) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_border_color(TrblRectangle::new(top, right, bottom, left));
        self
    }

    fn border_width(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_border_width(TrblRectangle::new(top, right, bottom, left));
        self
    }

    fn border_radius(&mut self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_border_radius([top, right, bottom, left]);
        self
    }

    fn scrollbar_color(&mut self, scrollbar_color: ScrollbarColor) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_scrollbar_color(scrollbar_color);
        self
    }

    fn scrollbar_thumb_margin(&mut self, top: f32, right: f32, bottom: f32, left: f32) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_scrollbar_thumb_margin(TrblRectangle::new(top, right, bottom, left));
        self
    }

    fn scrollbar_thumb_radius(
        &mut self,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_scrollbar_thumb_radius([top, right, bottom, left]);
        self
    }

    fn scrollbar_width(&mut self, scrollbar_width: f32) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_scrollbar_width(scrollbar_width);
        self
    }

    fn selection_color(&mut self, selection_color: Color) -> &mut Self
    where
        Self: Sized,
    {
        self.style_mut().set_selection_color(selection_color);
        self
    }

    /// Sets focus on the specified element, if it can be focused.
    ///
    /// The focused element is the element that will receive keyboard and similar events by default.
    fn focus(&mut self) where
        Self: Sized,
    {
        // Todo: check if the element is focusable. Should we return a result?
        FOCUS.with_borrow_mut(|focus| {
            *focus = self.element_data().me.clone();
        });
    }

    /// Returns true if the element has focus.
    fn is_focused(&self) -> bool {
        let focus_element = FOCUS.with(|focus| {
           focus.borrow().clone()
        });

        if focus_element.is_none() {
            return false;
        }

        let focus_element = focus_element.unwrap();

        Weak::ptr_eq(&focus_element, self.element_data().me.as_ref().unwrap())
    }

    /// Removes focus if the element has focus.
    fn unfocus(&mut self) -> &mut Self where
        Self: Sized,
    {
        if self.is_focused() {
            FOCUS.with(|focus| {
                *focus.borrow_mut() = None;
            });
        }

        self
    }
}
