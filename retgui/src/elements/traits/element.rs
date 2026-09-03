use std::any::Any;
use std::rc::Rc;

use retgui_primitives::Color;
use retgui_primitives::geometry::ElementBox;
use smol_str::SmolStr;

use retgui_primitives::brush::Brush;
use retgui_primitives::gradient::Gradient;

use crate::events::PointerId;

use crate::RetGuiError;
use crate::elements::internal_helpers::push_child_to_element;
use crate::elements::scrollable::{ScrollOptions, ScrollState};
use crate::elements::{DynElement, ElementEditor, ElementNode, Elements};
use crate::events::{CheckboxToggledEvent, ClickEvent, CustomEvent, EventCallbackKind, EventKind, EventListenerOptions, FocusEvent, KeyboardEvent, PointerButtonEvent, PointerCaptureEvent, PointerEnterEvent, PointerLeaveEvent, PointerMovedEvent, RadioValueChangedEvent, ScrollEvent, SliderValueChangedEvent, TextInputChangedEvent, UnfocusEvent};
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

/// A builder pattern for elements.
pub trait Element: Copy {
    /// Returns an element as a DynElement.
    fn as_dyn_element(&self) -> DynElement;

    /// Bind elements while building.
    fn edit(self, elements: &mut Elements) -> ElementEditor<'_, Self>
    where
        Self: Sized,
    {
        ElementEditor::new(self, elements)
    }

    /// Requests a redraw of this element's owning window.
    fn request_redraw(&self, elements: &Elements) {
        if let Some(element) = elements.try_get(self.as_dyn_element()) {
            element.request_window_redraw();
        }
    }

    /// Returns the element's children.
    fn get_children(&self, elements: &Elements) -> Vec<DynElement> {
        with_element(*self, elements, |element| element.get_children().to_vec()).unwrap_or_default()
    }

    /// Returns the element's previous sibling or a not found error.
    fn get_previous_sibling(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
        with_element(*self, elements, |element| element.get_previous_sibling(elements))
            .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Returns the element's next sibling or a not found error.
    fn get_next_sibling(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
        with_element(*self, elements, |element| element.get_next_sibling(elements))
            .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Returns the element's parent or a not found error.
    fn get_parent(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
        with_element(*self, elements, |element| element.parent())
            .flatten()
            .ok_or(RetGuiError::ElementNotFound)
    }

    /// Returns the element's first child or a not found error.
    fn get_first_child(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
        with_element(*self, elements, |element| element.get_first_child()).unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Returns the element's last child or a not found error.
    fn get_last_child(&self, elements: &Elements) -> Result<DynElement, RetGuiError> {
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
    fn push(self, elements: &mut Elements, child: impl Element) -> Self {
        let child = child.as_dyn_element();
        push_child_to_element(elements, self.as_dyn_element(), child);
        self
    }

    /// Adds an event listener.
    fn add_event_listener(
        self,
        elements: &mut Elements,
        callback: EventCallbackKind,
        options: EventListenerOptions,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.add_event_listener(callback, options)
        });
        self
    }

    /// Adds a pointer enter listener.
    fn on_pointer_enter(
        self,
        elements: &mut Elements,
        on_pointer_enter: impl Fn(&mut PointerEnterEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_pointer_enter(Rc::new(on_pointer_enter))
        });
        self
    }

    /// Adds a pointer leave listener.
    fn on_pointer_leave(
        self,
        elements: &mut Elements,
        on_pointer_leave: impl Fn(&mut PointerLeaveEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_pointer_leave(Rc::new(on_pointer_leave))
        });
        self
    }

    /// Adds a radio value changed listener.
    fn on_radio_value_changed(
        self,
        elements: &mut Elements,
        on_radio_value_changed: impl Fn(&mut RadioValueChangedEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_radio_value_changed(Rc::new(on_radio_value_changed))
        });
        self
    }

    /// Adds a checkbox toggled listener.
    fn on_checkbox_toggled(
        self,
        elements: &mut Elements,
        on_checkbox_toggled: impl Fn(&mut CheckboxToggledEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_checkbox_toggled(Rc::new(on_checkbox_toggled))
        });
        self
    }

    /// Adds a text intput changed listener.
    fn on_text_input_changed(
        self,
        elements: &mut Elements,
        on_text_input_changed: impl Fn(&mut TextInputChangedEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_text_input_changed(Rc::new(on_text_input_changed))
        });
        self
    }

    /// Sets the element's user based id.
    fn id(self, elements: &mut Elements, id: &str) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_id(id));
        self
    }

    /// Sets the accessibility name.
    fn accessibility_name(self, elements: &mut Elements, name: &str) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.element_data_mut().set_accessibility_name(name)
        });
        self
    }

    /// Returns the element's user based id. This id is not used by RetGUI.
    fn get_id(&self, elements: &Elements) -> Option<SmolStr> {
        with_element(*self, elements, |element| element.get_id()).flatten()
    }

    /// Adds a pointer button down listener.
    fn on_pointer_button_down(
        self,
        elements: &mut Elements,
        on_pointer_button_down: impl Fn(&mut PointerButtonEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_pointer_button_down(Rc::new(on_pointer_button_down))
        });
        self
    }

    /// Adds a pointer button moved listener.
    fn on_pointer_moved(
        self,
        elements: &mut Elements,
        on_pointer_moved: impl Fn(&mut PointerMovedEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_pointer_moved(Rc::new(on_pointer_moved))
        });
        self
    }

    /// Adds a pointer button up listener.
    fn on_pointer_button_up(
        self,
        elements: &mut Elements,
        on_pointer_button_up: impl Fn(&mut PointerButtonEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_pointer_button_up(Rc::new(on_pointer_button_up))
        });
        self
    }

    /// Adds a click listener.
    fn on_click(self, elements: &mut Elements, on_click: impl Fn(&mut ClickEvent, &mut Elements) + 'static) -> Self {
        with_element_mut(self, elements, |element, _elements| element.on_click(Rc::new(on_click)));
        self
    }

    /// Adds a custom event listener.
    fn on_custom_event(
        self,
        elements: &mut Elements,
        on_custom_event: impl Fn(&mut CustomEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_custom_event(Rc::new(on_custom_event))
        });
        self
    }

    /// Emits a custom event using the element as the target element.
    fn emit_custom_event<T: Any + 'static>(&self, elements: &mut Elements, detail: T) {
        let handle = self.as_dyn_element();
        if elements.contains(handle) {
            elements.queue_event(EventKind::Custom(CustomEvent::new(handle, detail)));
        }
    }

    /// Adds a focus event listener.
    fn on_focus(self, elements: &mut Elements, on_focus: impl Fn(&mut FocusEvent, &mut Elements) + 'static) -> Self {
        with_element_mut(self, elements, |element, _elements| element.on_focus(Rc::new(on_focus)));
        self
    }

    /// Adds an unfocus event listener.
    fn on_unfocus(
        self,
        elements: &mut Elements,
        on_unfocus: impl Fn(&mut UnfocusEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_unfocus(Rc::new(on_unfocus))
        });
        self
    }

    /// Adds a lost pointer capture event listener.
    fn on_lost_pointer_capture(
        self,
        elements: &mut Elements,
        on_lost_pointer_capture: impl Fn(&mut PointerCaptureEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_lost_pointer_capture(Rc::new(on_lost_pointer_capture))
        });
        self
    }

    /// Adds a got pointer capture event listener.
    fn on_got_pointer_capture(
        self,
        elements: &mut Elements,
        on_got_pointer_capture: impl Fn(&mut PointerCaptureEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_got_pointer_capture(Rc::new(on_got_pointer_capture))
        });
        self
    }

    /// Adds a keyboard input event listener.
    fn on_keyboard_input(
        self,
        elements: &mut Elements,
        on_keyboard_input: impl Fn(&mut KeyboardEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_keyboard_input(Rc::new(on_keyboard_input))
        });
        self
    }

    /// Adds a slider value changed event listener.
    fn on_slider_value_changed(
        self,
        elements: &mut Elements,
        on_slider_value_changed: impl Fn(&mut SliderValueChangedEvent, &mut Elements) + 'static,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_slider_value_changed(Rc::new(on_slider_value_changed))
        });
        self
    }

    /// Adds a scroll event listener.
    fn on_scroll(self, elements: &mut Elements, on_scroll: impl Fn(&mut ScrollEvent, &mut Elements) + 'static) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.on_scroll(Rc::new(on_scroll))
        });
        self
    }

    /// Scrolls to a child based on the child's user id.
    fn scroll_to_child_by_id(self, elements: &mut Elements, id: &str) -> Self {
        let handle = self.as_dyn_element();
        elements.try_dispatch_mut(handle, |element, elements| {
            element.scroll_to_child_by_id_with_options(elements, id, ScrollOptions::default())
        });
        self
    }

    /// Scrolls to a child based on the child's user id according to the scroll options.
    fn scroll_to_child_by_id_with_options(self, elements: &mut Elements, id: &str, options: ScrollOptions) -> Self {
        let handle = self.as_dyn_element();
        elements.try_dispatch_mut(handle, |element, elements| {
            element.scroll_to_child_by_id_with_options(elements, id, options)
        });
        self
    }

    /// Scrolls to a specific y value in logical pixels.
    fn scroll_to(self, elements: &mut Elements, y: f32) -> Self {
        with_element_mut(self, elements, |element, elements| element.scroll_to(elements, y));
        self
    }

    /// Scrolls to the top of the element.
    fn scroll_to_top(self, elements: &mut Elements) -> Self {
        with_element_mut(self, elements, |element, elements| element.scroll_to_top(elements));
        self
    }

    /// Scrolls to the button of the element.
    fn scroll_to_bottom(self, elements: &mut Elements) -> Self {
        with_element_mut(self, elements, |element, elements| element.scroll_to_bottom(elements));
        self
    }

    /// Scrolls by a logical amount of pixels.
    fn scroll_by(self, elements: &mut Elements, y: f32) -> Self {
        with_element_mut(self, elements, |element, elements| element.scroll_by(elements, y));
        self
    }

    /// Returns the elements current scroll state.
    fn get_scroll_state(&self, elements: &Elements) -> ScrollState {
        with_element(*self, elements, |element| element.get_scroll_state()).unwrap_or_default()
    }

    /// Sets the layout algorith e.g. block, flex, etc.
    fn display(self, elements: &mut Elements, display: Display) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_display(display));
        self
    }

    /// Sets the box sizing e.g. content box/border box.
    fn box_sizing(self, elements: &mut Elements, box_sizing: BoxSizing) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_box_sizing(box_sizing));
        self
    }

    /// Sets the position of the element.
    ///
    /// Unlike HTML, this has no effect on the visual order of the element.
    fn position(self, elements: &mut Elements, position: Position) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_position(position));
        self
    }

    /// Puts the element on top of other elements.
    fn overlay(self, elements: &mut Elements, overlay: bool) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_overlay(overlay));
        self
    }

    /// Returns if the element is put on top of other elements.
    fn get_overlay(&self, elements: &Elements) -> bool {
        with_element(*self, elements, |element| element.style().get_overlay()).unwrap_or(false)
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn margin(self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_margin(top, right, bottom, left)
        });
        self
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn margin_all(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_margin_all(value));
        self
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn margin_vertical(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_margin_vertical(value));
        self
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn margin_horizontal(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_margin_horizontal(value)
        });
        self
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn padding(self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_padding(top, right, bottom, left)
        });
        self
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn padding_all(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_padding_all(value));
        self
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn padding_vertical(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_padding_vertical(value));
        self
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn padding_horizontal(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_padding_horizontal(value)
        });
        self
    }

    /// Sets the gap between children for flex/grid containers.
    fn gap(self, elements: &mut Elements, row_gap: Unit, column_gap: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_gap(row_gap, column_gap)
        });
        self
    }

    /// Sets the row gap between children for flex/grid containers.
    fn row_gap(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_row_gap(value));
        self
    }

    /// Sets the column gap between children for flex/grid containers.
    fn column_gap(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_column_gap(value));
        self
    }

    /// Align the element relative to its sides. Only applies to positioned elements.
    fn inset(self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_inset(top, right, bottom, left)
        });
        self
    }

    /// Sets the minium width of the element.
    fn min_width(self, elements: &mut Elements, min_width: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_min_width(min_width));
        self
    }

    /// Sets the minium height of the element.
    fn min_height(self, elements: &mut Elements, min_height: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_min_height(min_height));
        self
    }

    /// Sets the width of the element.
    fn width(self, elements: &mut Elements, width: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_width(width));
        self
    }

    /// Sets the height of the element.
    fn height(self, elements: &mut Elements, height: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_height(height));
        self
    }

    /// Sets the max width of the element.
    fn max_width(self, elements: &mut Elements, max_width: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_max_width(max_width));
        self
    }

    /// Sets the max height of the element.
    fn max_height(self, elements: &mut Elements, max_height: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_max_height(max_height));
        self
    }

    /// Sets the wrapping behavior for flex elements.
    fn wrap(self, elements: &mut Elements, wrap: FlexWrap) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_wrap(wrap));
        self
    }

    /// Determines how flex/grid children are laid out on the cross axis.
    fn align_items(self, elements: &mut Elements, align_items: AlignItems) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_align_items(align_items)
        });
        self
    }

    /// Overrides a parent's align_items.
    fn align_self(self, elements: &mut Elements, align_self: AlignSelf) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_align_self(align_self));
        self
    }

    fn align_content(self, elements: &mut Elements, align_content: AlignContent) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_align_content(align_content)
        });
        self
    }

    fn justify_content(self, elements: &mut Elements, justify_content: JustifyContent) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_justify_content(justify_content)
        });
        self
    }

    fn flex_direction(self, elements: &mut Elements, flex_direction: FlexDirection) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_flex_direction(flex_direction)
        });
        self
    }

    fn flex_grow(self, elements: &mut Elements, flex_grow: f32) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_flex_grow(flex_grow));
        self
    }

    fn flex_shrink(self, elements: &mut Elements, flex_shrink: f32) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_flex_shrink(flex_shrink)
        });
        self
    }

    fn flex_basis(self, elements: &mut Elements, flex_basis: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_flex_basis(flex_basis));
        self
    }

    fn order(self, elements: &mut Elements, order: i32) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_order(order));
        self
    }

    fn font_family(self, elements: &mut Elements, font_family: FontFamily) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_font_family(font_family)
        });
        self
    }

    fn color(self, elements: &mut Elements, color: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_text_brush(Brush::Color(color))
        });
        self
    }

    fn text_gradient(self, elements: &mut Elements, gradient: Gradient) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_text_brush(Brush::Gradient(gradient))
        });
        self
    }

    fn background_color(self, elements: &mut Elements, background_color: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_background_brush(Brush::Color(background_color))
        });
        self
    }

    fn background_gradient(self, elements: &mut Elements, gradient: Gradient) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_background_brush(Brush::Gradient(gradient))
        });
        self
    }

    fn font_size(self, elements: &mut Elements, font_size: f32) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_font_size(font_size));
        self
    }

    fn line_height(self, elements: &mut Elements, line_height: f32) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_line_height(line_height)
        });
        self
    }

    fn font_weight(self, elements: &mut Elements, font_weight: FontWeight) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_font_weight(font_weight)
        });
        self
    }

    fn font_style(self, elements: &mut Elements, font_style: FontStyle) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_font_style(font_style));
        self
    }

    fn text_align(self, elements: &mut Elements, text_align: TextAlign) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_text_align(text_align));
        self
    }

    fn underline(self, elements: &mut Elements, thickness: Option<f32>, color: Color, offset: Option<f32>) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_underline(Some(Underline {
                thickness,
                brush: Brush::Color(color),
                offset,
            }))
        });

        self
    }

    fn underline_gradient(
        self,
        elements: &mut Elements,
        thickness: Option<f32>,
        gradient: Gradient,
        offset: Option<f32>,
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_underline(Some(Underline {
                thickness,
                brush: Brush::Gradient(gradient),
                offset,
            }))
        });

        self
    }

    fn overflow(self, elements: &mut Elements, overflow_x: Overflow, overflow_y: Overflow) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_overflow(overflow_x, overflow_y)
        });
        self
    }

    fn overflow_x(self, elements: &mut Elements, overflow_x: Overflow) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_overflow_x(overflow_x));
        self
    }

    fn overflow_y(self, elements: &mut Elements, overflow_y: Overflow) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_overflow_y(overflow_y));
        self
    }

    fn border_color(self, elements: &mut Elements, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_color(top, right, bottom, left)
        });
        self
    }

    fn border_color_all(self, elements: &mut Elements, value: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_border_color_all(value));
        self
    }

    fn border_color_vertical(self, elements: &mut Elements, value: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_color_vertical(value)
        });
        self
    }

    fn border_color_horizontal(self, elements: &mut Elements, value: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_color_horizontal(value)
        });
        self
    }

    fn border_width(self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_width(top, right, bottom, left)
        });
        self
    }

    fn border_width_all(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| element.set_border_width_all(value));
        self
    }

    fn border_width_vertical(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_width_vertical(value)
        });
        self
    }

    fn border_width_horizontal(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_width_horizontal(value)
        });
        self
    }

    fn outline_color(self, elements: &mut Elements, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_outline_color(top, right, bottom, left)
        });
        self
    }

    fn outline_color_all(self, elements: &mut Elements, value: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_outline_color_all(value)
        });
        self
    }

    fn outline_color_vertical(self, elements: &mut Elements, value: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_outline_color_vertical(value)
        });
        self
    }

    fn outline_color_horizontal(self, elements: &mut Elements, value: Color) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_outline_color_horizontal(value)
        });
        self
    }

    fn outline_width(self, elements: &mut Elements, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_outline_width(top, right, bottom, left)
        });
        self
    }

    fn outline_width_all(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_outline_width_all(value)
        });
        self
    }

    fn outline_width_vertical(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_outline_width_vertical(value)
        });
        self
    }

    fn outline_width_horizontal(self, elements: &mut Elements, value: Unit) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_outline_width_horizontal(value)
        });
        self
    }

    fn border_radius(
        self,
        elements: &mut Elements,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_radius(top, right, bottom, left)
        });
        self
    }

    fn border_radius_all(self, elements: &mut Elements, value: (f32, f32)) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_radius_all(value)
        });
        self
    }

    fn border_radius_vertical(self, elements: &mut Elements, value: (f32, f32)) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_radius_vertical(value)
        });
        self
    }

    fn border_radius_horizontal(self, elements: &mut Elements, value: (f32, f32)) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_border_radius_horizontal(value)
        });
        self
    }

    fn scrollbar_color(self, elements: &mut Elements, scrollbar_color: ScrollbarColor) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_scrollbar_color(scrollbar_color)
        });
        self
    }

    fn scrollbar_thumb_margin(self, elements: &mut Elements, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_scrollbar_thumb_margin(top, right, bottom, left)
        });
        self
    }

    fn scrollbar_thumb_radius(
        self,
        elements: &mut Elements,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_scrollbar_thumb_radius(top, right, bottom, left)
        });
        self
    }

    fn scrollbar_width(self, elements: &mut Elements, scrollbar_width: f32) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_scrollbar_width(scrollbar_width)
        });
        self
    }

    /// Sets the list of animations.
    fn animations(self, elements: &mut Elements, animations: Vec<Animation>) -> Self {
        with_element_mut(self, elements, |element, elements| {
            element.set_animations(elements, animations)
        });
        self
    }

    /// Sets the box shadows on this element.
    fn box_shadows(self, elements: &mut Elements, box_shadows: Vec<BoxShadow>) -> Self {
        with_element_mut(self, elements, |element, _elements| {
            element.set_box_shadows(box_shadows)
        });
        self
    }

    /// Focus the element.
    fn focus(self, elements: &mut Elements) -> Self {
        let handle = self.as_dyn_element();
        elements.try_dispatch_mut(handle, |element, elements| element.focus(elements));
        self
    }

    /// Returns whether the element current has focus.
    fn is_focused(&self, elements: &mut Elements) -> bool {
        with_element(*self, elements, |element| element.is_focused()).unwrap_or(false)
    }

    /// Unfocuses the element.
    fn unfocus(self, elements: &mut Elements) -> Self {
        let handle = self.as_dyn_element();
        elements.try_dispatch_mut(handle, |element, elements| element.unfocus(elements));
        self
    }

    /// Get the elements box in logical pixels.
    fn get_computed_box_transformed(&self, elements: &Elements) -> ElementBox {
        with_element(*self, elements, |element| element.get_computed_box_transformed()).unwrap_or_default()
    }

    /// Returns whether the element has pointer capture.
    fn has_pointer_capture(&self, elements: &mut Elements, pointer_id: PointerId) -> bool {
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
