use std::any::Any;

use retgui_primitives::Color;
use retgui_primitives::brush::Brush;
use retgui_primitives::gradient::Gradient;
use retgui_resource_manager::ResourceId;

#[cfg(feature = "audio")]
use crate::elements::Audio;
use crate::elements::scrollable::ScrollOptions;
use crate::elements::{Calendar, Dropdown, Element, Elements, Image, Radio, Slider, SliderDirection, Text, TextInput, TinyVg};
use crate::events::{CheckboxToggledEvent, ClickEvent, CustomEvent, EventCallbackKind, EventListenerOptions, FocusEvent, KeyboardEvent, PointerButtonEvent, PointerCaptureEvent, PointerEnterEvent, PointerLeaveEvent, PointerMovedEvent, RadioValueChangedEvent, ScrollEvent, SliderValueChangedEvent, TextInputChangedEvent, UnfocusEvent};
use crate::style::{AlignContent, AlignItems, AlignSelf, Animation, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, TextAlign, Unit};
use crate::text::RangedStyles;

/// Binds elements to a specific element for edits.
#[must_use = "call finish() to recover the element handle"]
pub struct ElementEditor<'a, E: Element> {
    element: E,
    elements: &'a mut Elements,
}

impl<'a, E: Element> ElementEditor<'a, E> {
    pub(crate) fn new(element: E, elements: &'a mut Elements) -> Self {
        Self { element, elements }
    }

    /// Returns the edited, copyable element handle and releases the store.
    pub fn finish(self) -> E {
        self.element
    }

    /// Returns a copy of the element handle without ending the edit.
    pub fn handle(&self) -> E {
        self.element
    }

    /// Runs a custom element operation while the store is bound.
    ///
    /// This is the extension point for widget-specific methods that are not
    /// directly exposed by `ElementEditor`. The operation is not called when
    /// the element has been deleted.
    pub fn apply(self, operation: impl FnOnce(E, &mut Elements)) -> Self {
        if self.elements.contains(self.element.as_dyn_element()) {
            operation(self.element, self.elements);
        }
        self
    }

    /// Adds an already-created child.
    pub fn push(self, child: impl Element) -> Self {
        self.apply(|element, elements| {
            element.push(elements, child);
        })
    }

    /// Creates and adds a child without requiring another store variable.
    pub fn push_with<C: Element>(self, build: impl FnOnce(&mut Elements) -> C) -> Self {
        if self.elements.contains(self.element.as_dyn_element()) {
            let child = build(self.elements);
            self.element.push(self.elements, child);
        }
        self
    }

    /// Removes every child from this element.
    pub fn remove_all_children(self) -> Self {
        self.apply(|element, elements| element.remove_all_children(elements))
    }

    /// Deletes every child subtree and invalidates their handles.
    pub fn delete_all_children(self) -> Self {
        self.apply(|element, elements| element.delete_all_children(elements))
    }

    pub fn id(self, id: &str) -> Self {
        self.apply(|element, elements| {
            element.set_id(elements, id);
        })
    }

    pub fn accessibility_name(self, name: &str) -> Self {
        self.apply(|element, elements| {
            element.set_accessibility_name(elements, name);
        })
    }

    pub fn add_event_listener(self, callback: EventCallbackKind, options: EventListenerOptions) -> Self {
        self.apply(|element, elements| {
            element.add_event_listener(elements, callback, options);
        })
    }

    pub fn on_pointer_enter(self, callback: impl Fn(&mut PointerEnterEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_pointer_enter(elements, callback);
        })
    }

    pub fn on_pointer_leave(self, callback: impl Fn(&mut PointerLeaveEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_pointer_leave(elements, callback);
        })
    }

    pub fn on_radio_value_changed(
        self,
        callback: impl Fn(&mut RadioValueChangedEvent, &mut Elements) + 'static,
    ) -> Self {
        self.apply(|element, elements| {
            element.on_radio_value_changed(elements, callback);
        })
    }

    pub fn on_checkbox_toggled(self, callback: impl Fn(&mut CheckboxToggledEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_checkbox_toggled(elements, callback);
        })
    }

    pub fn on_text_input_changed(self, callback: impl Fn(&mut TextInputChangedEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_text_input_changed(elements, callback);
        })
    }

    pub fn on_pointer_button_down(self, callback: impl Fn(&mut PointerButtonEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_pointer_button_down(elements, callback);
        })
    }

    pub fn on_pointer_moved(self, callback: impl Fn(&mut PointerMovedEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_pointer_moved(elements, callback);
        })
    }

    pub fn on_pointer_button_up(self, callback: impl Fn(&mut PointerButtonEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_pointer_button_up(elements, callback);
        })
    }

    pub fn on_click(self, callback: impl Fn(&mut ClickEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_click(elements, callback);
        })
    }

    pub fn on_custom_event(self, callback: impl Fn(&mut CustomEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_custom_event(elements, callback);
        })
    }

    pub fn emit_custom_event<T: Any + 'static>(self, detail: T) -> Self {
        self.apply(|element, elements| element.emit_custom_event(elements, detail))
    }

    pub fn on_focus(self, callback: impl Fn(&mut FocusEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_focus(elements, callback);
        })
    }

    pub fn on_unfocus(self, callback: impl Fn(&mut UnfocusEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_unfocus(elements, callback);
        })
    }

    pub fn on_lost_pointer_capture(self, callback: impl Fn(&mut PointerCaptureEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_lost_pointer_capture(elements, callback);
        })
    }

    pub fn on_got_pointer_capture(self, callback: impl Fn(&mut PointerCaptureEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_got_pointer_capture(elements, callback);
        })
    }

    pub fn on_keyboard_input(self, callback: impl Fn(&mut KeyboardEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_keyboard_input(elements, callback);
        })
    }

    pub fn on_slider_value_changed(
        self,
        callback: impl Fn(&mut SliderValueChangedEvent, &mut Elements) + 'static,
    ) -> Self {
        self.apply(|element, elements| {
            element.on_slider_value_changed(elements, callback);
        })
    }

    pub fn on_scroll(self, callback: impl Fn(&mut ScrollEvent, &mut Elements) + 'static) -> Self {
        self.apply(|element, elements| {
            element.on_scroll(elements, callback);
        })
    }

    pub fn scroll_to_child_by_id(self, id: &str) -> Self {
        self.apply(|element, elements| {
            element.scroll_to_child_by_id(elements, id);
        })
    }

    pub fn scroll_to_child_by_id_with_options(self, id: &str, options: ScrollOptions) -> Self {
        self.apply(|element, elements| {
            element.scroll_to_child_by_id_with_options(elements, id, options);
        })
    }

    pub fn scroll_to(self, y: f32) -> Self {
        self.apply(|element, elements| {
            element.scroll_to(elements, y);
        })
    }

    pub fn scroll_to_top(self) -> Self {
        self.apply(|element, elements| {
            element.scroll_to_top(elements);
        })
    }

    pub fn scroll_to_bottom(self) -> Self {
        self.apply(|element, elements| {
            element.scroll_to_bottom(elements);
        })
    }

    pub fn scroll_by(self, y: f32) -> Self {
        self.apply(|element, elements| {
            element.scroll_by(elements, y);
        })
    }

    pub fn display(self, value: Display) -> Self {
        self.apply(|element, elements| {
            element.set_display(elements, value);
        })
    }

    pub fn box_sizing(self, value: BoxSizing) -> Self {
        self.apply(|element, elements| {
            element.set_box_sizing(elements, value);
        })
    }

    pub fn position(self, value: Position) -> Self {
        self.apply(|element, elements| {
            element.set_position(elements, value);
        })
    }

    pub fn overlay(self, value: bool) -> Self {
        self.apply(|element, elements| {
            element.set_overlay(elements, value);
        })
    }

    pub fn margin(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_margin(elements, top, right, bottom, left);
        })
    }

    pub fn margin_all(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_margin_all(elements, value);
        })
    }

    pub fn margin_vertical(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_margin_vertical(elements, value);
        })
    }

    pub fn margin_horizontal(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_margin_horizontal(elements, value);
        })
    }

    pub fn padding(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_padding(elements, top, right, bottom, left);
        })
    }

    pub fn padding_all(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_padding_all(elements, value);
        })
    }

    pub fn padding_vertical(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_padding_vertical(elements, value);
        })
    }

    pub fn padding_horizontal(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_padding_horizontal(elements, value);
        })
    }

    pub fn gap(self, row_gap: Unit, column_gap: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_gap(elements, row_gap, column_gap);
        })
    }

    pub fn row_gap(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_row_gap(elements, value);
        })
    }

    pub fn column_gap(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_column_gap(elements, value);
        })
    }

    pub fn inset(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_inset(elements, top, right, bottom, left);
        })
    }

    pub fn min_width(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_min_width(elements, value);
        })
    }

    pub fn min_height(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_min_height(elements, value);
        })
    }

    pub fn width(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_width(elements, value);
        })
    }

    pub fn height(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_height(elements, value);
        })
    }

    pub fn max_width(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_max_width(elements, value);
        })
    }

    pub fn max_height(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_max_height(elements, value);
        })
    }

    pub fn wrap(self, value: FlexWrap) -> Self {
        self.apply(|element, elements| {
            element.set_wrap(elements, value);
        })
    }

    pub fn align_items(self, value: AlignItems) -> Self {
        self.apply(|element, elements| {
            element.set_align_items(elements, value);
        })
    }

    pub fn align_self(self, value: AlignSelf) -> Self {
        self.apply(|element, elements| {
            element.set_align_self(elements, value);
        })
    }

    pub fn align_content(self, value: AlignContent) -> Self {
        self.apply(|element, elements| {
            element.set_align_content(elements, value);
        })
    }

    pub fn justify_content(self, value: JustifyContent) -> Self {
        self.apply(|element, elements| {
            element.set_justify_content(elements, value);
        })
    }

    pub fn flex_direction(self, value: FlexDirection) -> Self {
        self.apply(|element, elements| {
            element.set_flex_direction(elements, value);
        })
    }

    pub fn flex_grow(self, value: f32) -> Self {
        self.apply(|element, elements| {
            element.set_flex_grow(elements, value);
        })
    }

    pub fn flex_shrink(self, value: f32) -> Self {
        self.apply(|element, elements| {
            element.set_flex_shrink(elements, value);
        })
    }

    pub fn flex_basis(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_flex_basis(elements, value);
        })
    }

    pub fn order(self, value: i32) -> Self {
        self.apply(|element, elements| {
            element.set_order(elements, value);
        })
    }

    pub fn font_family(self, value: FontFamily) -> Self {
        self.apply(|element, elements| {
            element.set_font_family(elements, value);
        })
    }

    pub fn color(self, value: Color) -> Self {
        self.apply(|element, elements| {
            element.set_color(elements, value);
        })
    }

    pub fn text_gradient(self, value: Gradient) -> Self {
        self.apply(|element, elements| {
            element.set_text_gradient(elements, value);
        })
    }

    pub fn background_color(self, value: Color) -> Self {
        self.apply(|element, elements| {
            element.set_background_color(elements, value);
        })
    }

    pub fn background_gradient(self, value: Gradient) -> Self {
        self.apply(|element, elements| {
            element.set_background_gradient(elements, value);
        })
    }

    pub fn font_size(self, value: f32) -> Self {
        self.apply(|element, elements| {
            element.set_font_size(elements, value);
        })
    }

    pub fn line_height(self, value: f32) -> Self {
        self.apply(|element, elements| {
            element.set_line_height(elements, value);
        })
    }

    pub fn font_weight(self, value: FontWeight) -> Self {
        self.apply(|element, elements| {
            element.set_font_weight(elements, value);
        })
    }

    pub fn font_style(self, value: FontStyle) -> Self {
        self.apply(|element, elements| {
            element.set_font_style(elements, value);
        })
    }

    pub fn text_align(self, value: TextAlign) -> Self {
        self.apply(|element, elements| {
            element.set_text_align(elements, value);
        })
    }

    pub fn underline(self, thickness: Option<f32>, color: Color, offset: Option<f32>) -> Self {
        self.apply(|element, elements| {
            element.set_underline(elements, thickness, color, offset);
        })
    }

    pub fn underline_gradient(self, thickness: Option<f32>, gradient: Gradient, offset: Option<f32>) -> Self {
        self.apply(|element, elements| {
            element.set_underline_gradient(elements, thickness, gradient, offset);
        })
    }

    pub fn overflow(self, overflow_x: Overflow, overflow_y: Overflow) -> Self {
        self.apply(|element, elements| {
            element.set_overflow(elements, overflow_x, overflow_y);
        })
    }

    pub fn overflow_x(self, value: Overflow) -> Self {
        self.apply(|element, elements| {
            element.set_overflow_x(elements, value);
        })
    }

    pub fn overflow_y(self, value: Overflow) -> Self {
        self.apply(|element, elements| {
            element.set_overflow_y(elements, value);
        })
    }

    pub fn border_color(self, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.apply(|element, elements| {
            element.set_border_color(elements, top, right, bottom, left);
        })
    }

    pub fn border_color_all(self, value: Color) -> Self {
        self.apply(|element, elements| {
            element.set_border_color_all(elements, value);
        })
    }

    pub fn border_color_vertical(self, value: Color) -> Self {
        self.apply(|element, elements| {
            element.set_border_color_vertical(elements, value);
        })
    }

    pub fn border_color_horizontal(self, value: Color) -> Self {
        self.apply(|element, elements| {
            element.set_border_color_horizontal(elements, value);
        })
    }

    pub fn border_width(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_border_width(elements, top, right, bottom, left);
        })
    }

    pub fn border_width_all(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_border_width_all(elements, value);
        })
    }

    pub fn border_width_vertical(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_border_width_vertical(elements, value);
        })
    }

    pub fn border_width_horizontal(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_border_width_horizontal(elements, value);
        })
    }

    pub fn outline_color(self, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.apply(|element, elements| {
            element.set_outline_color(elements, top, right, bottom, left);
        })
    }

    pub fn outline_color_all(self, value: Color) -> Self {
        self.apply(|element, elements| {
            element.set_outline_color_all(elements, value);
        })
    }

    pub fn outline_color_vertical(self, value: Color) -> Self {
        self.apply(|element, elements| {
            element.set_outline_color_vertical(elements, value);
        })
    }

    pub fn outline_color_horizontal(self, value: Color) -> Self {
        self.apply(|element, elements| {
            element.set_outline_color_horizontal(elements, value);
        })
    }

    pub fn outline_width(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_outline_width(elements, top, right, bottom, left);
        })
    }

    pub fn outline_width_all(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_outline_width_all(elements, value);
        })
    }

    pub fn outline_width_vertical(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_outline_width_vertical(elements, value);
        })
    }

    pub fn outline_width_horizontal(self, value: Unit) -> Self {
        self.apply(|element, elements| {
            element.set_outline_width_horizontal(elements, value);
        })
    }

    pub fn border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.apply(|element, elements| {
            element.set_border_radius(elements, top, right, bottom, left);
        })
    }

    pub fn border_radius_all(self, value: (f32, f32)) -> Self {
        self.apply(|element, elements| {
            element.set_border_radius_all(elements, value);
        })
    }

    pub fn border_radius_vertical(self, value: (f32, f32)) -> Self {
        self.apply(|element, elements| {
            element.set_border_radius_vertical(elements, value);
        })
    }

    pub fn border_radius_horizontal(self, value: (f32, f32)) -> Self {
        self.apply(|element, elements| {
            element.set_border_radius_horizontal(elements, value);
        })
    }

    pub fn scrollbar_color(self, value: ScrollbarColor) -> Self {
        self.apply(|element, elements| {
            element.set_scrollbar_color(elements, value);
        })
    }

    pub fn scrollbar_thumb_margin(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.apply(|element, elements| {
            element.set_scrollbar_thumb_margin(elements, top, right, bottom, left);
        })
    }

    pub fn scrollbar_thumb_radius(
        self,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> Self {
        self.apply(|element, elements| {
            element.set_scrollbar_thumb_radius(elements, top, right, bottom, left);
        })
    }

    pub fn scrollbar_width(self, value: f32) -> Self {
        self.apply(|element, elements| {
            element.set_scrollbar_width(elements, value);
        })
    }

    pub fn animations(self, value: Vec<Animation>) -> Self {
        self.apply(|element, elements| {
            element.set_animations(elements, value);
        })
    }

    pub fn box_shadows(self, value: Vec<BoxShadow>) -> Self {
        self.apply(|element, elements| {
            element.set_box_shadows(elements, value);
        })
    }

    pub fn focus(self) -> Self {
        self.apply(|element, elements| {
            element.focus(elements);
        })
    }

    pub fn unfocus(self) -> Self {
        self.apply(|element, elements| {
            element.unfocus(elements);
        })
    }
}

impl ElementEditor<'_, Text> {
    pub fn selectable(self, value: bool) -> Self {
        self.apply(|element, elements| {
            element.set_selectable(elements, value);
        })
    }

    pub fn text(self, value: &str) -> Self {
        self.apply(|element, elements| {
            element.set_text(elements, value);
        })
    }

    pub fn text_smol_str(self, value: smol_str::SmolStr) -> Self {
        self.apply(|element, elements| {
            element.set_text_smol_str(elements, value);
        })
    }
}

impl ElementEditor<'_, TextInput> {
    pub fn disabled(self, value: bool) -> Self {
        self.apply(|element, elements| {
            element.set_disabled(elements, value);
        })
    }

    pub fn multiline(self, value: bool) -> Self {
        self.apply(|element, elements| {
            element.set_multiline(elements, value);
        })
    }

    pub fn text(self, value: &str) -> Self {
        self.apply(|element, elements| {
            element.set_text(elements, value);
        })
    }

    pub fn ranged_styles(self, value: RangedStyles) -> Self {
        self.apply(|element, elements| {
            element.set_ranged_styles(elements, value);
        })
    }
}

impl ElementEditor<'_, Slider> {
    pub fn value(self, value: f64) -> Self {
        self.apply(|element, elements| {
            element.set_value(elements, value);
        })
    }

    pub fn step(self, value: f64) -> Self {
        self.apply(|element, elements| {
            element.set_step(elements, value);
        })
    }

    pub fn min(self, value: f64) -> Self {
        self.apply(|element, elements| {
            element.set_min(elements, value);
        })
    }

    pub fn max(self, value: f64) -> Self {
        self.apply(|element, elements| {
            element.set_max(elements, value);
        })
    }

    pub fn direction(self, value: SliderDirection) -> Self {
        self.apply(|element, elements| {
            element.set_direction(elements, value);
        })
    }

    pub fn thumb_size(self, value: f64) -> Self {
        self.apply(|element, elements| {
            element.set_thumb_size(elements, value);
        })
    }

    pub fn thumb_color(self, value: Brush) -> Self {
        self.apply(|element, elements| {
            element.set_thumb_color(elements, value);
        })
    }

    pub fn thumb_border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.apply(|element, elements| {
            element.set_thumb_border_radius(elements, top, right, bottom, left);
        })
    }

    pub fn track_color(self, value: Color) -> Self {
        self.apply(|element, elements| {
            element.set_track_color(elements, value);
        })
    }

    pub fn track_gradient(self, value: Gradient) -> Self {
        self.apply(|element, elements| {
            element.set_track_gradient(elements, value);
        })
    }

    pub fn track_border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.apply(|element, elements| {
            element.set_track_border_radius(elements, top, right, bottom, left);
        })
    }
}

impl ElementEditor<'_, Image> {
    pub fn resource_id(self, value: ResourceId) -> Self {
        self.apply(|element, elements| {
            element.set_resource_id(elements, value);
        })
    }
}

impl ElementEditor<'_, TinyVg> {
    pub fn resource_id(self, value: ResourceId) -> Self {
        self.apply(|element, elements| {
            element.set_resource_id(elements, value);
        })
    }
}

impl ElementEditor<'_, Dropdown> {
    pub fn selected_item(self, index: usize) -> Self {
        self.apply(|element, elements| {
            element.set_selected_item(elements, index);
        })
    }
}

impl ElementEditor<'_, Calendar> {
    pub fn start_year(self, year: i32) -> Self {
        self.apply(|element, elements| {
            element.set_start_year(elements, year);
        })
    }

    pub fn end_year(self, year: i32) -> Self {
        self.apply(|element, elements| {
            element.set_end_year(elements, year);
        })
    }
}

impl ElementEditor<'_, Radio> {
    pub fn hide_radio(self) -> Self {
        self.apply(|element, elements| {
            element.hide_radio(elements);
        })
    }
}

#[cfg(feature = "audio")]
impl ElementEditor<'_, Audio> {
    pub fn controls(self, value: bool) -> Self {
        self.apply(|element, elements| {
            element.set_controls(elements, value);
        })
    }

    pub fn play(self) -> Self {
        self.apply(|element, elements| {
            element.play(elements);
        })
    }

    pub fn pause(self) -> Self {
        self.apply(|element, elements| {
            element.pause(elements);
        })
    }

    pub fn toggle(self) -> Self {
        self.apply(|element, elements| {
            element.toggle(elements);
        })
    }
}
