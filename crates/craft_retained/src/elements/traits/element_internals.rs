use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use accesskit::{Action, Role};
use craft_primitives::geometry::borders::CssRoundedRect;
use craft_primitives::geometry::{ElementBox, Rectangle, TrblRectangle};
use craft_renderer::RenderList;
use kurbo::{Affine, Point, Vec2};
use peniko::Color;
use ui_events::pointer::PointerId;
use crate::app::{DOCUMENTS, ELEMENTS, FOCUS, TAFFY_TREE};
use crate::CraftError;
use crate::document::Document;
use crate::elements::{ElementData, ElementIdMap, ScrollOptions, WindowInternal};
use crate::elements::scrollable::ScrollState;
use crate::events::{CraftMessage, Event, KeyboardInputHandler, PointerCaptureHandler, PointerEnterHandler, PointerEventHandler, PointerLeaveHandler, PointerUpdateHandler, ScrollHandler, SliderValueChangedHandler};
use crate::layout::TaffyTree;
use crate::layout::layout_item::{CssComputedBorder, LayoutItem, draw_borders_generic};
use crate::style::{AlignItems, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, Style, Underline, Unit};
use crate::text::text_context::TextContext;

/// Internal element methods that should typically be ignored by users. Public for custom elements.
pub trait ElementInternals: ElementData + Any {
    fn position_in_parent(&self) -> Option<usize> {
        let parent = self.parent();

        // @OPTIMIZE: We are copying the vec here.
        if let Some(parent) = parent
            && let Some(parent) = parent.upgrade()
        {
            let me_ptr = self.element_data().me.clone().upgrade().unwrap();
            let children = parent.borrow_mut().element_data().children.clone();

            let self_position = children.iter().position(|x| Rc::ptr_eq(x, &me_ptr)).unwrap();

            Some(self_position)
        } else {
            None
        }
    }

    /// A helper to apply the layout for all children.
    #[allow(clippy::too_many_arguments)]
    fn apply_layout_children(
        &mut self,
        taffy_tree: &mut TaffyTree,
        z_index: &mut u32,
        transform: Affine,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        scale_factor: f64,
        _dirty: bool,
    ) {
        for child in &self.element_data().children {
            child.borrow_mut().apply_layout(
                taffy_tree,
                self.element_data().layout_item.computed_box.position,
                z_index,
                transform,
                pointer,
                text_context,
                self.element_data().layout_item.clip_bounds,
                scale_factor,
            );
        }
    }

    /// A helper to check if the element is visible.
    fn is_visible(&self) -> bool {
        let style = &self.element_data().style;
        style.get_visible() && style.get_display() != Display::None
    }

    /// A helper to draw all children.
    fn draw_children(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        pointer: Option<Point>,
        scale_factor: f64,
    ) {
        for child in self.children() {
            child.borrow_mut().draw(renderer, text_context, pointer, scale_factor);
        }
    }

    /// A helper to re-apply the style to the layout node when dirty.
    fn apply_style_to_layout_node_if_dirty(&mut self, taffy_tree: &mut TaffyTree) {
        let element_data = self.element_data_mut();
        if element_data.style.is_dirty {
            let node_id = element_data.layout_item.taffy_node_id.unwrap();
            let style: taffy::Style = element_data.style.to_taffy_style();
            taffy_tree.set_style(node_id, style);
            element_data.style.is_dirty = false;
        }
    }

    /// Applies the layout results from the [`TaffyTree`].
    /// This method retrieves the computed layout for `root_node` and updates the
    /// element’s internal state accordingly. It resolves the element's position,
    /// transform, clipping, borders, and stacking order, producing the final
    /// layout state used for rendering.
    ///
    /// # Parameters
    /// - `taffy_tree`: The layout tree containing the computed results.
    /// - `root_node`: The node whose layout information should be applied.
    /// - `position`: The absolute position of the element within its parent context.
    /// - `z_index`: A mutable counter used to assign stacking order as elements
    ///   are processed.
    /// - `transform`: The accumulated transform to apply to this element.
    /// - `pointer`: The current pointer position, if available, for hit-testing.
    /// - `text_context`: Context used for text layout and measurement.
    /// - `clip_bounds`: Optional clipping rectangle inherited from ancestors.
    ///
    /// # Effects
    /// This function mutates internal element state to reflect the final resolved
    /// layout and may trigger updates such as clipping regions, border geometry,
    /// and z-index assignment.
    #[allow(clippy::too_many_arguments)]
    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    );

    /// Draws the element and its visual contents.
    ///
    /// Implementations should use the provided [`RenderList`] to issue
    /// drawing commands.
    ///
    /// - `renderer`: the active render list to draw into.
    /// - `text_context`: text shaping and layout context.
    /// - `pointer`: optional pointer position for hover effects.
    /// - `window`: optional window handle.
    /// - `scale_factor`: scale factor.
    fn draw(
        &mut self,
        _renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _pointer: Option<Point>,
        _scale_factor: f64,
    ) {
    }

    /// Computes a [`TreeUpdate`] reflecting any accessibility changes.
    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    fn compute_accessibility_tree(
        &mut self,
        tree: &mut accesskit::TreeUpdate,
        parent_index: Option<usize>,
        scale_factor: f64,
    ) {
        let current_node_id = accesskit::NodeId(self.element_data().internal_id);

        let mut current_node = accesskit::Node::new(Role::GenericContainer);
        if !self.element_data().on_pointer_button_up.is_empty() {
            current_node.set_role(Role::Button);
            current_node.add_action(Action::Click);
        }

        let padding_box = self
            .element_data()
            .layout_item
            .computed_box_transformed
            .padding_rectangle()
            .scale(scale_factor);

        current_node.set_bounds(accesskit::Rect {
            x0: padding_box.left() as f64,
            y0: padding_box.top() as f64,
            x1: padding_box.right() as f64,
            y1: padding_box.bottom() as f64,
        });

        let current_index = tree.nodes.len(); // The current node is the last one added.

        if let Some(parent_index) = parent_index {
            let parent_node = tree.nodes.get_mut(parent_index).unwrap();
            parent_node.1.push_child(current_node_id);
        }

        tree.nodes.push((current_node_id, current_node));

        for child in self.element_data_mut().children.iter_mut() {
            child
                .borrow_mut()
                .compute_accessibility_tree(tree, Some(current_index), scale_factor);
        }
    }

    /// Handles default events.
    fn on_event(
        &mut self,
        _message: &CraftMessage,
        _text_context: &mut TextContext,
        _event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
    }

    /// Computes this element's box model.
    fn resolve_box(
        &mut self,
        relative_position: Point,
        scroll_transform: Affine,
        result: &taffy::Layout,
        layout_order: &mut u32,
    ) {
        let position = self.element_data().style.get_position();
        self.element_data_mut().layout_item.resolve_box(
            relative_position,
            scroll_transform,
            result,
            layout_order,
            position,
        );
    }

    /// Computes this element's clip box.
    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        self.element_data_mut().layout_item.resolve_clip(clip_bounds);
    }

    fn apply_borders(&mut self, scale_factor: f64) {
        let current_style = self.element_data().current_style();
        let has_border = current_style.has_border();
        let border_radius = current_style.get_border_radius();
        let border_color = &current_style.get_border_color();
        let box_shadows = current_style.get_box_shadows();

        self.element_data_mut()
            .layout_item
            .apply_borders(has_border, border_radius, scale_factor, border_color, box_shadows);
    }

    /// A bit of a hack to reset the layout item of an element recursively.
    fn reset_layout_item(&mut self) {
        self.element_data_mut().layout_item = LayoutItem::default();

        for child in self.element_data_mut().children.iter_mut() {
            child.borrow_mut().reset_layout_item();
        }
    }

    fn add_hit_testable(&mut self, renderer: &mut RenderList, hit_testable: bool, scale_factor: f64) {
        /*let ed = self.element_data().borrow();
        let has_events =
            !ed.on_pointer_button_up.is_empty() ||
            !ed.on_pointer_moved.is_empty() ||
            !ed.on_keyboard_input.is_empty() ||
            !ed.on_pointer_button_down.is_empty() ||
            !ed.on_got_pointer_capture.is_empty() ||
            !ed.on_pointer_enter.is_empty() ||
            !ed.on_pointer_leave.is_empty() ||
            !ed.on_lost_pointer_capture;*/
        if hit_testable {
            let id = self.element_data().internal_id;
            renderer.push_hit_testable(
                id,
                self.element_data()
                    .layout_item
                    .computed_box_transformed
                    .padding_rectangle()
                    .scale(scale_factor),
            );
        }
    }

    fn draw_borders(&self, renderer: &mut RenderList, scale_factor: f64) {
        let current_style = self.element_data().current_style();

        self.element_data()
            .layout_item
            .draw_borders(renderer, current_style, scale_factor);
    }

    fn maybe_start_layer(&self, renderer: &mut RenderList, scale_factor: f64) {
        let element_data = self.element_data();
        let padding_rectangle = element_data
            .layout_item
            .computed_box_transformed
            .padding_rectangle()
            .scale(scale_factor);

        if self.should_start_new_layer() {
            renderer.push_layer(padding_rectangle);
        }
    }

    fn maybe_end_layer(&self, renderer: &mut RenderList) {
        if self.should_start_new_layer() {
            renderer.pop_layer();
        }
    }

    fn draw_scrollbar(&mut self, renderer: &mut RenderList, scale_factor: f64) {
        if !self.element_data().is_scrollable() {
            return;
        }

        let border_color = self.element_data().current_style().get_border_color();
        let scrollbar_color = self.element_data().current_style().get_scrollbar_color();
        let scrollbar_thumb_radius = self
            .element_data()
            .current_style()
            .get_scrollbar_thumb_radius()
            .map(|radii| Vec2::new(radii.0 as f64, radii.1 as f64));
        // let scrollbar_thumb_radius = self.element_data().current_style().
        let track_rect = self
            .element_data_mut()
            .layout_item
            .computed_scroll_track
            .scale(scale_factor);
        let thumb_rect = self
            .element_data_mut()
            .layout_item
            .computed_scroll_thumb
            .scale(scale_factor);

        let border_spec = CssRoundedRect::new(thumb_rect.to_kurbo(), [0.0, 0.0, 0.0, 0.0], scrollbar_thumb_radius);
        let mut computed_border_spec = CssComputedBorder::new(border_spec);
        computed_border_spec.scale(scale_factor);

        renderer.draw_rect(track_rect, scrollbar_color.track_color);
        draw_borders_generic(
            renderer,
            &computed_border_spec,
            border_color.to_array(),
            scrollbar_color.thumb_color,
        );
    }

    fn should_start_new_layer(&self) -> bool {
        let element_data = self.element_data();

        element_data.current_style().get_overflow()[1] == Overflow::Scroll
    }

    /// Returns the element's [`ElementBox`] without any transforms applied.
    fn computed_box(&self) -> ElementBox {
        self.element_data().layout_item.computed_box
    }

    /// Gets
    fn get_default_style() -> Box<Style>
    where
        Self: Sized,
    {
        Style::new()
    }

    /// Mark layout node dirty.
    fn mark_dirty(&mut self) {
        let id = self.element_data().layout_item.taffy_node_id;
        if let Some(id) = id {
            TAFFY_TREE.with_borrow_mut(|taffy_tree| {
                taffy_tree.mark_dirty(id);
            });
        }
    }

    /// Updates taffy's style to reflect craft's style struct.
    fn update_taffy_style(&mut self) {
        let id = self.element_data().layout_item.taffy_node_id;
        if let Some(id) = id {
            TAFFY_TREE.with_borrow_mut(|taffy_tree| {
                taffy_tree.set_style(id, self.element_data().style.to_taffy_style());
            });
        }
    }

    /// Set's this element's scale factor. This should not be used to scale individual elements.
    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.apply_borders(scale_factor);
        for child in &self.element_data().children {
            child.borrow_mut().set_scale_factor(scale_factor);
        }
    }

    fn get_first_child(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        self.children().first().cloned().ok_or(CraftError::ElementNotFound)
    }

    fn get_last_child(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        self.children().last().cloned().ok_or(CraftError::ElementNotFound)
    }

    fn get_previous_sibling(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        let parent = self.parent();
        let position = self.position_in_parent();

        if let Some(position) = position
            && let Some(parent) = parent.unwrap().upgrade()
        {
            if let Some(next_sibling) = parent.borrow().children().get(position - 1) {
                Ok(next_sibling.clone())
            } else {
                Err(CraftError::ElementNotFound)
            }
        } else {
            Err(CraftError::ElementNotFound)
        }
    }

    fn get_next_sibling(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        let parent = self.parent();
        let position = self.position_in_parent();

        if let Some(position) = position
            && let Some(parent) = parent.unwrap().upgrade()
        {
            if let Some(next_sibling) = parent.borrow().children().get(position + 1) {
                Ok(next_sibling.clone())
            } else {
                Err(CraftError::ElementNotFound)
            }
        } else {
            Err(CraftError::ElementNotFound)
        }
    }

    fn swap_child(
        &mut self,
        child_1: Rc<RefCell<dyn ElementInternals>>,
        child_2: Rc<RefCell<dyn ElementInternals>>,
    ) -> Result<(), CraftError> {
        let children = &mut self.element_data_mut().children;
        let position_1 = children
            .iter()
            .position(|x| Rc::ptr_eq(x, &child_1))
            .ok_or(CraftError::ElementNotFound)?;

        let position_2 = children
            .iter()
            .position(|x| Rc::ptr_eq(x, &child_2))
            .ok_or(CraftError::ElementNotFound)?;

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
                let mut tchildren = taffy_tree.children(parent_id).to_vec();

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

                taffy_tree.set_children(parent_id, &tchildren);
                taffy_tree.mark_dirty(parent_id);
                taffy_tree.request_layout();
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
    fn remove_child(
        &mut self,
        child: Rc<RefCell<dyn ElementInternals>>,
    ) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        // Find the node.
        let children = &mut self.element_data_mut().children;
        let position = children
            .iter()
            .position(|x| Rc::ptr_eq(x, &child))
            .ok_or(CraftError::ElementNotFound)?;

        let child = children[position].clone();

        // Remove the node from the element.

        children.remove(position);

        // Remove the parent reference.

        child.borrow_mut().element_data_mut().parent = None;

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let child_id = child.borrow().element_data().layout_item.taffy_node_id;

            if let Some(child_id) = child_id {
                taffy_tree.remove_subtree(child_id);
            }

            let parent_id = self.element_data().layout_item.taffy_node_id;
            taffy_tree.mark_dirty(parent_id.unwrap());
        });

        // TODO: Move to document
        fn remove_element_from_document(
            node: Rc<RefCell<dyn ElementInternals>>,
            document: &mut Document,
            elements: &mut ElementIdMap,
        ) {
            elements.remove_id(node.borrow().element_data().internal_id);
            document
                .pointer_captures
                .retain(|_, v| !Weak::ptr_eq(v, &node.borrow().element_data().me));
            document
                .pending_pointer_captures
                .retain(|_, v| !Weak::ptr_eq(v, &node.borrow().element_data().me));
            for child in node.borrow().children() {
                remove_element_from_document(child.clone(), document, elements);
            }
        }

        DOCUMENTS.with_borrow_mut(|documents| {
            ELEMENTS.with_borrow_mut(|elements| {
                remove_element_from_document(child.clone(), documents.get_current_document(), elements);
            });
        });

        child.borrow_mut().unfocus();

        Ok(child)
    }

    fn remove_all_children(&mut self) {
        // @OPTIMIZE: We are copying the vec here.
        for child in self.element_data().children.clone().iter().rev() {
            self.remove_child(child.clone()).unwrap();
        }
    }

    fn push(&mut self, _child: Rc<RefCell<dyn ElementInternals>>) {
        panic!("Pushing children is not supported.")
    }

    fn on_pointer_enter(&mut self, on_pointer_enter: PointerEnterHandler) {
        self.element_data_mut().on_pointer_enter.push(on_pointer_enter);
    }

    fn on_slider_value_changed(&mut self, on_slider_value_changed: SliderValueChangedHandler) -> &mut Self
    where
        Self: Sized,
    {
        self.element_data_mut()
            .on_slider_value_changed
            .push(on_slider_value_changed);
        self
    }

    fn on_pointer_leave(&mut self, on_pointer_leave: PointerLeaveHandler) {
        self.element_data_mut().on_pointer_leave.push(on_pointer_leave);
    }

    fn on_got_pointer_capture(&mut self, on_got_pointer_capture: PointerCaptureHandler) {
        self.element_data_mut()
            .on_got_pointer_capture
            .push(on_got_pointer_capture);
    }

    fn on_lost_pointer_capture(&mut self, on_lost_pointer_capture: PointerCaptureHandler) {
        self.element_data_mut()
            .on_lost_pointer_capture
            .push(on_lost_pointer_capture);
    }

    fn set_id(&mut self, id: &str) {
        self.element_data_mut().id = Some(id.into());
    }

    fn on_pointer_button_down(&mut self, on_pointer_button_down: PointerEventHandler) {
        self.element_data_mut()
            .on_pointer_button_down
            .push(on_pointer_button_down);
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

    fn on_scroll(&mut self, on_scroll: ScrollHandler) {
        self.element_data_mut().on_scroll.push(on_scroll);
    }

    fn scroll_to_child_by_id_with_options(&mut self, id: &str, options: ScrollOptions) {
        crate::elements::scrollable::scroll_to_child_by_id_with_options(self.element_data_mut(), id, options);
    }

    fn scroll_to(&mut self, y: f32) {
        crate::elements::scrollable::scroll_to(self.element_data_mut(), y);
    }

    fn scroll_to_top(&mut self) {
        crate::elements::scrollable::scroll_to_top(self.element_data_mut());
    }

    fn scroll_to_bottom(&mut self) {
        crate::elements::scrollable::scroll_to_bottom(self.element_data_mut());
    }

    fn scroll_by(&mut self, y: f32) {
        crate::elements::scrollable::scroll_by(self.element_data_mut(), y);
    }

    fn get_scroll_state(&self) -> ScrollState {
        self.element_data().layout_item.scroll_state
    }

    /// Returns the element's [`ElementBox`].
    fn get_computed_box_transformed(&self) -> ElementBox {
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
            current_doc
                .pending_pointer_captures
                .insert(pointer_id, self.element_data().me.clone());
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
            current_doc
                .pending_pointer_captures
                .get(&pointer_id)
                .cloned()
                .map(|w| w.as_ptr())
                == Some(self.element_data().me.clone().as_ptr())
        })
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn set_display(&mut self, display: Display) {
        self.style_mut().set_display(display);
        self.update_taffy_style();
    }

    fn set_box_sizing(&mut self, box_sizing: BoxSizing) {
        self.style_mut().set_box_sizing(box_sizing);
        self.update_taffy_style();
    }

    fn set_position(&mut self, position: Position) {
        self.style_mut().set_position(position);
        self.update_taffy_style();
    }

    fn set_margin(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut()
            .set_margin(TrblRectangle::new(top, right, bottom, left));
        self.update_taffy_style();
    }

    fn set_padding(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut()
            .set_padding(TrblRectangle::new(top, right, bottom, left));
        self.update_taffy_style();
    }

    fn set_gap(&mut self, row_gap: Unit, column_gap: Unit) {
        self.style_mut().set_gap([row_gap, column_gap]);
        self.update_taffy_style();
    }

    fn set_inset(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut().set_inset(TrblRectangle::new(top, right, bottom, left));
        self.update_taffy_style();
    }

    fn set_min_width(&mut self, min_width: Unit) {
        self.style_mut().set_min_width(min_width);
        self.update_taffy_style();
    }

    fn set_min_height(&mut self, min_height: Unit) {
        self.style_mut().set_min_height(min_height);
        self.update_taffy_style();
    }

    fn set_width(&mut self, width: Unit) {
        self.style_mut().set_width(width);
        self.update_taffy_style();
    }

    fn set_height(&mut self, height: Unit) {
        self.style_mut().set_height(height);
        self.update_taffy_style();
    }

    fn set_max_width(&mut self, max_width: Unit) {
        self.style_mut().set_max_width(max_width);
        self.update_taffy_style();
    }

    fn set_max_height(&mut self, max_height: Unit) {
        self.style_mut().set_max_height(max_height);
        self.update_taffy_style();
    }

    fn set_wrap(&mut self, wrap: FlexWrap) {
        self.style_mut().set_wrap(wrap);
        self.update_taffy_style();
    }

    fn set_align_items(&mut self, align_items: Option<AlignItems>) {
        self.style_mut().set_align_items(align_items);
        self.update_taffy_style();
    }

    fn set_justify_content(&mut self, justify_content: Option<JustifyContent>) {
        self.style_mut().set_justify_content(justify_content);
        self.update_taffy_style();
    }

    fn set_flex_direction(&mut self, flex_direction: FlexDirection) {
        self.style_mut().set_flex_direction(flex_direction);
        self.update_taffy_style();
    }

    fn set_flex_grow(&mut self, flex_grow: f32) {
        self.style_mut().set_flex_grow(flex_grow);
        self.update_taffy_style();
    }

    fn set_flex_shrink(&mut self, flex_shrink: f32) {
        self.style_mut().set_flex_shrink(flex_shrink);
        self.update_taffy_style();
    }

    fn set_flex_basis(&mut self, flex_basis: Unit) {
        self.style_mut().set_flex_basis(flex_basis);
        self.update_taffy_style();
    }

    fn set_font_family(&mut self, font_family: FontFamily) {
        self.style_mut().set_font_family(font_family);
        self.update_taffy_style();
    }

    fn set_color(&mut self, color: Color) {
        self.style_mut().set_color(color);
        self.update_taffy_style();
    }

    fn set_background_color(&mut self, color: Color) {
        self.style_mut().set_background_color(color);
    }

    fn set_font_size(&mut self, font_size: f32) {
        self.style_mut().set_font_size(font_size);
        self.update_taffy_style();
    }

    fn set_line_height(&mut self, line_height: f32) {
        self.style_mut().set_line_height(line_height);
        self.update_taffy_style();
    }

    fn set_font_weight(&mut self, font_weight: FontWeight) {
        self.style_mut().set_font_weight(font_weight);
        self.update_taffy_style();
    }

    fn set_font_style(&mut self, font_style: FontStyle) {
        self.style_mut().set_font_style(font_style);
        self.update_taffy_style();
    }

    fn set_underline(&mut self, underline: Option<Underline>) {
        self.style_mut().set_underline(underline);
        self.update_taffy_style();
    }

    fn set_overflow(&mut self, overflow_x: Overflow, overflow_y: Overflow) {
        self.style_mut().set_overflow([overflow_x, overflow_y]);
        self.update_taffy_style();
    }

    fn set_border_color(&mut self, top: Color, right: Color, bottom: Color, left: Color) {
        self.style_mut()
            .set_border_color(TrblRectangle::new(top, right, bottom, left));
    }

    fn set_border_width(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut()
            .set_border_width(TrblRectangle::new(top, right, bottom, left));
        self.update_taffy_style();
    }

    fn border_radius(&mut self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> &mut Self
    where
        Self: Sized,
    {
        self.set_border_radius(top, right, bottom, left);
        self
    }

    fn set_border_radius(&mut self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) {
        self.style_mut().set_border_radius([top, right, bottom, left]);
    }

    fn set_scrollbar_color(&mut self, scrollbar_color: ScrollbarColor) {
        self.style_mut().set_scrollbar_color(scrollbar_color);
    }

    fn set_scrollbar_thumb_margin(&mut self, top: f32, right: f32, bottom: f32, left: f32) {
        self.style_mut()
            .set_scrollbar_thumb_margin(TrblRectangle::new(top, right, bottom, left));
    }

    fn set_scrollbar_thumb_radius(&mut self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) {
        self.style_mut().set_scrollbar_thumb_radius([top, right, bottom, left]);
    }

    fn set_scrollbar_width(&mut self, scrollbar_width: f32) {
        self.style_mut().set_scrollbar_width(scrollbar_width);
    }

    fn set_selection_color(&mut self, selection_color: Color) {
        self.style_mut().set_selection_color(selection_color);
    }

    fn set_box_shadows(&mut self, box_shadows: Vec<BoxShadow>) {
        self.style_mut().set_box_shadows(box_shadows);
    }

    /// Sets focus on the specified element, if it can be focused.
    ///
    /// The focused element is the element that will receive keyboard and similar events by default.
    fn focus(&mut self) {
        // Todo: check if the element is focusable. Should we return a result?
        FOCUS.with_borrow_mut(|focus| {
            *focus = Some(self.element_data().me.clone());
        });
    }

    /// Returns true if the element has focus.
    fn is_focused(&self) -> bool {
        let focus_element = FOCUS.with(|focus| focus.borrow().clone());

        if focus_element.is_none() {
            return false;
        }

        let focus_element = focus_element.unwrap();

        Weak::ptr_eq(&focus_element, &self.element_data().me)
    }

    /// Removes focus if the element has focus.
    fn unfocus(&mut self) {
        if self.is_focused() {
            FOCUS.with(|focus| {
                *focus.borrow_mut() = None;
            });
        }
    }

    /// Re-
    fn to_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.element_data().me.upgrade().unwrap()
    }

    /// Returns the root element.
    fn get_root_element(&self) -> Weak<RefCell<dyn ElementInternals>> {
        let mut root_ancestor: Weak<RefCell<dyn ElementInternals>> = self.element_data().me.clone();
        loop {
            let me = root_ancestor.upgrade().unwrap();
            if let Some(parent) = me.borrow().parent() {
                root_ancestor = parent;
            } else {
                break;
            }
        }
        root_ancestor
    }

    /// Gets the winit window of this element.
    ///
    /// This will panic if the element does not have a window as its root.
    fn get_winit_window(&self) -> Option<Arc<winit::window::Window>> {
        let root = self.get_root_element().upgrade().unwrap();
        root.borrow()
            .as_any()
            .downcast_ref::<WindowInternal>()
            .unwrap()
            .winit_window
            .clone()
    }
}

pub fn resolve_clip_for_scrollable(element: &mut dyn ElementInternals, clip_bounds: Option<Rectangle>) {
    let element_data = element.element_data_mut();
    if element_data.is_scrollable() {
        let scroll_clip_bounds = element_data.layout_item.computed_box_transformed.padding_rectangle();
        if let Some(clip_bounds) = clip_bounds {
            element_data.layout_item.clip_bounds = scroll_clip_bounds.intersection(&clip_bounds);
        } else {
            element_data.layout_item.clip_bounds = Some(scroll_clip_bounds);
        }
    } else {
        element_data.layout_item.clip_bounds = clip_bounds;
    }
}
