use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use crate::app::{ELEMENTS, FOCUS, GUMMY_TREE};
use crate::elements::scrollable::{ScrollState, draw_scrollbar};
use crate::elements::{ElementData, ScrollOptions, WindowInternal};
use crate::events::pointer_capture::PointerCapture;
use crate::events::{CheckboxToggledHandler, ClickHandler, DropdownItemSelectedHandler, Event, EventKind, KeyboardInputHandler, PointerCaptureHandler, PointerEnterHandler, PointerEventHandler, PointerLeaveHandler, PointerUpdateHandler, RadioValueChangedHandler, ScrollHandler, SliderValueChangedHandler, TextInputChangedHandler};
use crate::layout::GummyTree;
use crate::style::{AlignItems, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, Style, TextAlign, Underline, Unit};
use crate::text::text_context::TextContext;
use crate::{Color, RetGuiError};
use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Affine, ElementBox, Point, Rectangle, TrblRectangle};
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;
use ui_events::pointer::PointerId;

/// Internal element methods that should typically be ignored by users. Public for custom elements.
///
/// Drop is required to clean up any gummy nodes allocated by the element.
#[allow(drop_bounds)]
pub trait ElementInternals: ElementData + Any + Drop {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>>;

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
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        transform: Affine,
        text_context: &mut TextContext,
        scale_factor: f64,
        clip_bounds: Option<Rectangle>,
    ) {
        for child in &self.element_data().children {
            child.borrow_mut().apply_layout(
                gummy_tree,
                self.element_data().layout.computed_box.position,
                z_index,
                transform,
                text_context,
                clip_bounds,
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
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        for child in self.children() {
            child
                .borrow_mut()
                .draw(renderer, resource_manager.clone(), scale_factor, text_context);
        }
    }

    /// A helper to re-apply the style to the layout node when dirty.
    fn apply_style_to_layout_node_if_dirty(&mut self, gummy_tree: &mut GummyTree) {
        let element_data = self.element_data_mut();
        if element_data.style.is_dirty {
            let node_id = element_data.layout.gummy_node_id.unwrap();
            let style: gummy::Style = element_data.style.to_gummy_style();
            gummy_tree.set_style(node_id, style);
            element_data.style.is_dirty = false;
        }
    }

    /// Applies the layout results from the [`GummyTree`].
    /// This method retrieves the computed layout for `root_node` and updates the
    /// element’s internal state accordingly. It resolves the element's position,
    /// transform, clipping, borders, and stacking order, producing the final
    /// layout state used for rendering.
    ///
    /// # Parameters
    /// - `gummy_tree`: The layout tree containing the computed results.
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
        gummy_tree: &mut GummyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
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
        _renderer: &mut dyn Renderer,
        _resource_manager: Arc<ResourceManager>,
        _scale_factor: f64,
        _text_context: &mut TextContext,
    ) {
    }

    fn sync_accessibility_children(&mut self) {
        let data = self.element_data();
        let Some(node) = data.access_key else {
            return;
        };
        let children = data
            .children
            .iter()
            .filter_map(|child| child.borrow().element_data().access_key)
            .collect::<Vec<_>>();
        data.access_tree.set_children(node, &children);
    }

    /// Handles default events.
    fn on_event(&mut self, _message: &EventKind, _text_context: &mut TextContext, _event: &mut Event) {}

    /// Computes this element's box model.
    fn resolve_box(
        &mut self,
        relative_position: Point,
        scroll_transform: Affine,
        result: &gummy::Layout,
        layout_order: &mut u32,
    ) {
        let position = self.element_data().style.get_position();
        self.element_data_mut()
            .layout
            .resolve_box(relative_position, scroll_transform, result, layout_order, position);
    }

    /// Computes this element's clip box.
    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        self.element_data_mut().layout.apply_clip(clip_bounds);
    }

    fn apply_borders(&mut self, scale_factor: f64) {
        self.element_data_mut().apply_borders(scale_factor);
    }

    fn add_hit_testable(&mut self, renderer: &mut dyn Renderer, hit_testable: bool, scale_factor: f64) {
        if hit_testable {
            let id = self.element_data().internal_id;
            renderer.push_hit_testable(
                id,
                self.element_data()
                    .layout
                    .computed_box_transformed
                    .padding_rectangle()
                    .scale(scale_factor),
            );
        }
    }

    fn draw_borders(&self, renderer: &mut dyn Renderer, scale_factor: f64) {
        let current_style = self.element_data().style();

        self.element_data()
            .layout
            .draw_borders(renderer, current_style, scale_factor);
    }

    fn maybe_start_layer(&self, renderer: &mut dyn Renderer, scale_factor: f64) {
        let element_data = self.element_data();
        let padding_rectangle = element_data
            .layout
            .computed_box_transformed
            .padding_rectangle()
            .scale(scale_factor);

        if self.should_start_new_layer() {
            renderer.push_layer(padding_rectangle);
        }
    }

    fn maybe_end_layer(&self, renderer: &mut dyn Renderer) {
        if self.should_start_new_layer() {
            renderer.pop_layer();
        }
    }

    fn draw_scrollbar(&mut self, renderer: &mut dyn Renderer, scale_factor: f64) {
        let element_data = self.element_data();
        draw_scrollbar(&element_data.style, &element_data.layout, renderer, scale_factor);
    }

    fn should_start_new_layer(&self) -> bool {
        let element_data = self.element_data();

        element_data.style().get_overflow()[1] == Overflow::Scroll
    }

    /// Returns the element's [`ElementBox`] without any transforms applied.
    fn computed_box(&self) -> ElementBox {
        self.element_data().layout.computed_box.clone()
    }

    /// Gets
    fn get_default_style() -> Style
    where
        Self: Sized,
    {
        Style::new()
    }

    /// Mark layout node dirty.
    fn mark_dirty(&mut self) {
        let id = self.element_data().layout.gummy_node_id;
        if let Some(id) = id {
            GUMMY_TREE.with_borrow_mut(|gummy_tree| {
                gummy_tree.mark_dirty(id);
            });
        }
    }

    /// Updates gummy's style to reflect retgui's style struct.
    fn update_gummy_style(&mut self) {
        let id = self.element_data().layout.gummy_node_id;
        if let Some(id) = id {
            GUMMY_TREE.with_borrow_mut(|gummy_tree| {
                gummy_tree.set_style(id, self.element_data().style.to_gummy_style());
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

    fn get_first_child(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, RetGuiError> {
        self.children().first().cloned().ok_or(RetGuiError::ElementNotFound)
    }

    fn get_last_child(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, RetGuiError> {
        self.children().last().cloned().ok_or(RetGuiError::ElementNotFound)
    }

    fn get_previous_sibling(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, RetGuiError> {
        let parent = self.parent();
        let position = self.position_in_parent();

        if let Some(position) = position
            && let Some(parent) = parent.unwrap().upgrade()
        {
            if let Some(next_sibling) = parent.borrow().children().get(position - 1) {
                Ok(next_sibling.clone())
            } else {
                Err(RetGuiError::ElementNotFound)
            }
        } else {
            Err(RetGuiError::ElementNotFound)
        }
    }

    fn get_next_sibling(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, RetGuiError> {
        let parent = self.parent();
        let position = self.position_in_parent();

        if let Some(position) = position
            && let Some(parent) = parent.unwrap().upgrade()
        {
            if let Some(next_sibling) = parent.borrow().children().get(position + 1) {
                Ok(next_sibling.clone())
            } else {
                Err(RetGuiError::ElementNotFound)
            }
        } else {
            Err(RetGuiError::ElementNotFound)
        }
    }

    fn swap_child(
        &mut self,
        child_1: Rc<RefCell<dyn ElementInternals>>,
        child_2: Rc<RefCell<dyn ElementInternals>>,
    ) -> Result<(), RetGuiError> {
        let children = &mut self.element_data_mut().children;
        let position_1 = children
            .iter()
            .position(|x| Rc::ptr_eq(x, &child_1))
            .ok_or(RetGuiError::ElementNotFound)?;

        let position_2 = children
            .iter()
            .position(|x| Rc::ptr_eq(x, &child_2))
            .ok_or(RetGuiError::ElementNotFound)?;

        // Swap the children.
        self.element_data_mut().children.swap(position_1, position_2);

        // Swap the children's gummy nodes.
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let parent_id = self.element_data().layout.gummy_node_id;
            let child_1_id = child_1.borrow().element_data().layout.gummy_node_id;
            let child_2_id = child_2.borrow().element_data().layout.gummy_node_id;

            if let Some(parent_id) = parent_id
                && let Some(child_1_id) = child_1_id
                && let Some(child_2_id) = child_2_id
            {
                // There isn't a swap API in the gummy tree. Instead swap the children and call set_children.
                let mut tchildren = gummy_tree.children(parent_id).to_vec();

                let i1 = tchildren
                    .iter()
                    .position(|x| *x == child_1_id)
                    .ok_or(RetGuiError::ElementNotFound)
                    .expect("Failed to find gummy child");
                let i2 = tchildren
                    .iter()
                    .position(|x| *x == child_2_id)
                    .ok_or(RetGuiError::ElementNotFound)
                    .expect("Failed to find gummy child");

                tchildren.swap(i1, i2);

                gummy_tree.set_children(parent_id, &tchildren);
                gummy_tree.mark_dirty(parent_id);
                gummy_tree.request_layout();
            }
        });

        // TODO: Fix. This is likely doing more work than required.
        self.sync_accessibility_children();

        Ok(())
    }

    /// Removes a direct child of this element and returns the removed node.
    ///
    /// # Errors
    /// Returns [`RetGuiError::ElementNotFound`] if `child` is not an immediate child
    /// of this element.
    ///
    /// # Panics
    /// Panics if the corresponding Gummy layout nodes fail to be removed.
    fn remove_child(
        &mut self,
        child: Rc<RefCell<dyn ElementInternals>>,
    ) -> Result<Rc<RefCell<dyn ElementInternals>>, RetGuiError> {
        // Find the node.
        let children = &mut self.element_data_mut().children;
        let position = children
            .iter()
            .position(|x| Rc::ptr_eq(x, &child))
            .ok_or(RetGuiError::ElementNotFound)?;

        let child = children[position].clone();

        // Remove the node from the element.

        children.remove(position);

        // Remove the parent reference.
        child.borrow_mut().element_data_mut().parent = None;
        //child.borrow_mut().element_data_mut().window = None;
        child.borrow_mut().propagate_window_down();

        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let child_id = child.borrow().element_data().layout.gummy_node_id;

            if let Some(child_id) = child_id {
                gummy_tree.unparent_node(child_id);
            }

            let parent_id = self.element_data().layout.gummy_node_id;
            gummy_tree.mark_dirty(parent_id.unwrap());
        });

        // TODO: Move to document
        fn remove_element_from_document(node: Rc<RefCell<dyn ElementInternals>>, pointer_capture: &mut PointerCapture) {
            pointer_capture.remove_element(&node);
            for child in node.borrow().children() {
                remove_element_from_document(child.clone(), pointer_capture);
            }
        }

        if let Some(pointer_capture) = self.pointer_capture() {
            remove_element_from_document(child.clone(), &mut pointer_capture.borrow_mut());
        }

        child.borrow_mut().unfocus();

        crate::accessibility::detach_subtree(&mut *child.borrow_mut());

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

    /// Called after a node is added to the gummy tree.
    fn on_post_add_layout_tree(&mut self, _gummy_tree: &mut GummyTree) {}

    fn on_pointer_enter(&mut self, on_pointer_enter: PointerEnterHandler) {
        self.element_data_mut().on_pointer_enter.push(on_pointer_enter);
    }

    fn on_dropdown_item_selected(&mut self, on_dropdown_item_selected: DropdownItemSelectedHandler) {
        self.element_data_mut()
            .on_dropdown_item_selected
            .push(on_dropdown_item_selected);
    }

    fn on_slider_value_changed(&mut self, on_slider_value_changed: SliderValueChangedHandler) {
        self.element_data_mut()
            .on_slider_value_changed
            .push(on_slider_value_changed);
    }

    fn on_pointer_leave(&mut self, on_pointer_leave: PointerLeaveHandler) {
        self.element_data_mut().on_pointer_leave.push(on_pointer_leave);
    }

    fn on_radio_value_changed(&mut self, on_radio_value_changed: RadioValueChangedHandler) {
        self.element_data_mut()
            .on_radio_value_changed
            .push(on_radio_value_changed);
    }

    fn on_checkbox_toggled(&mut self, on_backbox_toggled: CheckboxToggledHandler) {
        self.element_data_mut().on_checkbox_toggled.push(on_backbox_toggled);
    }

    fn on_text_input_changed(&mut self, on_text_input_changed: TextInputChangedHandler) {
        self.element_data_mut()
            .on_text_input_changed
            .push(on_text_input_changed);
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

    fn get_id(&self) -> Option<smol_str::SmolStr> {
        self.element_data().id.clone()
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

    fn on_click(&mut self, on_click: ClickHandler) {
        self.element_data_mut().on_click.push(on_click);
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
        self.element_data().layout.scroll_state
    }

    /// Returns the element's [`ElementBox`].
    fn get_computed_box_transformed(&self) -> ElementBox {
        self.element_data().layout.computed_box_transformed.clone()
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
        let rect = element_data.layout.computed_box_transformed.border_rectangle();

        if let Some(clip) = element_data.layout.clip_bounds {
            match rect.intersection(&clip) {
                Some(bounds) => bounds.contains(&point),
                None => false,
            }
        } else {
            false
        }
    }

    fn pointer_capture(&self) -> Option<Rc<RefCell<PointerCapture>>> {
        let element_data = self.element_data();
        let window = element_data.window.clone();
        if let Some(window) = window {
            Some(window.upgrade().unwrap().borrow().pointer_capture.clone())
        } else {
            None
        }
    }

    fn propagate_window_down(&mut self) {
        let window = self.element_data().window.clone();
        for child in &self.element_data().children {
            let mut child_borrow = child.borrow_mut();
            child_borrow.element_data_mut().window = window.clone();
            child_borrow.propagate_window_down();
        }
    }

    fn set_pointer_capture(&self, pointer_id: PointerId) {
        // 9.2 Setting pointer capture
        // https://w3c.github.io/pointerevents/#setting-pointer-capture

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
        if let Some(pointer_capture) = self.pointer_capture() {
            pointer_capture
                .borrow_mut()
                .pending_pointer_captures
                .insert(pointer_id, self.element_data().me.clone());
        }
    }

    fn release_pointer_capture(&self, pointer_id: PointerId) {
        // 9.3 Releasing pointer capture
        // https://w3c.github.io/pointerevents/#releasing-pointer-capture
        let has_pointer_capture = self.has_pointer_capture(pointer_id);
        // 1. If the pointerId provided as the method's argument does not match any of the active pointers and these steps are not being invoked as a result of the implicit release of pointer capture, then throw a "NotFoundError" DOMException.
        // TODO (POINTER CAPTURE)
        // 2. If hasPointerCapture is false for the Element with the specified pointerId, then terminate these steps.
        if !has_pointer_capture {
            return;
        }
        // 3. For the specified pointerId, clear the pending pointer capture target override, if set.
        if let Some(pointer_capture) = self.pointer_capture() {
            pointer_capture
                .borrow_mut()
                .pending_pointer_captures
                .remove(&pointer_id);
        }
    }

    fn has_pointer_capture(&self, pointer_id: PointerId) -> bool {
        // https://w3c.github.io/pointerevents/#dom-element-haspointercapture
        if let Some(pointer_capture) = self.pointer_capture() {
            pointer_capture
                .borrow()
                .pending_pointer_captures
                .get(&pointer_id)
                .cloned()
                .map(|w| w.as_ptr())
                == Some(self.element_data().me.clone().as_ptr())
        } else {
            false
        }
    }

    fn set_display(&mut self, display: Display) {
        self.style_mut().set_display(display);
        self.update_gummy_style();
    }

    fn set_box_sizing(&mut self, box_sizing: BoxSizing) {
        self.style_mut().set_box_sizing(box_sizing);
        self.update_gummy_style();
    }

    fn set_position(&mut self, position: Position) {
        self.style_mut().set_position(position);
        self.update_gummy_style();
    }

    fn set_margin(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut()
            .set_margin(TrblRectangle::new(top, right, bottom, left));
        self.update_gummy_style();
    }

    fn set_margin_all(&mut self, value: Unit) {
        self.set_margin(value, value, value, value);
    }

    fn set_margin_horizontal(&mut self, value: Unit) {
        let margin = self.style().get_margin();
        self.set_margin(margin.top, value, margin.bottom, value);
    }

    fn set_margin_vertical(&mut self, value: Unit) {
        let margin = self.style().get_margin();
        self.set_margin(value, margin.right, value, margin.left);
    }

    fn set_padding(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut()
            .set_padding(TrblRectangle::new(top, right, bottom, left));
        self.update_gummy_style();
    }

    fn set_padding_all(&mut self, value: Unit) {
        self.set_padding(value, value, value, value);
    }

    fn set_padding_horizontal(&mut self, value: Unit) {
        let padding = self.style().get_padding();
        self.set_padding(padding.top, value, padding.bottom, value);
    }

    fn set_padding_vertical(&mut self, value: Unit) {
        let padding = self.style().get_padding();
        self.set_padding(value, padding.right, value, padding.left);
    }

    fn set_gap(&mut self, column_gap: Unit, row_gap: Unit) {
        self.style_mut().set_gap([column_gap, row_gap]);
        self.update_gummy_style();
    }

    fn set_row_gap(&mut self, value: Unit) {
        let column_gap = self.style().get_gap()[0];
        self.set_gap(column_gap, value);
    }

    fn set_column_gap(&mut self, value: Unit) {
        let row_gap = self.style().get_gap()[1];
        self.set_gap(value, row_gap);
    }

    fn set_inset(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut().set_inset(TrblRectangle::new(top, right, bottom, left));
        self.update_gummy_style();
    }

    fn set_min_width(&mut self, min_width: Unit) {
        self.style_mut().set_min_width(min_width);
        self.update_gummy_style();
    }

    fn set_min_height(&mut self, min_height: Unit) {
        self.style_mut().set_min_height(min_height);
        self.update_gummy_style();
    }

    fn set_width(&mut self, width: Unit) {
        self.style_mut().set_width(width);
        self.update_gummy_style();
    }

    fn set_height(&mut self, height: Unit) {
        self.style_mut().set_height(height);
        self.update_gummy_style();
    }

    fn set_max_width(&mut self, max_width: Unit) {
        self.style_mut().set_max_width(max_width);
        self.update_gummy_style();
    }

    fn set_max_height(&mut self, max_height: Unit) {
        self.style_mut().set_max_height(max_height);
        self.update_gummy_style();
    }

    fn set_wrap(&mut self, wrap: FlexWrap) {
        self.style_mut().set_wrap(wrap);
        self.update_gummy_style();
    }

    fn set_align_items(&mut self, align_items: Option<AlignItems>) {
        self.style_mut().set_align_items(align_items);
        self.update_gummy_style();
    }

    fn set_justify_content(&mut self, justify_content: Option<JustifyContent>) {
        self.style_mut().set_justify_content(justify_content);
        self.update_gummy_style();
    }

    fn set_flex_direction(&mut self, flex_direction: FlexDirection) {
        self.style_mut().set_flex_direction(flex_direction);
        self.update_gummy_style();
    }

    fn set_flex_grow(&mut self, flex_grow: f32) {
        self.style_mut().set_flex_grow(flex_grow);
        self.update_gummy_style();
    }

    fn set_flex_shrink(&mut self, flex_shrink: f32) {
        self.style_mut().set_flex_shrink(flex_shrink);
        self.update_gummy_style();
    }

    fn set_flex_basis(&mut self, flex_basis: Unit) {
        self.style_mut().set_flex_basis(flex_basis);
        self.update_gummy_style();
    }

    fn set_font_family(&mut self, font_family: FontFamily) {
        self.style_mut().set_font_family(font_family);
        self.update_gummy_style();
    }

    fn set_text_brush(&mut self, brush: Brush) {
        self.style_mut().set_text_brush(brush);
        self.update_gummy_style();
    }

    fn set_background_brush(&mut self, brush: Brush) {
        self.style_mut().set_background_brush(brush);
    }

    fn set_font_size(&mut self, font_size: f32) {
        self.style_mut().set_font_size(font_size);
        self.update_gummy_style();
    }

    fn set_line_height(&mut self, line_height: f32) {
        self.style_mut().set_line_height(line_height);
        self.update_gummy_style();
    }

    fn set_font_weight(&mut self, font_weight: FontWeight) {
        self.style_mut().set_font_weight(font_weight);
        self.update_gummy_style();
    }

    fn set_font_style(&mut self, font_style: FontStyle) {
        self.style_mut().set_font_style(font_style);
        self.update_gummy_style();
    }

    fn set_text_align(&mut self, text_align: TextAlign) {
        self.style_mut().set_text_align(text_align);
        self.update_gummy_style();
    }

    fn set_underline(&mut self, underline: Option<Underline>) {
        self.style_mut().set_underline(underline);
        self.update_gummy_style();
    }

    fn set_overflow(&mut self, overflow_x: Overflow, overflow_y: Overflow) {
        self.style_mut().set_overflow([overflow_x, overflow_y]);
        self.update_gummy_style();
    }

    fn set_overflow_x(&mut self, overflow: Overflow) {
        let overflow_y = self.style().get_overflow()[1];
        self.set_overflow(overflow, overflow_y);
    }

    fn set_overflow_y(&mut self, overflow: Overflow) {
        let overflow_x = self.style().get_overflow()[0];
        self.set_overflow(overflow_x, overflow);
    }

    fn set_border_color(&mut self, top: Color, right: Color, bottom: Color, left: Color) {
        self.style_mut()
            .set_border_color(TrblRectangle::new(top, right, bottom, left));
    }

    fn set_border_color_all(&mut self, value: Color) {
        self.set_border_color(value, value, value, value);
    }

    fn set_border_color_vertical(&mut self, value: Color) {
        let border_color = self.style().get_border_color();
        self.set_border_color(value, border_color.right, value, border_color.left);
    }

    fn set_border_color_horizontal(&mut self, value: Color) {
        let border_color = self.style().get_border_color();
        self.set_border_color(border_color.top, value, border_color.bottom, value);
    }

    fn set_border_width(&mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.style_mut()
            .set_border_width(TrblRectangle::new(top, right, bottom, left));
        self.update_gummy_style();
    }

    fn set_border_width_all(&mut self, value: Unit) {
        self.set_border_width(value, value, value, value);
    }

    fn set_border_width_vertical(&mut self, value: Unit) {
        let border_width = self.style().get_border_width();
        self.set_border_width(value, border_width.right, value, border_width.left);
    }

    fn set_border_width_horizontal(&mut self, value: Unit) {
        let border_width = self.style().get_border_width();
        self.set_border_width(border_width.top, value, border_width.bottom, value);
    }

    fn set_border_radius(&mut self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) {
        self.style_mut().set_border_radius([top, right, bottom, left]);
        self.update_gummy_style();
    }

    fn set_border_radius_all(&mut self, value: (f32, f32)) {
        self.set_border_radius(value, value, value, value);
    }

    fn set_border_radius_vertical(&mut self, value: (f32, f32)) {
        let border_radius = self.style().get_border_radius();
        self.set_border_radius(value, border_radius[1], value, border_radius[3]);
    }

    fn set_border_radius_horizontal(&mut self, value: (f32, f32)) {
        let border_radius = self.style().get_border_radius();
        self.set_border_radius(border_radius[0], value, border_radius[2], value);
    }

    fn set_scrollbar_color(&mut self, scrollbar_color: ScrollbarColor) {
        self.style_mut().set_scrollbar_brush(scrollbar_color);
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

    fn set_selection_brush(&mut self, selection_brush: Brush) {
        self.style_mut().set_selection_brush(selection_brush);
    }

    fn set_box_shadows(&mut self, box_shadows: Vec<BoxShadow>) {
        self.style_mut().set_box_shadows(box_shadows);
    }

    /// Sets focus on the specified element, if it can be focused.
    ///
    /// The focused element is the element that will receive keyboard and similar events by default.
    fn focus(&mut self) {
        // Todo: check if the element is focusable. Should we return a result?
        let me = self.element_data().me.clone();
        let _previous = FOCUS.with_borrow_mut(|focus| {
            let previous = focus.take();
            *focus = Some(me.clone());
            previous
        });
        {
            if let Some(previous) = _previous
                && !Weak::ptr_eq(&previous, &me)
                && let Some(previous) = previous.upgrade()
            {
                let previous = previous.borrow();
                let data = previous.element_data();
                if let Some(root) = data.access_root {
                    data.access_tree.set_focus(root, Some(root));
                }
            }

            let data = self.element_data();
            if let Some((root, node)) = data.access_root.zip(data.access_key) {
                data.access_tree.set_focus(root, Some(node));
            }
        }
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
            {
                let data = self.element_data();
                if let Some(root) = data.access_root {
                    data.access_tree.set_focus(root, Some(root));
                }
            }
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
        (root.borrow().deref() as &dyn Any)
            .downcast_ref::<WindowInternal>()
            .unwrap()
            .winit_window
            .clone()
    }

    /// Recursively prints the IDs of this element and all of its descendants.
    fn print_tree_ids(&self, depth: usize) {
        let indent = "  ".repeat(depth);

        // Access the ID from element_data.
        // If it's None, we can print "Unnamed Element" or the internal_id.
        let id_label = self.element_data().internal_id.to_string();

        println!("{}└─ {}: {}", indent, id_label, self.element_data().window.is_some());

        for child in self.children() {
            child.borrow().print_tree_ids(depth + 1);
        }
    }

    fn drop(&mut self) {
        for child in self.element_data().children.clone() {
            crate::accessibility::detach_subtree(&mut *child.borrow_mut());
        }
        if let Some(key) = self.element_data().access_key {
            self.element_data().access_tree.remove_node(key);
        }
        if let Some(gummy_node) = self.element_data().layout.gummy_node_id {
            GUMMY_TREE.with_borrow_mut(|gummy_tree| {
                gummy_tree.remove_node(gummy_node);
            });
        }
        ELEMENTS.with_borrow_mut(|elements| {
            elements.remove_id(self.element_data().internal_id);
        });
    }

    /// Use the element's window to request a redraw.
    fn request_window_redraw(&self) {
        let Some(winit_window_weak) = &self.element_data().window else {
            return;
        };
        let Some(rc) = winit_window_weak.upgrade() else {
            return;
        };
        let borrowed = rc.borrow();
        let Some(winit_window) = &borrowed.winit_window else {
            return;
        };
        winit_window.request_redraw();
    }
}

pub fn resolve_clip_for_scrollable(element: &mut dyn ElementInternals, clip_bounds: Option<Rectangle>) {
    element
        .element_data_mut()
        .layout
        .resolve_clip_for_scrollable(clip_bounds);
}
