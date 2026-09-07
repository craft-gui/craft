use std::any::Any;

use retgui_primitives::Color;
use retgui_primitives::brush::Brush;
use retgui_primitives::gradient::Gradient;

use retgui_resource_manager::ResourceId;

use smol_str::SmolStr;

use crate::App;
#[cfg(feature = "audio")]
use crate::elements::Audio;
use crate::elements::editor::capabilities::{EditorIdentity, ResourceContent, TextContent};
use crate::elements::scrollable::ScrollOptions;
use crate::elements::{Calendar, Dropdown, Element, Radio, Slider, SliderDirection, Text, TextInput};
use crate::events::{CheckboxToggledEvent, ClickEvent, CustomEvent, EventCallbackKind, EventListenerOptions, FocusEvent, KeyboardEvent, PointerButtonEvent, PointerCaptureEvent, PointerEnterEvent, PointerLeaveEvent, PointerMovedEvent, RadioValueChangedEvent, ScrollEvent, SliderValueChangedEvent, TextInputChangedEvent, UnfocusEvent};
use crate::style::{AlignContent, AlignItems, AlignSelf, Animation, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, TextAlign, Unit};
use crate::text::RangedStyles;

/// Binds elements to a specific element for edits.
#[must_use = "call finish() to recover the element handle"]
pub struct ElementEditor<'a, E: Element> {
    element: E,
    app: &'a mut App,
}

impl<'a, E: Element> ElementEditor<'a, E> {
    pub(crate) fn new(element: E, app: &'a mut App) -> Self {
        Self { element, app }
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
    pub fn apply(self, operation: impl FnOnce(E, &mut App)) -> Self {
        if self.app.contains(self.element.as_dyn_element()) {
            operation(self.element, self.app);
        }
        self
    }

    /// Adds an already-created child.
    pub fn push(self, child: impl Element) -> Self {
        self.apply(|element, app| {
            element.push(app, child);
        })
    }

    /// Creates and adds a child without requiring another store variable.
    pub fn push_with<C: Element>(self, build: impl FnOnce(&mut App) -> C) -> Self {
        if self.app.contains(self.element.as_dyn_element()) {
            let child = build(self.app);
            self.element.push(self.app, child);
        }
        self
    }

    /// Removes every child from this element.
    pub fn remove_all_children(self) -> Self {
        self.apply(|element, app| element.remove_all_children(app))
    }

    /// Deletes every child subtree and invalidates their handles.
    pub fn delete_all_children(self) -> Self {
        self.apply(|element, app| element.delete_all_children(app))
    }

    pub fn id(self, id: &str) -> Self {
        self.apply(|element, app| {
            element.set_id(app, id);
        })
    }

    pub fn accessibility_name(self, name: &str) -> Self {
        self.apply(|element, app| {
            element.set_accessibility_name(app, name);
        })
    }

    pub fn add_event_listener(self, callback: EventCallbackKind, options: EventListenerOptions) -> Self {
        self.apply(|element, app| {
            element.add_event_listener(app, callback, options);
        })
    }

    pub fn add_pointer_enter_listener(self, callback: impl Fn(&mut PointerEnterEvent, &mut App) + 'static) -> Self {
        self.apply(|element, app| {
            element.add_pointer_enter_listener(app, callback);
        })
    }

    pub fn add_pointer_leave_listener(self, callback: impl Fn(&mut PointerLeaveEvent, &mut App) + 'static) -> Self {
        self.apply(|element, app| {
            element.add_pointer_leave_listener(app, callback);
        })
    }

    pub fn add_radio_value_changed_listener(
        self,
        callback: impl Fn(&mut RadioValueChangedEvent, &mut App) + 'static,
    ) -> Self {
        self.apply(|element, app| {
            element.add_radio_value_changed_listener(app, callback);
        })
    }

    pub fn add_checkbox_toggled_listener(
        self,
        callback: impl Fn(&mut CheckboxToggledEvent, &mut App) + 'static,
    ) -> Self {
        self.apply(|element, app| {
            element.add_checkbox_toggled_listener(app, callback);
        })
    }

    pub fn add_text_input_changed_listener(
        self,
        callback: impl Fn(&mut TextInputChangedEvent, &mut App) + 'static,
    ) -> Self {
        self.apply(|element, app| {
            element.add_text_input_changed_listener(app, callback);
        })
    }

    pub fn add_pointer_button_down_listener(
        self,
        callback: impl Fn(&mut PointerButtonEvent, &mut App) + 'static,
    ) -> Self {
        self.apply(|element, app| {
            element.add_pointer_button_down_listener(app, callback);
        })
    }

    pub fn add_pointer_moved_listener(self, callback: impl Fn(&mut PointerMovedEvent, &mut App) + 'static) -> Self {
        self.apply(|element, app| {
            element.add_pointer_moved_listener(app, callback);
        })
    }

    pub fn add_pointer_button_up_listener(
        self,
        callback: impl Fn(&mut PointerButtonEvent, &mut App) + 'static,
    ) -> Self {
        self.apply(|element, app| {
            element.add_pointer_button_up_listener(app, callback);
        })
    }

    pub fn add_click_listener(self, callback: impl Fn(&mut ClickEvent, &mut App) + 'static) -> Self {
        self.apply(|element, app| {
            element.add_click_listener(app, callback);
        })
    }

    pub fn add_custom_event_listener(self, callback: impl Fn(&mut CustomEvent, &mut App) + 'static) -> Self {
        self.apply(|element, app| {
            element.add_custom_event_listener(app, callback);
        })
    }

    pub fn emit_custom_event<T: Any + 'static>(self, detail: T) -> Self {
        self.apply(|element, app| element.emit_custom_event(app, detail))
    }

    pub fn add_focus_listener(self, callback: impl Fn(&mut FocusEvent, &mut App) + 'static) -> Self {
        self.apply(|element, app| {
            element.add_focus_listener(app, callback);
        })
    }

    pub fn add_unfocus_listener(self, callback: impl Fn(&mut UnfocusEvent, &mut App) + 'static) -> Self {
        self.apply(|element, app| {
            element.add_unfocus_listener(app, callback);
        })
    }

    pub fn add_lost_pointer_capture_listener(
        self,
        callback: impl Fn(&mut PointerCaptureEvent, &mut App) + 'static,
    ) -> Self {
        self.apply(|element, app| {
            element.add_lost_pointer_capture_listener(app, callback);
        })
    }

    pub fn add_got_pointer_capture_listener(
        self,
        callback: impl Fn(&mut PointerCaptureEvent, &mut App) + 'static,
    ) -> Self {
        self.apply(|element, app| {
            element.add_got_pointer_capture_listener(app, callback);
        })
    }

    pub fn add_keyboard_input_listener(self, callback: impl Fn(&mut KeyboardEvent, &mut App) + 'static) -> Self {
        self.apply(|element, app| {
            element.add_keyboard_input_listener(app, callback);
        })
    }

    pub fn add_slider_value_changed_listener(
        self,
        callback: impl Fn(&mut SliderValueChangedEvent, &mut App) + 'static,
    ) -> Self {
        self.apply(|element, app| {
            element.add_slider_value_changed_listener(app, callback);
        })
    }

    pub fn add_scroll_listener(self, callback: impl Fn(&mut ScrollEvent, &mut App) + 'static) -> Self {
        self.apply(|element, app| {
            element.add_scroll_listener(app, callback);
        })
    }

    pub fn scroll_to_child_by_id(self, id: &str) -> Self {
        self.apply(|element, app| {
            element.scroll_to_child_by_id(app, id);
        })
    }

    pub fn scroll_to_child_by_id_with_options(self, id: &str, options: ScrollOptions) -> Self {
        self.apply(|element, app| {
            element.scroll_to_child_by_id_with_options(app, id, options);
        })
    }

    pub fn scroll_to(self, y: f32) -> Self {
        self.apply(|element, app| {
            element.scroll_to(app, y);
        })
    }

    pub fn scroll_to_top(self) -> Self {
        self.apply(|element, app| {
            element.scroll_to_top(app);
        })
    }

    pub fn scroll_to_bottom(self) -> Self {
        self.apply(|element, app| {
            element.scroll_to_bottom(app);
        })
    }

    pub fn scroll_by(self, y: f32) -> Self {
        self.apply(|element, app| {
            element.scroll_by(app, y);
        })
    }

    pub fn display(self, value: Display) -> Self {
        self.apply(|element, app| {
            element.set_display(app, value);
        })
    }

    pub fn box_sizing(self, value: BoxSizing) -> Self {
        self.apply(|element, app| {
            element.set_box_sizing(app, value);
        })
    }

    pub fn position(self, value: Position) -> Self {
        self.apply(|element, app| {
            element.set_position(app, value);
        })
    }

    pub fn overlay(self, value: bool) -> Self {
        self.apply(|element, app| {
            element.set_overlay(app, value);
        })
    }

    pub fn margin(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, app| {
            element.set_margin(app, top, right, bottom, left);
        })
    }

    pub fn margin_all(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_margin_all(app, value);
        })
    }

    pub fn margin_vertical(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_margin_vertical(app, value);
        })
    }

    pub fn margin_horizontal(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_margin_horizontal(app, value);
        })
    }

    pub fn padding(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, app| {
            element.set_padding(app, top, right, bottom, left);
        })
    }

    pub fn padding_all(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_padding_all(app, value);
        })
    }

    pub fn padding_vertical(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_padding_vertical(app, value);
        })
    }

    pub fn padding_horizontal(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_padding_horizontal(app, value);
        })
    }

    pub fn gap(self, row_gap: Unit, column_gap: Unit) -> Self {
        self.apply(|element, app| {
            element.set_gap(app, row_gap, column_gap);
        })
    }

    pub fn row_gap(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_row_gap(app, value);
        })
    }

    pub fn column_gap(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_column_gap(app, value);
        })
    }

    pub fn inset(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, app| {
            element.set_inset(app, top, right, bottom, left);
        })
    }

    pub fn min_width(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_min_width(app, value);
        })
    }

    pub fn min_height(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_min_height(app, value);
        })
    }

    pub fn width(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_width(app, value);
        })
    }

    pub fn height(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_height(app, value);
        })
    }

    pub fn max_width(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_max_width(app, value);
        })
    }

    pub fn max_height(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_max_height(app, value);
        })
    }

    pub fn wrap(self, value: FlexWrap) -> Self {
        self.apply(|element, app| {
            element.set_wrap(app, value);
        })
    }

    pub fn align_items(self, value: AlignItems) -> Self {
        self.apply(|element, app| {
            element.set_align_items(app, value);
        })
    }

    pub fn align_self(self, value: AlignSelf) -> Self {
        self.apply(|element, app| {
            element.set_align_self(app, value);
        })
    }

    pub fn align_content(self, value: AlignContent) -> Self {
        self.apply(|element, app| {
            element.set_align_content(app, value);
        })
    }

    pub fn justify_content(self, value: JustifyContent) -> Self {
        self.apply(|element, app| {
            element.set_justify_content(app, value);
        })
    }

    pub fn flex_direction(self, value: FlexDirection) -> Self {
        self.apply(|element, app| {
            element.set_flex_direction(app, value);
        })
    }

    pub fn flex_grow(self, value: f32) -> Self {
        self.apply(|element, app| {
            element.set_flex_grow(app, value);
        })
    }

    pub fn flex_shrink(self, value: f32) -> Self {
        self.apply(|element, app| {
            element.set_flex_shrink(app, value);
        })
    }

    pub fn flex_basis(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_flex_basis(app, value);
        })
    }

    pub fn order(self, value: i32) -> Self {
        self.apply(|element, app| {
            element.set_order(app, value);
        })
    }

    pub fn font_family(self, value: FontFamily) -> Self {
        self.apply(|element, app| {
            element.set_font_family(app, value);
        })
    }

    pub fn color(self, value: Color) -> Self {
        self.apply(|element, app| {
            element.set_color(app, value);
        })
    }

    pub fn text_gradient(self, value: Gradient) -> Self {
        self.apply(|element, app| {
            element.set_text_gradient(app, value);
        })
    }

    pub fn background_color(self, value: Color) -> Self {
        self.apply(|element, app| {
            element.set_background_color(app, value);
        })
    }

    pub fn background_gradient(self, value: Gradient) -> Self {
        self.apply(|element, app| {
            element.set_background_gradient(app, value);
        })
    }

    pub fn font_size(self, value: f32) -> Self {
        self.apply(|element, app| {
            element.set_font_size(app, value);
        })
    }

    pub fn line_height(self, value: f32) -> Self {
        self.apply(|element, app| {
            element.set_line_height(app, value);
        })
    }

    pub fn font_weight(self, value: FontWeight) -> Self {
        self.apply(|element, app| {
            element.set_font_weight(app, value);
        })
    }

    pub fn font_style(self, value: FontStyle) -> Self {
        self.apply(|element, app| {
            element.set_font_style(app, value);
        })
    }

    pub fn text_align(self, value: TextAlign) -> Self {
        self.apply(|element, app| {
            element.set_text_align(app, value);
        })
    }

    pub fn underline(self, thickness: Option<f32>, color: Color, offset: Option<f32>) -> Self {
        self.apply(|element, app| {
            element.set_underline(app, thickness, color, offset);
        })
    }

    pub fn underline_gradient(self, thickness: Option<f32>, gradient: Gradient, offset: Option<f32>) -> Self {
        self.apply(|element, app| {
            element.set_underline_gradient(app, thickness, gradient, offset);
        })
    }

    pub fn overflow(self, overflow_x: Overflow, overflow_y: Overflow) -> Self {
        self.apply(|element, app| {
            element.set_overflow(app, overflow_x, overflow_y);
        })
    }

    pub fn overflow_x(self, value: Overflow) -> Self {
        self.apply(|element, app| {
            element.set_overflow_x(app, value);
        })
    }

    pub fn overflow_y(self, value: Overflow) -> Self {
        self.apply(|element, app| {
            element.set_overflow_y(app, value);
        })
    }

    pub fn border_color(self, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.apply(|element, app| {
            element.set_border_color(app, top, right, bottom, left);
        })
    }

    pub fn border_color_all(self, value: Color) -> Self {
        self.apply(|element, app| {
            element.set_border_color_all(app, value);
        })
    }

    pub fn border_color_vertical(self, value: Color) -> Self {
        self.apply(|element, app| {
            element.set_border_color_vertical(app, value);
        })
    }

    pub fn border_color_horizontal(self, value: Color) -> Self {
        self.apply(|element, app| {
            element.set_border_color_horizontal(app, value);
        })
    }

    pub fn border_width(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, app| {
            element.set_border_width(app, top, right, bottom, left);
        })
    }

    pub fn border_width_all(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_border_width_all(app, value);
        })
    }

    pub fn border_width_vertical(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_border_width_vertical(app, value);
        })
    }

    pub fn border_width_horizontal(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_border_width_horizontal(app, value);
        })
    }

    pub fn outline_color(self, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.apply(|element, app| {
            element.set_outline_color(app, top, right, bottom, left);
        })
    }

    pub fn outline_color_all(self, value: Color) -> Self {
        self.apply(|element, app| {
            element.set_outline_color_all(app, value);
        })
    }

    pub fn outline_color_vertical(self, value: Color) -> Self {
        self.apply(|element, app| {
            element.set_outline_color_vertical(app, value);
        })
    }

    pub fn outline_color_horizontal(self, value: Color) -> Self {
        self.apply(|element, app| {
            element.set_outline_color_horizontal(app, value);
        })
    }

    pub fn outline_width(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.apply(|element, app| {
            element.set_outline_width(app, top, right, bottom, left);
        })
    }

    pub fn outline_width_all(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_outline_width_all(app, value);
        })
    }

    pub fn outline_width_vertical(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_outline_width_vertical(app, value);
        })
    }

    pub fn outline_width_horizontal(self, value: Unit) -> Self {
        self.apply(|element, app| {
            element.set_outline_width_horizontal(app, value);
        })
    }

    pub fn border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.apply(|element, app| {
            element.set_border_radius(app, top, right, bottom, left);
        })
    }

    pub fn border_radius_all(self, value: (f32, f32)) -> Self {
        self.apply(|element, app| {
            element.set_border_radius_all(app, value);
        })
    }

    pub fn border_radius_vertical(self, value: (f32, f32)) -> Self {
        self.apply(|element, app| {
            element.set_border_radius_vertical(app, value);
        })
    }

    pub fn border_radius_horizontal(self, value: (f32, f32)) -> Self {
        self.apply(|element, app| {
            element.set_border_radius_horizontal(app, value);
        })
    }

    pub fn scrollbar_color(self, value: ScrollbarColor) -> Self {
        self.apply(|element, app| {
            element.set_scrollbar_color(app, value);
        })
    }

    pub fn scrollbar_thumb_margin(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.apply(|element, app| {
            element.set_scrollbar_thumb_margin(app, top, right, bottom, left);
        })
    }

    pub fn scrollbar_thumb_radius(
        self,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> Self {
        self.apply(|element, app| {
            element.set_scrollbar_thumb_radius(app, top, right, bottom, left);
        })
    }

    pub fn scrollbar_width(self, value: f32) -> Self {
        self.apply(|element, app| {
            element.set_scrollbar_width(app, value);
        })
    }

    pub fn animations(self, value: Vec<Animation>) -> Self {
        self.apply(|element, app| {
            element.set_animations(app, value);
        })
    }

    pub fn box_shadows(self, value: Vec<BoxShadow>) -> Self {
        self.apply(|element, app| {
            element.set_box_shadows(app, value);
        })
    }

    pub fn focus(self) -> Self {
        self.apply(|element, app| {
            element.focus(app);
        })
    }

    pub fn unfocus(self) -> Self {
        self.apply(|element, app| {
            element.unfocus(app);
        })
    }

    pub fn selectable(self, value: bool) -> Self
    where
        E: EditorIdentity<Handle = Text>,
    {
        self.apply(|element, app| {
            let element: Text = element.into_handle();
            element.set_selectable(app, value);
        })
    }

    pub fn text(self, value: &str) -> Self
    where
        E: TextContent,
    {
        self.apply(|element, app| {
            TextContent::set_text(element, &mut app.elements, &mut app.gummy_tree, value);
        })
    }

    pub fn text_smol_str(self, value: SmolStr) -> Self
    where
        E: EditorIdentity<Handle = Text>,
    {
        self.apply(|element, app| {
            let element: Text = element.into_handle();
            element.set_text_smol_str(app, value);
        })
    }

    pub fn disabled(self, value: bool) -> Self
    where
        E: EditorIdentity<Handle = TextInput>,
    {
        self.apply(|element, app| {
            let element: TextInput = element.into_handle();
            element.set_disabled(app, value);
        })
    }

    pub fn multiline(self, value: bool) -> Self
    where
        E: EditorIdentity<Handle = TextInput>,
    {
        self.apply(|element, app| {
            let element: TextInput = element.into_handle();
            element.set_multiline(app, value);
        })
    }

    pub fn ranged_styles(self, value: RangedStyles) -> Self
    where
        E: EditorIdentity<Handle = TextInput>,
    {
        self.apply(|element, app| {
            let element: TextInput = element.into_handle();
            element.set_ranged_styles(app, value);
        })
    }

    pub fn value(self, value: f64) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_value(app, value);
        })
    }

    pub fn step(self, value: f64) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_step(app, value);
        })
    }

    pub fn min(self, value: f64) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_min(app, value);
        })
    }

    pub fn max(self, value: f64) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_max(app, value);
        })
    }

    pub fn direction(self, value: SliderDirection) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_direction(app, value);
        })
    }

    pub fn thumb_size(self, value: f64) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_thumb_size(app, value);
        })
    }

    pub fn thumb_color(self, value: Brush) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_thumb_color(app, value);
        })
    }

    pub fn thumb_border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_thumb_border_radius(app, top, right, bottom, left);
        })
    }

    pub fn track_color(self, value: Color) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_track_color(app, value);
        })
    }

    pub fn track_gradient(self, value: Gradient) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_track_gradient(app, value);
        })
    }

    pub fn track_border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self
    where
        E: EditorIdentity<Handle = Slider>,
    {
        self.apply(|element, app| {
            let element: Slider = element.into_handle();
            element.set_track_border_radius(app, top, right, bottom, left);
        })
    }

    pub fn resource_id(self, value: ResourceId) -> Self
    where
        E: ResourceContent,
    {
        self.apply(|element, app| {
            ResourceContent::set_resource_id(
                element,
                &mut app.elements,
                &mut app.gummy_tree,
                &mut app.pending_resources,
                value,
            );
        })
    }

    pub fn selected_item(self, index: usize) -> Self
    where
        E: EditorIdentity<Handle = Dropdown>,
    {
        self.apply(|element, app| {
            let element: Dropdown = element.into_handle();
            element.set_selected_item(app, index);
        })
    }

    pub fn start_year(self, year: i32) -> Self
    where
        E: EditorIdentity<Handle = Calendar>,
    {
        self.apply(|element, app| {
            let element: Calendar = element.into_handle();
            element.set_start_year(app, year);
        })
    }

    pub fn end_year(self, year: i32) -> Self
    where
        E: EditorIdentity<Handle = Calendar>,
    {
        self.apply(|element, app| {
            let element: Calendar = element.into_handle();
            element.set_end_year(app, year);
        })
    }

    pub fn hide_radio(self) -> Self
    where
        E: EditorIdentity<Handle = Radio>,
    {
        self.apply(|element, app| {
            let element: Radio = element.into_handle();
            element.hide_radio(app);
        })
    }

    #[cfg(feature = "audio")]
    pub fn controls(self, value: bool) -> Self
    where
        E: EditorIdentity<Handle = Audio>,
    {
        self.apply(|element, app| {
            let element: Audio = element.into_handle();
            element.set_controls(app, value);
        })
    }

    #[cfg(feature = "audio")]
    pub fn play(self) -> Self
    where
        E: EditorIdentity<Handle = Audio>,
    {
        self.apply(|element, app| {
            let element: Audio = element.into_handle();
            element.play(app);
        })
    }

    #[cfg(feature = "audio")]
    pub fn pause(self) -> Self
    where
        E: EditorIdentity<Handle = Audio>,
    {
        self.apply(|element, app| {
            let element: Audio = element.into_handle();
            element.pause(app);
        })
    }

    #[cfg(feature = "audio")]
    pub fn toggle(self) -> Self
    where
        E: EditorIdentity<Handle = Audio>,
    {
        self.apply(|element, app| {
            let element: Audio = element.into_handle();
            element.toggle(app);
        })
    }
}

mod capabilities {
    use std::collections::VecDeque;

    use retgui_resource_manager::ResourceId;
    use retgui_resource_manager::resource_type::ResourceType;

    use crate::elements::{Element, Image, ImageElement, RetainedElements, Text, TextElement, TextInput, TextInputElement, TinyVg, TinyVgElement};
    use crate::layout::GummyTree;

    pub trait EditorIdentity: Element {
        type Handle: Element;

        fn into_handle(self) -> Self::Handle;
    }

    impl<E: Element> EditorIdentity for E {
        type Handle = E;

        fn into_handle(self) -> E {
            self
        }
    }

    pub trait Sealed {}

    impl Sealed for Text {}
    impl Sealed for TextInput {}
    impl Sealed for Image {}
    impl Sealed for TinyVg {}

    pub trait TextContent: Element + Sealed {
        fn set_text(self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree, value: &str);
    }

    impl TextContent for Text {
        fn set_text(self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree, value: &str) {
            if let Some(element) = elements.try_get_as_mut::<TextElement>(self.inner) {
                element.set_text(gummy_tree, value);
            }
        }
    }

    impl TextContent for TextInput {
        fn set_text(self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree, value: &str) {
            if let Some(element) = elements.try_get_as_mut::<TextInputElement>(self.inner) {
                element.set_text(gummy_tree, value);
            }
        }
    }

    pub trait ResourceContent: Element + Sealed {
        fn set_resource_id(
            self,
            elements: &mut RetainedElements,
            gummy_tree: &mut GummyTree,
            pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
            value: ResourceId,
        );
    }

    impl ResourceContent for Image {
        fn set_resource_id(
            self,
            elements: &mut RetainedElements,
            gummy_tree: &mut GummyTree,
            pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
            value: ResourceId,
        ) {
            if let Some(element) = elements.try_get_as_mut::<ImageElement>(self.inner) {
                element.set_image(gummy_tree, pending_resources, value);
            }
        }
    }

    impl ResourceContent for TinyVg {
        fn set_resource_id(
            self,
            elements: &mut RetainedElements,
            gummy_tree: &mut GummyTree,
            pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
            value: ResourceId,
        ) {
            if let Some(element) = elements.try_get_as_mut::<TinyVgElement>(self.inner) {
                element.set_resource_id(gummy_tree, pending_resources, value);
            }
        }
    }
}
