use std::any::Any;
use std::rc::Rc;

use retgui_primitives::Color;
use retgui_primitives::geometry::ElementBox;
use smol_str::SmolStr;

use retgui_primitives::brush::Brush;
use retgui_primitives::gradient::Gradient;

use crate::events::PointerId;

use crate::RetGuiError;
use crate::app::queue_event;
use crate::elements::scrollable::{ScrollOptions, ScrollState};
use crate::elements::{AsElement, DynElement};
use crate::events::{CheckboxToggledEvent, ClickEvent, CustomEvent, EventCallbackKind, EventKind, EventListenerOptions, FocusEvent, KeyboardEvent, PointerButtonEvent, PointerCaptureEvent, PointerEnterEvent, PointerLeaveEvent, PointerMovedEvent, RadioValueChangedEvent, ScrollEvent, SliderValueChangedEvent, TextInputChangedEvent, UnfocusEvent};
use crate::style::{AlignContent, AlignItems, AlignSelf, Animation, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, TextAlign, Underline, Unit};

/// Exposes a fluent/builder-pattern like API for elements.
/// Setters in this trait return Self and have no prefix.
/// Getters in this trait return specific data and have a get prefix.
pub trait Element: Clone + AsElement {

    /// Returns the element's children.
    fn get_children(&self) -> Vec<DynElement> {
        self.with(|element| element.get_children().to_vec())
    }

    /// Returns the element's previous sibling or a not found error.
    fn get_previous_sibling(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.get_previous_sibling())
    }

    /// Returns the element's next sibling or a not found error.
    fn get_next_sibling(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.get_next_sibling())
    }

    /// Returns the element's parent or a not found error.
    fn get_parent(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.parent())
            .ok_or(RetGuiError::ElementNotFound)
    }

    /// Returns the element's first child or a not found error.
    fn get_first_child(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.get_first_child())
    }

    /// Returns the element's last child or a not found error.
    fn get_last_child(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.get_last_child())
    }

    /// Removes the element's child or a not found error.
    fn remove_child(&self, child: DynElement) -> Result<DynElement, RetGuiError> {
        self.with_mut(|element| element.remove_child(child))
    }

    /// Removes the element's children.
    fn remove_all_children(&self) {
        self.with_mut(|element| element.remove_all_children())
    }

    /// Swaps the element's children or returns a not found error if either child is missing.
    fn swap_child(&self, child_1: DynElement, child_2: DynElement) -> Result<(), RetGuiError> {
        self.with_mut(|element| element.swap_child(child_1, child_2))
    }

    /// Pushes a child.
    fn push(self, child: impl AsElement) -> Self {
        let child = child.with(|element| element.to_dyn_element());
        self.with_mut(|element| element.push(child));
        self
    }

    /// Adds an event listener.
    fn add_event_listener(self, callback: EventCallbackKind, options: EventListenerOptions) -> Self {
        self.with_mut(|element| element.add_event_listener(callback, options));
        self
    }

    /// Adds a pointer enter listener.
    fn on_pointer_enter(self, on_pointer_enter: impl Fn(&mut PointerEnterEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_pointer_enter(Rc::new(on_pointer_enter)));
        self
    }

    /// Adds a pointer leave listener.
    fn on_pointer_leave(self, on_pointer_leave: impl Fn(&mut PointerLeaveEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_pointer_leave(Rc::new(on_pointer_leave)));
        self
    }

    /// Adds a radio value changed listener.
    fn on_radio_value_changed(self, on_radio_value_changed: impl Fn(&mut RadioValueChangedEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_radio_value_changed(Rc::new(on_radio_value_changed)));
        self
    }

    /// Adds a checkbox toggled listener.
    fn on_checkbox_toggled(self, on_checkbox_toggled: impl Fn(&mut CheckboxToggledEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_checkbox_toggled(Rc::new(on_checkbox_toggled)));
        self
    }

    /// Adds a text intput changed listener.
    fn on_text_input_changed(self, on_text_input_changed: impl Fn(&mut TextInputChangedEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_text_input_changed(Rc::new(on_text_input_changed)));
        self
    }

    /// Sets the element's user based id.
    fn id(self, id: &str) -> Self {
        self.with_mut(|element| element.set_id(id));
        self
    }

    /// Sets the accessibility name.
    fn accessibility_name(self, name: &str) -> Self {
        self.with_mut(|element| element.element_data_mut().set_accessibility_name(name));
        self
    }

    /// Returns the element's user based id. This id is not used by RetGUI.
    fn get_id(&self) -> Option<SmolStr> {
        self.with(|element| element.get_id())
    }

    /// Adds a pointer button down listener.
    fn on_pointer_button_down(self, on_pointer_button_down: impl Fn(&mut PointerButtonEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_pointer_button_down(Rc::new(on_pointer_button_down)));
        self
    }

    /// Adds a pointer button moved listener.
    fn on_pointer_moved(self, on_pointer_moved: impl Fn(&mut PointerMovedEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_pointer_moved(Rc::new(on_pointer_moved)));
        self
    }

    /// Adds a pointer button up listener.
    fn on_pointer_button_up(self, on_pointer_button_up: impl Fn(&mut PointerButtonEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_pointer_button_up(Rc::new(on_pointer_button_up)));
        self
    }

    /// Adds a click listener.
    fn on_click(self, on_click: impl Fn(&mut ClickEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_click(Rc::new(on_click)));
        self
    }

    /// Adds a custom event listener.
    fn on_custom_event(self, on_custom_event: impl Fn(&mut CustomEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_custom_event(Rc::new(on_custom_event)));
        self
    }

    /// Emits a custom event using the element as the target element.
    fn emit_custom_event<T: Any + 'static>(&self, detail: T) {
        queue_event(EventKind::Custom(CustomEvent::new(self.as_dyn_element(), detail)));
    }

    /// Adds a focus event listener.
    fn on_focus(self, on_focus: impl Fn(&mut FocusEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_focus(Rc::new(on_focus)));
        self
    }

    /// Adds an unfocus event listener.
    fn on_unfocus(self, on_unfocus: impl Fn(&mut UnfocusEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_unfocus(Rc::new(on_unfocus)));
        self
    }

    /// Adds a lost pointer capture event listener.
    fn on_lost_pointer_capture(self, on_lost_pointer_capture: impl Fn(&mut PointerCaptureEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_lost_pointer_capture(Rc::new(on_lost_pointer_capture)));
        self
    }

    /// Adds a got pointer capture event listener.
    fn on_got_pointer_capture(self, on_got_pointer_capture: impl Fn(&mut PointerCaptureEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_got_pointer_capture(Rc::new(on_got_pointer_capture)));
        self
    }

    /// Adds a keyboard input event listener.
    fn on_keyboard_input(self, on_keyboard_input: impl Fn(&mut KeyboardEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_keyboard_input(Rc::new(on_keyboard_input)));
        self
    }

    /// Adds a slider value changed event listener.
    fn on_slider_value_changed(self, on_slider_value_changed: impl Fn(&mut SliderValueChangedEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_slider_value_changed(Rc::new(on_slider_value_changed)));
        self
    }

    /// Adds a scroll event listener.
    fn on_scroll(self, on_scroll: impl Fn(&mut ScrollEvent) + 'static) -> Self {
        self.with_mut(|element| element.on_scroll(Rc::new(on_scroll)));
        self
    }

    /// Scrolls to a child based on the child's user id.
    fn scroll_to_child_by_id(self, id: &str) -> Self {
        self.with_mut(|element| element.scroll_to_child_by_id_with_options(id, ScrollOptions::default()));
        self
    }

    /// Scrolls to a child based on the child's user id according to the scroll options.
    fn scroll_to_child_by_id_with_options(self, id: &str, options: ScrollOptions) -> Self {
        self.with_mut(|element| element.scroll_to_child_by_id_with_options(id, options));
        self
    }

    /// Scrolls to a specific y value in logical pixels.
    fn scroll_to(self, y: f32) -> Self {
        self.with_mut(|element| element.scroll_to(y));
        self
    }

    /// Scrolls to the top of the element.
    fn scroll_to_top(self) -> Self {
        self.with_mut(|element| element.scroll_to_top());
        self
    }

    /// Scrolls to the button of the element.
    fn scroll_to_bottom(self) -> Self {
        self.with_mut(|element| element.scroll_to_bottom());
        self
    }

    /// Scrolls by a logical amount of pixels.
    fn scroll_by(self, y: f32) -> Self {
        self.with_mut(|element| element.scroll_by(y));
        self
    }

    /// Returns the elements current scroll state.
    fn get_scroll_state(&self) -> ScrollState {
        self.with_mut(|element| element.get_scroll_state())
    }

    /// Sets the layout algorith e.g. block, flex, etc.
    fn display(self, display: Display) -> Self {
        self.with_mut(|element| element.set_display(display));
        self
    }

    /// Sets the box sizing e.g. content box/border box.
    fn box_sizing(self, box_sizing: BoxSizing) -> Self {
        self.with_mut(|element| element.set_box_sizing(box_sizing));
        self
    }

    /// Sets the position of the element.
    ///
    /// Unlike HTML, this has no effect on the visual order of the element.
    fn position(self, position: Position) -> Self {
        self.with_mut(|element| element.set_position(position));
        self
    }

    /// Puts the element on top of other elements.
    fn overlay(self, overlay: bool) -> Self {
        self.with_mut(|element| element.set_overlay(overlay));
        self
    }

    /// Returns if the element is put on top of other elements.
    fn get_overlay(&self) -> bool {
        self.with(|element| element.style().get_overlay())
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn margin(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.with_mut(|element| element.set_margin(top, right, bottom, left));
        self
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn margin_all(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_margin_all(value));
        self
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn margin_vertical(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_margin_vertical(value));
        self
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn margin_horizontal(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_margin_horizontal(value));
        self
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn padding(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.with_mut(|element| element.set_padding(top, right, bottom, left));
        self
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn padding_all(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_padding_all(value));
        self
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn padding_vertical(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_padding_vertical(value));
        self
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn padding_horizontal(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_padding_horizontal(value));
        self
    }

    /// Sets the gap between children for flex/grid containers.
    fn gap(self, row_gap: Unit, column_gap: Unit) -> Self {
        self.with_mut(|element| element.set_gap(row_gap, column_gap));
        self
    }

    /// Sets the row gap between children for flex/grid containers.
    fn row_gap(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_row_gap(value));
        self
    }

    /// Sets the column gap between children for flex/grid containers.
    fn column_gap(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_column_gap(value));
        self
    }

    /// Align the element relative to its sides. Only applies to positioned elements.
    fn inset(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.with_mut(|element| element.set_inset(top, right, bottom, left));
        self
    }

    /// Sets the minium width of the element.
    fn min_width(self, min_width: Unit) -> Self {
        self.with_mut(|element| element.set_min_width(min_width));
        self
    }

    /// Sets the minium height of the element.
    fn min_height(self, min_height: Unit) -> Self {
        self.with_mut(|element| element.set_min_height(min_height));
        self
    }

    /// Sets the width of the element.
    fn width(self, width: Unit) -> Self {
        self.with_mut(|element| element.set_width(width));
        self
    }

    /// Sets the height of the element.
    fn height(self, height: Unit) -> Self {
        self.with_mut(|element| element.set_height(height));
        self
    }

    /// Sets the max width of the element.
    fn max_width(self, max_width: Unit) -> Self {
        self.with_mut(|element| element.set_max_width(max_width));
        self
    }

    /// Sets the max height of the element.
    fn max_height(self, max_height: Unit) -> Self {
        self.with_mut(|element| element.set_max_height(max_height));
        self
    }

    /// Sets the wrapping behavior for flex elements.
    fn wrap(self, wrap: FlexWrap) -> Self {
        self.with_mut(|element| element.set_wrap(wrap));
        self
    }

    /// Determines how flex/grid children are laid out on the cross axis.
    fn align_items(self, align_items: AlignItems) -> Self {
        self.with_mut(|element| element.set_align_items(align_items));
        self
    }

    /// Overrides a parent's align_items.
    fn align_self(self, align_self: AlignSelf) -> Self {
        self.with_mut(|element| element.set_align_self(align_self));
        self
    }

    fn align_content(self, align_content: AlignContent) -> Self {
        self.with_mut(|element| element.set_align_content(align_content));
        self
    }

    fn justify_content(self, justify_content: JustifyContent) -> Self {
        self.with_mut(|element| element.set_justify_content(justify_content));
        self
    }

    fn flex_direction(self, flex_direction: FlexDirection) -> Self {
        self.with_mut(|element| element.set_flex_direction(flex_direction));
        self
    }

    fn flex_grow(self, flex_grow: f32) -> Self {
        self.with_mut(|element| element.set_flex_grow(flex_grow));
        self
    }

    fn flex_shrink(self, flex_shrink: f32) -> Self {
        self.with_mut(|element| element.set_flex_shrink(flex_shrink));
        self
    }

    fn flex_basis(self, flex_basis: Unit) -> Self {
        self.with_mut(|element| element.set_flex_basis(flex_basis));
        self
    }

    fn order(self, order: i32) -> Self {
        self.with_mut(|element| element.set_order(order));
        self
    }

    fn font_family(self, font_family: FontFamily) -> Self {
        self.with_mut(|element| element.set_font_family(font_family));
        self
    }

    fn color(self, color: Color) -> Self {
        self.with_mut(|element| element.set_text_brush(Brush::Color(color)));
        self
    }

    fn text_gradient(self, gradient: Gradient) -> Self {
        self.with_mut(|element| element.set_text_brush(Brush::Gradient(gradient)));
        self
    }

    fn background_color(self, background_color: Color) -> Self {
        self.with_mut(|element| element.set_background_brush(Brush::Color(background_color)));
        self
    }

    fn background_gradient(self, gradient: Gradient) -> Self {
        self.with_mut(|element| element.set_background_brush(Brush::Gradient(gradient)));
        self
    }

    fn font_size(self, font_size: f32) -> Self {
        self.with_mut(|element| element.set_font_size(font_size));
        self
    }

    fn line_height(self, line_height: f32) -> Self {
        self.with_mut(|element| element.set_line_height(line_height));
        self
    }

    fn font_weight(self, font_weight: FontWeight) -> Self {
        self.with_mut(|element| element.set_font_weight(font_weight));
        self
    }

    fn font_style(self, font_style: FontStyle) -> Self {
        self.with_mut(|element| element.set_font_style(font_style));
        self
    }

    fn text_align(self, text_align: TextAlign) -> Self {
        self.with_mut(|element| element.set_text_align(text_align));
        self
    }

    fn underline(self, thickness: Option<f32>, color: Color, offset: Option<f32>) -> Self {
        self.with_mut(|element| {
            element.set_underline(Some(Underline {
                thickness,
                brush: Brush::Color(color),
                offset,
            }))
        });

        self
    }

    fn underline_gradient(self, thickness: Option<f32>, gradient: Gradient, offset: Option<f32>) -> Self {
        self.with_mut(|element| {
            element.set_underline(Some(Underline {
                thickness,
                brush: Brush::Gradient(gradient),
                offset,
            }))
        });

        self
    }

    fn overflow(self, overflow_x: Overflow, overflow_y: Overflow) -> Self {
        self.with_mut(|element| element.set_overflow(overflow_x, overflow_y));
        self
    }

    fn overflow_x(self, overflow_x: Overflow) -> Self {
        self.with_mut(|element| element.set_overflow_x(overflow_x));
        self
    }

    fn overflow_y(self, overflow_y: Overflow) -> Self {
        self.with_mut(|element| element.set_overflow_y(overflow_y));
        self
    }

    fn border_color(self, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.with_mut(|element| element.set_border_color(top, right, bottom, left));
        self
    }

    fn border_color_all(self, value: Color) -> Self {
        self.with_mut(|element| element.set_border_color_all(value));
        self
    }

    fn border_color_vertical(self, value: Color) -> Self {
        self.with_mut(|element| element.set_border_color_vertical(value));
        self
    }

    fn border_color_horizontal(self, value: Color) -> Self {
        self.with_mut(|element| element.set_border_color_horizontal(value));
        self
    }

    fn border_width(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.with_mut(|element| element.set_border_width(top, right, bottom, left));
        self
    }

    fn border_width_all(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_border_width_all(value));
        self
    }

    fn border_width_vertical(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_border_width_vertical(value));
        self
    }

    fn border_width_horizontal(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_border_width_horizontal(value));
        self
    }

    fn outline_color(self, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.with_mut(|element| element.set_outline_color(top, right, bottom, left));
        self
    }

    fn outline_color_all(self, value: Color) -> Self {
        self.with_mut(|element| element.set_outline_color_all(value));
        self
    }

    fn outline_color_vertical(self, value: Color) -> Self {
        self.with_mut(|element| element.set_outline_color_vertical(value));
        self
    }

    fn outline_color_horizontal(self, value: Color) -> Self {
        self.with_mut(|element| element.set_outline_color_horizontal(value));
        self
    }

    fn outline_width(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.with_mut(|element| element.set_outline_width(top, right, bottom, left));
        self
    }

    fn outline_width_all(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_outline_width_all(value));
        self
    }

    fn outline_width_vertical(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_outline_width_vertical(value));
        self
    }

    fn outline_width_horizontal(self, value: Unit) -> Self {
        self.with_mut(|element| element.set_outline_width_horizontal(value));
        self
    }

    fn border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.with_mut(|element| element.set_border_radius(top, right, bottom, left));
        self
    }

    fn border_radius_all(self, value: (f32, f32)) -> Self {
        self.with_mut(|element| element.set_border_radius_all(value));
        self
    }

    fn border_radius_vertical(self, value: (f32, f32)) -> Self {
        self.with_mut(|element| element.set_border_radius_vertical(value));
        self
    }

    fn border_radius_horizontal(self, value: (f32, f32)) -> Self {
        self.with_mut(|element| element.set_border_radius_horizontal(value));
        self
    }

    fn scrollbar_color(self, scrollbar_color: ScrollbarColor) -> Self {
        self.with_mut(|element| element.set_scrollbar_color(scrollbar_color));
        self
    }

    fn scrollbar_thumb_margin(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.with_mut(|element| element.set_scrollbar_thumb_margin(top, right, bottom, left));
        self
    }

    fn scrollbar_thumb_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.with_mut(|element| element.set_scrollbar_thumb_radius(top, right, bottom, left));
        self
    }

    fn scrollbar_width(self, scrollbar_width: f32) -> Self {
        self.with_mut(|element| element.set_scrollbar_width(scrollbar_width));
        self
    }

    /// Sets the list of animations.
    fn animations(self, animations: Vec<Animation>) -> Self {
        self.with_mut(|element| element.set_animations(animations));
        self
    }

    /// Sets the box shadows on this element.
    fn box_shadows(self, box_shadows: Vec<BoxShadow>) -> Self {
        self.with_mut(|element| element.set_box_shadows(box_shadows));
        self
    }

    /// Focus the element.
    fn focus(self) -> Self {
        self.with_mut(|element| element.focus());
        self
    }

    /// Returns whether the element current has focus.
    fn is_focused(&self) -> bool {
        self.with(|element| element.is_focused())
    }

    /// Unfocuses the element.
    fn unfocus(self) -> Self {
        self.with_mut(|element| element.unfocus());
        self
    }

    /// Get the elements box in logical pixels.
    fn get_computed_box_transformed(&self) -> ElementBox {
        self.with(|element| element.get_computed_box_transformed())
    }

    /// Returns whether the element has pointer capture.
    fn has_pointer_capture(&self, pointer_id: PointerId) -> bool {
        self.with(|element| element.has_pointer_capture(pointer_id))
    }

    /// Converts the element to a `DynElement`. This is useful when the type of element being stored does not matter.
    fn as_dyn_element(&self) -> DynElement {
        self.with(|element| element.to_dyn_element())
    }
}
