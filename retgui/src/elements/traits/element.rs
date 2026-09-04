use std::any::Any;
use std::rc::Rc;

use retgui_primitives::Color;
use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::ElementBox;
use retgui_primitives::gradient::Gradient;

use smol_str::SmolStr;

use crate::RetGuiError;
use crate::elements::internal_helpers::push_child_to_element;
use crate::elements::scrollable::{ScrollOptions, ScrollState};
use crate::elements::{DynElement, ElementEditor, ElementNode, Elements};
use crate::events::{CheckboxToggledEvent, ClickEvent, CustomEvent, EventCallbackKind, EventKind, EventListenerOptions, FocusEvent, KeyboardEvent, PointerButtonEvent, PointerCaptureEvent, PointerEnterEvent, PointerId, PointerLeaveEvent, PointerMovedEvent, RadioValueChangedEvent, ScrollEvent, SliderValueChangedEvent, TextInputChangedEvent, UnfocusEvent};
use crate::style::{AlignContent, AlignItems, AlignSelf, Animation, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, TextAlign, Underline, Unit};

fn with_element<E: Element, R>(
    element: E,
    elements: &Elements,
    callback: impl FnOnce(&dyn ElementNode) -> R,
) -> Option<R> {
    elements.try_get(element.as_dyn_element()).map(callback)
}

fn with_element_mut<E: Element, R>(
    element: E,
    elements: &mut Elements,
    callback: impl FnOnce(&mut dyn ElementNode, &mut Elements) -> R,
) -> Option<R> {
    elements.try_dispatch_mut(element.as_dyn_element(), callback)
}

/// Exposes common element functionality likes styles and tree modifications.
pub trait Element: Copy {
    /// Returns an element as a DynElement.
    fn as_dyn_element(&self) -> DynElement;

    /// Bind elements while building.
    fn edit<'a>(&self, elements: &'a mut Elements) -> ElementEditor<'a, Self>
    where
        Self: Sized,
    {
        ElementEditor::new(*self, elements)
    }

    /// Requests a redraw of this element's owning window.
    fn request_redraw(&self, elements: &Elements) {
        if let Some(element) = elements.try_get(self.as_dyn_element()) {
            element.request_window_redraw();
        }
    }

    /// Returns the element's children.
    fn children(&self, elements: &Elements) -> Vec<DynElement> {
        with_element(*self, elements, |element| element.get_children().to_vec()).unwrap_or_default()
    }

    /// Returns the element's previous sibling or a not found error.
    fn previous_sibling(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
        with_element(*self, elements, |element| element.get_previous_sibling(elements))
            .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Returns the element's next sibling or a not found error.
    fn next_sibling(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
        with_element(*self, elements, |element| element.get_next_sibling(elements))
            .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Returns the element's parent or a not found error.
    fn parent(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
        with_element(*self, elements, |element| element.parent())
            .flatten()
            .ok_or(RetGuiError::ElementNotFound)
    }

    /// Returns the element's first child or a not found error.
    fn first_child(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
        with_element(*self, elements, |element| element.get_first_child()).unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Returns the element's last child or a not found error.
    fn last_child(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
        with_element(*self, elements, |element| element.get_last_child()).unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Removes the element's child or a not found error.
    fn remove_child(&self, elements: &mut Elements, child: DynElement) -> Result<DynElement, RetGuiError> {
        let handle = self.as_dyn_element();
        elements
            .try_dispatch_mut(handle, |element, elements| element.remove_child(elements, child))
            .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Detaches the element's children while keeping their handles valid.
    ///
    /// Use [`delete_all_children`](Self::delete_all_children) when the removed
    /// subtrees should be destroyed and their arena storage reclaimed.
    fn remove_all_children(&self, elements: &mut Elements) {
        let handle = self.as_dyn_element();
        elements.try_dispatch_mut(handle, |element, elements| element.remove_all_children(elements));
    }

    /// Deletes all direct children and their retained subtrees from the store.
    ///
    /// Unlike [`remove_all_children`](Self::remove_all_children), this
    /// invalidates every copy of each removed handle and reclaims its arena,
    /// layout, and accessibility storage.
    fn delete_all_children(&self, elements: &mut Elements) {
        let handle = self.as_dyn_element();
        if elements.contains(handle) {
            elements.delete_all_children(handle);
        }
    }

    /// Swaps the element's children or returns a not found error if either child is missing.
    fn swap_child(&self, elements: &mut Elements, child_1: DynElement, child_2: DynElement) -> Result<(), RetGuiError> {
        let handle = self.as_dyn_element();
        elements
            .try_dispatch_mut(handle, |element, elements| {
                element.swap_child(elements, child_1, child_2)
            })
            .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Pushes a child.
    fn push(&self, elements: &mut Elements, child: impl Element) {
        let child = child.as_dyn_element();
        push_child_to_element(elements, self.as_dyn_element(), child);
    }

    /// Adds an event listener.
    fn add_event_listener(&self, elements: &mut Elements, callback: EventCallbackKind, options: EventListenerOptions) {
        with_element_mut(*self, elements, |element, _elements| {
            element.add_event_listener(callback, options)
        });
    }

    /// Adds a pointer enter listener.
    fn add_pointer_enter_listener(
        &self,
        elements: &mut Elements,
        on_pointer_enter: impl Fn(&mut PointerEnterEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_pointer_enter(Rc::new(on_pointer_enter))
        });
    }

    /// Adds a pointer leave listener.
    fn add_pointer_leave_listener(
        &self,
        elements: &mut Elements,
        on_pointer_leave: impl Fn(&mut PointerLeaveEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_pointer_leave(Rc::new(on_pointer_leave))
        });
    }

    /// Adds a radio value changed listener.
    fn add_radio_value_changed_listener(
        &self,
        elements: &mut Elements,
        on_radio_value_changed: impl Fn(&mut RadioValueChangedEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_radio_value_changed(Rc::new(on_radio_value_changed))
        });
    }

    /// Adds a checkbox toggled listener.
    fn add_checkbox_toggled_listener(
        &self,
        elements: &mut Elements,
        on_checkbox_toggled: impl Fn(&mut CheckboxToggledEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_checkbox_toggled(Rc::new(on_checkbox_toggled))
        });
    }

    /// Adds a text input changed listener.
    fn add_text_input_changed_listener(
        &self,
        elements: &mut Elements,
        on_text_input_changed: impl Fn(&mut TextInputChangedEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_text_input_changed(Rc::new(on_text_input_changed))
        });
    }

    /// Sets the element's user based id.
    fn set_id(&self, elements: &mut Elements, id: &str) {
        with_element_mut(*self, elements, |element, _elements| element.set_id(id));
    }

    /// Sets the accessibility name.
    fn set_accessibility_name(&self, elements: &mut Elements, name: &str) {
        with_element_mut(*self, elements, |element, _elements| {
            element.element_data_mut().set_accessibility_name(name)
        });
    }

    /// Returns the element's user based id. This id is not used by RetGUI.
    fn id(&self, elements: &Elements) -> Option<SmolStr> {
        with_element(*self, elements, |element| element.get_id()).flatten()
    }

    /// Adds a pointer button down listener.
    fn add_pointer_button_down_listener(
        &self,
        elements: &mut Elements,
        on_pointer_button_down: impl Fn(&mut PointerButtonEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_pointer_button_down(Rc::new(on_pointer_button_down))
        });
    }

    /// Adds a pointer button moved listener.
    fn add_pointer_moved_listener(
        &self,
        elements: &mut Elements,
        on_pointer_moved: impl Fn(&mut PointerMovedEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_pointer_moved(Rc::new(on_pointer_moved))
        });
    }

    /// Adds a pointer button up listener.
    fn add_pointer_button_up_listener(
        &self,
        elements: &mut Elements,
        on_pointer_button_up: impl Fn(&mut PointerButtonEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_pointer_button_up(Rc::new(on_pointer_button_up))
        });
    }

    /// Adds a click listener.
    fn add_click_listener(&self, elements: &mut Elements, on_click: impl Fn(&mut ClickEvent, &mut Elements) + 'static) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_click(Rc::new(on_click))
        });
    }

    /// Adds a custom event listener.
    fn add_custom_event_listener(
        &self,
        elements: &mut Elements,
        on_custom_event: impl Fn(&mut CustomEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_custom_event(Rc::new(on_custom_event))
        });
    }

    /// Emits a custom event using the element as the target element.
    fn emit_custom_event<T: Any + 'static>(&self, elements: &mut Elements, detail: T) {
        let handle = self.as_dyn_element();
        if elements.contains(handle) {
            elements.queue_event(EventKind::Custom(CustomEvent::new(handle, detail)));
        }
    }

    /// Adds a focus event listener.
    fn add_focus_listener(&self, elements: &mut Elements, on_focus: impl Fn(&mut FocusEvent, &mut Elements) + 'static) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_focus(Rc::new(on_focus))
        });
    }

    /// Adds an unfocus event listener.
    fn add_unfocus_listener(
        &self,
        elements: &mut Elements,
        on_unfocus: impl Fn(&mut UnfocusEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_unfocus(Rc::new(on_unfocus))
        });
    }

    /// Adds a lost pointer capture event listener.
    fn add_lost_pointer_capture_listener(
        &self,
        elements: &mut Elements,
        on_lost_pointer_capture: impl Fn(&mut PointerCaptureEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_lost_pointer_capture(Rc::new(on_lost_pointer_capture))
        });
    }

    /// Adds a got pointer capture event listener.
    fn add_got_pointer_capture_listener(
        &self,
        elements: &mut Elements,
        on_got_pointer_capture: impl Fn(&mut PointerCaptureEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_got_pointer_capture(Rc::new(on_got_pointer_capture))
        });
    }

    /// Adds a keyboard input event listener.
    fn add_keyboard_input_listener(
        &self,
        elements: &mut Elements,
        on_keyboard_input: impl Fn(&mut KeyboardEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_keyboard_input(Rc::new(on_keyboard_input))
        });
    }

    /// Adds a slider value changed event listener.
    fn add_slider_value_changed_listener(
        &self,
        elements: &mut Elements,
        on_slider_value_changed: impl Fn(&mut SliderValueChangedEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_slider_value_changed(Rc::new(on_slider_value_changed))
        });
    }

    /// Adds a scroll event listener.
    fn add_scroll_listener(
        &self,
        elements: &mut Elements,
        on_scroll: impl Fn(&mut ScrollEvent, &mut Elements) + 'static,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.on_scroll(Rc::new(on_scroll))
        });
    }

    /// Scrolls to a child based on the child's user id.
    fn scroll_to_child_by_id(&self, elements: &mut Elements, id: &str) {
        let handle = self.as_dyn_element();
        elements.try_dispatch_mut(handle, |element, elements| {
            element.scroll_to_child_by_id_with_options(elements, id, ScrollOptions::default())
        });
    }

    /// Scrolls to a child based on the child's user id according to the scroll options.
    fn scroll_to_child_by_id_with_options(&self, elements: &mut Elements, id: &str, options: ScrollOptions) {
        let handle = self.as_dyn_element();
        elements.try_dispatch_mut(handle, |element, elements| {
            element.scroll_to_child_by_id_with_options(elements, id, options)
        });
    }

    /// Scrolls to a specific y value in logical pixels.
    fn scroll_to(&self, elements: &mut Elements, y: f32) {
        with_element_mut(*self, elements, |element, elements| element.scroll_to(elements, y));
    }

    /// Scrolls to the top of the element.
    fn scroll_to_top(&self, elements: &mut Elements) {
        with_element_mut(*self, elements, |element, elements| element.scroll_to_top(elements));
    }

    /// Scrolls to the button of the element.
    fn scroll_to_bottom(&self, elements: &mut Elements) {
        with_element_mut(*self, elements, |element, elements| element.scroll_to_bottom(elements));
    }

    /// Scrolls by a logical amount of pixels.
    fn scroll_by(&self, elements: &mut Elements, y: f32) {
        with_element_mut(*self, elements, |element, elements| element.scroll_by(elements, y));
    }

    /// Returns the elements current scroll state.
    fn scroll_state(&self, elements: &Elements) -> ScrollState {
        with_element(*self, elements, |element| element.get_scroll_state()).unwrap_or_default()
    }

    /// Sets the layout algorith e.g. block, flex, etc.
    fn set_display(&self, elements: &mut Elements, display: Display) {
        with_element_mut(*self, elements, |element, _elements| element.set_display(display));
    }

    /// Sets the box sizing e.g. content box/border box.
    fn set_box_sizing(&self, elements: &mut Elements, box_sizing: BoxSizing) {
        with_element_mut(*self, elements, |element, _elements| element.set_box_sizing(box_sizing));
    }

    /// Sets the position of the element.
    ///
    /// Unlike HTML, this has no effect on the visual order of the element.
    fn set_position(&self, elements: &mut Elements, position: Position) {
        with_element_mut(*self, elements, |element, _elements| element.set_position(position));
    }

    /// Puts the element on top of other elements.
    fn set_overlay(&self, elements: &mut Elements, overlay: bool) {
        with_element_mut(*self, elements, |element, _elements| element.set_overlay(overlay));
    }

    /// Returns if the element is put on top of other elements.
    fn is_overlay(&self, elements: &Elements) -> bool {
        with_element(*self, elements, |element| element.style().get_overlay()).unwrap_or(false)
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin(&self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_margin(top, right, bottom, left)
        });
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin_all(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_margin_all(value));
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin_vertical(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_margin_vertical(value));
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin_horizontal(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_margin_horizontal(value)
        });
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding(&self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_padding(top, right, bottom, left)
        });
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding_all(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_padding_all(value));
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding_vertical(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_padding_vertical(value)
        });
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding_horizontal(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_padding_horizontal(value)
        });
    }

    /// Sets the gap between children for flex/grid containers.
    fn set_gap(&self, elements: &mut Elements, row_gap: Unit, column_gap: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_gap(row_gap, column_gap)
        });
    }

    /// Sets the row gap between children for flex/grid containers.
    fn set_row_gap(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_row_gap(value));
    }

    /// Sets the column gap between children for flex/grid containers.
    fn set_column_gap(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_column_gap(value));
    }

    /// Align the element relative to its sides. Only applies to positioned elements.
    fn set_inset(&self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_inset(top, right, bottom, left)
        });
    }

    /// Sets the minium width of the element.
    fn set_min_width(&self, elements: &mut Elements, min_width: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_min_width(min_width));
    }

    /// Sets the minium height of the element.
    fn set_min_height(&self, elements: &mut Elements, min_height: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_min_height(min_height));
    }

    /// Sets the width of the element.
    fn set_width(&self, elements: &mut Elements, width: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_width(width));
    }

    /// Sets the height of the element.
    fn set_height(&self, elements: &mut Elements, height: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_height(height));
    }

    /// Sets the max width of the element.
    fn set_max_width(&self, elements: &mut Elements, max_width: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_max_width(max_width));
    }

    /// Sets the max height of the element.
    fn set_max_height(&self, elements: &mut Elements, max_height: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_max_height(max_height));
    }

    /// Sets the wrapping behavior for flex elements.
    fn set_wrap(&self, elements: &mut Elements, wrap: FlexWrap) {
        with_element_mut(*self, elements, |element, _elements| element.set_wrap(wrap));
    }

    /// Determines how flex/grid children are laid out on the cross axis.
    fn set_align_items(&self, elements: &mut Elements, align_items: AlignItems) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_align_items(align_items)
        });
    }

    /// Overrides a parent's align_items.
    fn set_align_self(&self, elements: &mut Elements, align_self: AlignSelf) {
        with_element_mut(*self, elements, |element, _elements| element.set_align_self(align_self));
    }

    fn set_align_content(&self, elements: &mut Elements, align_content: AlignContent) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_align_content(align_content)
        });
    }

    fn set_justify_content(&self, elements: &mut Elements, justify_content: JustifyContent) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_justify_content(justify_content)
        });
    }

    fn set_flex_direction(&self, elements: &mut Elements, flex_direction: FlexDirection) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_flex_direction(flex_direction)
        });
    }

    fn set_flex_grow(&self, elements: &mut Elements, flex_grow: f32) {
        with_element_mut(*self, elements, |element, _elements| element.set_flex_grow(flex_grow));
    }

    fn set_flex_shrink(&self, elements: &mut Elements, flex_shrink: f32) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_flex_shrink(flex_shrink)
        });
    }

    fn set_flex_basis(&self, elements: &mut Elements, flex_basis: Unit) {
        with_element_mut(*self, elements, |element, _elements| element.set_flex_basis(flex_basis));
    }

    fn set_order(&self, elements: &mut Elements, order: i32) {
        with_element_mut(*self, elements, |element, _elements| element.set_order(order));
    }

    fn set_font_family(&self, elements: &mut Elements, font_family: FontFamily) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_font_family(font_family)
        });
    }

    fn set_color(&self, elements: &mut Elements, color: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_text_brush(Brush::Color(color))
        });
    }

    fn set_text_gradient(&self, elements: &mut Elements, gradient: Gradient) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_text_brush(Brush::Gradient(gradient))
        });
    }

    fn set_background_color(&self, elements: &mut Elements, background_color: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_background_brush(Brush::Color(background_color))
        });
    }

    fn set_background_gradient(&self, elements: &mut Elements, gradient: Gradient) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_background_brush(Brush::Gradient(gradient))
        });
    }

    fn set_font_size(&self, elements: &mut Elements, font_size: f32) {
        with_element_mut(*self, elements, |element, _elements| element.set_font_size(font_size));
    }

    fn set_line_height(&self, elements: &mut Elements, line_height: f32) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_line_height(line_height)
        });
    }

    fn set_font_weight(&self, elements: &mut Elements, font_weight: FontWeight) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_font_weight(font_weight)
        });
    }

    fn set_font_style(&self, elements: &mut Elements, font_style: FontStyle) {
        with_element_mut(*self, elements, |element, _elements| element.set_font_style(font_style));
    }

    fn set_text_align(&self, elements: &mut Elements, text_align: TextAlign) {
        with_element_mut(*self, elements, |element, _elements| element.set_text_align(text_align));
    }

    fn set_underline(&self, elements: &mut Elements, thickness: Option<f32>, color: Color, offset: Option<f32>) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_underline(Some(Underline {
                thickness,
                brush: Brush::Color(color),
                offset,
            }))
        });
    }

    fn set_underline_gradient(
        &self,
        elements: &mut Elements,
        thickness: Option<f32>,
        gradient: Gradient,
        offset: Option<f32>,
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_underline(Some(Underline {
                thickness,
                brush: Brush::Gradient(gradient),
                offset,
            }))
        });
    }

    fn set_overflow(&self, elements: &mut Elements, overflow_x: Overflow, overflow_y: Overflow) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_overflow(overflow_x, overflow_y)
        });
    }

    fn set_overflow_x(&self, elements: &mut Elements, overflow_x: Overflow) {
        with_element_mut(*self, elements, |element, _elements| element.set_overflow_x(overflow_x));
    }

    fn set_overflow_y(&self, elements: &mut Elements, overflow_y: Overflow) {
        with_element_mut(*self, elements, |element, _elements| element.set_overflow_y(overflow_y));
    }

    fn set_border_color(&self, elements: &mut Elements, top: Color, right: Color, bottom: Color, left: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_color(top, right, bottom, left)
        });
    }

    fn set_border_color_all(&self, elements: &mut Elements, value: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_color_all(value)
        });
    }

    fn set_border_color_vertical(&self, elements: &mut Elements, value: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_color_vertical(value)
        });
    }

    fn set_border_color_horizontal(&self, elements: &mut Elements, value: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_color_horizontal(value)
        });
    }

    fn set_border_width(&self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_width(top, right, bottom, left)
        });
    }

    fn set_border_width_all(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_width_all(value)
        });
    }

    fn set_border_width_vertical(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_width_vertical(value)
        });
    }

    fn set_border_width_horizontal(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_width_horizontal(value)
        });
    }

    fn set_outline_color(&self, elements: &mut Elements, top: Color, right: Color, bottom: Color, left: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_outline_color(top, right, bottom, left)
        });
    }

    fn set_outline_color_all(&self, elements: &mut Elements, value: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_outline_color_all(value)
        });
    }

    fn set_outline_color_vertical(&self, elements: &mut Elements, value: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_outline_color_vertical(value)
        });
    }

    fn set_outline_color_horizontal(&self, elements: &mut Elements, value: Color) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_outline_color_horizontal(value)
        });
    }

    fn set_outline_width(&self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_outline_width(top, right, bottom, left)
        });
    }

    fn set_outline_width_all(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_outline_width_all(value)
        });
    }

    fn set_outline_width_vertical(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_outline_width_vertical(value)
        });
    }

    fn set_outline_width_horizontal(&self, elements: &mut Elements, value: Unit) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_outline_width_horizontal(value)
        });
    }

    fn set_border_radius(
        &self,
        elements: &mut Elements,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_radius(top, right, bottom, left)
        });
    }

    fn set_border_radius_all(&self, elements: &mut Elements, value: (f32, f32)) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_radius_all(value)
        });
    }

    fn set_border_radius_vertical(&self, elements: &mut Elements, value: (f32, f32)) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_radius_vertical(value)
        });
    }

    fn set_border_radius_horizontal(&self, elements: &mut Elements, value: (f32, f32)) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_border_radius_horizontal(value)
        });
    }

    fn set_scrollbar_color(&self, elements: &mut Elements, scrollbar_color: ScrollbarColor) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_scrollbar_color(scrollbar_color)
        });
    }

    fn set_scrollbar_thumb_margin(&self, elements: &mut Elements, top: f32, right: f32, bottom: f32, left: f32) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_scrollbar_thumb_margin(top, right, bottom, left)
        });
    }

    fn set_scrollbar_thumb_radius(
        &self,
        elements: &mut Elements,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_scrollbar_thumb_radius(top, right, bottom, left)
        });
    }

    fn set_scrollbar_width(&self, elements: &mut Elements, scrollbar_width: f32) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_scrollbar_width(scrollbar_width)
        });
    }

    /// Sets the list of animations.
    fn set_animations(&self, elements: &mut Elements, animations: Vec<Animation>) {
        with_element_mut(*self, elements, |element, elements| {
            element.set_animations(elements, animations)
        });
    }

    /// Sets the box shadows on this element.
    fn set_box_shadows(&self, elements: &mut Elements, box_shadows: Vec<BoxShadow>) {
        with_element_mut(*self, elements, |element, _elements| {
            element.set_box_shadows(box_shadows)
        });
    }

    /// Focus the element.
    fn focus(&self, elements: &mut Elements) {
        let handle = self.as_dyn_element();
        elements.try_dispatch_mut(handle, |element, elements| element.focus(elements));
    }

    /// Returns whether the element current has focus.
    fn is_focused(&self, elements: &Elements) -> bool {
        with_element(*self, elements, |element| element.is_focused()).unwrap_or(false)
    }

    /// Unfocuses the element.
    fn unfocus(&self, elements: &mut Elements) {
        let handle = self.as_dyn_element();
        elements.try_dispatch_mut(handle, |element, elements| element.unfocus(elements));
    }

    /// Get the elements box in logical pixels.
    fn computed_box_transformed(&self, elements: &Elements) -> ElementBox {
        with_element(*self, elements, |element| element.get_computed_box_transformed()).unwrap_or_default()
    }

    /// Returns whether the element has pointer capture.
    fn has_pointer_capture(&self, elements: &Elements, pointer_id: PointerId) -> bool {
        with_element(*self, elements, |element| {
            element.has_pointer_capture(elements, pointer_id)
        })
        .unwrap_or(false)
    }

    /// Captures subsequent events for this pointer on the element.
    fn set_pointer_capture(&self, elements: &mut Elements, pointer_id: PointerId) {
        with_element_mut(*self, elements, |element, elements| {
            element.set_pointer_capture(elements, pointer_id)
        });
    }

    /// Releases this element's capture of the pointer.
    fn release_pointer_capture(&self, elements: &mut Elements, pointer_id: PointerId) {
        with_element_mut(*self, elements, |element, elements| {
            element.release_pointer_capture(elements, pointer_id)
        });
    }
}
