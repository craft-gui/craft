use std::any::Any;
use std::rc::Rc;

use retgui_primitives::Color;
use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::ElementBox;
use retgui_primitives::gradient::Gradient;

use smol_str::SmolStr;

use crate::elements::internal_helpers::push_child_to_element;
use crate::elements::scrollable::{ScrollOptions, ScrollState};
use crate::elements::{DynElement, ElementEditor, ElementInternals, RetainedElements};
use crate::events::{CheckboxToggledEvent, ClickEvent, CustomEvent, EventCallbackKind, EventKind, EventListenerOptions, FocusEvent, KeyboardEvent, PointerButtonEvent, PointerCaptureEvent, PointerEnterEvent, PointerId, PointerLeaveEvent, PointerMovedEvent, RadioValueChangedEvent, ScrollEvent, SliderValueChangedEvent, TextInputChangedEvent, UnfocusEvent};
use crate::style::{AlignContent, AlignItems, AlignSelf, Animation, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, TextAlign, Underline, Unit};
use crate::{App, RetGuiError};

fn with_element<R>(
    element: DynElement,
    elements: &RetainedElements,
    callback: impl FnOnce(&dyn ElementInternals) -> R,
) -> Option<R> {
    elements.try_get(element).map(callback)
}

fn with_element_mut<R>(
    element: DynElement,
    elements: &mut RetainedElements,
    callback: impl FnOnce(&mut dyn ElementInternals) -> R,
) -> Option<R> {
    elements.try_get_mut(element).map(callback)
}

/// Exposes common element functionality likes styles and tree modifications.
pub trait Element: Copy {
    /// Returns an element as a DynElement.
    fn as_dyn_element(&self) -> DynElement;

    /// Bind elements while building.
    fn edit<'a>(&self, app: &'a mut App) -> ElementEditor<'a, Self>
    where
        Self: Sized,
    {
        ElementEditor::new(*self, app)
    }

    /// Requests a redraw of this element's owning window.
    fn request_redraw(&self, app: &App) {
        if let Some(element) = app.try_get(self.as_dyn_element()) {
            element.request_window_redraw();
        }
    }

    /// Returns the element's children.
    fn children(&self, app: &App) -> Vec<DynElement> {
        with_element(self.as_dyn_element(), &app.elements, |element| {
            element.get_children().to_vec()
        })
        .unwrap_or_default()
    }

    /// Returns the element's previous sibling or a not found error.
    fn previous_sibling(&self, app: &App) -> Result<DynElement, RetGuiError> {
        with_element(self.as_dyn_element(), &app.elements, |element| {
            element.get_previous_sibling(&app.elements)
        })
        .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Returns the element's next sibling or a not found error.
    fn next_sibling(&self, app: &App) -> Result<DynElement, RetGuiError> {
        with_element(self.as_dyn_element(), &app.elements, |element| {
            element.get_next_sibling(&app.elements)
        })
        .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Returns the element's parent or a not found error.
    fn parent(&self, app: &App) -> Result<DynElement, RetGuiError> {
        with_element(self.as_dyn_element(), &app.elements, |element| element.parent())
            .flatten()
            .ok_or(RetGuiError::ElementNotFound)
    }

    /// Returns the element's first child or a not found error.
    fn first_child(&self, app: &App) -> Result<DynElement, RetGuiError> {
        with_element(self.as_dyn_element(), &app.elements, |element| {
            element.get_first_child()
        })
        .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Returns the element's last child or a not found error.
    fn last_child(&self, app: &App) -> Result<DynElement, RetGuiError> {
        with_element(self.as_dyn_element(), &app.elements, |element| element.get_last_child())
            .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Removes the element's child or a not found error.
    fn remove_child(&self, app: &mut App, child: DynElement) -> Result<DynElement, RetGuiError> {
        let handle = self.as_dyn_element();
        app.elements
            .try_dispatch_mut(handle, |element, arena| {
                element.remove_child(arena, &mut app.gummy_tree, &mut app.event_queue, &mut app.focus, child)
            })
            .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Detaches the element's children while keeping their handles valid.
    ///
    /// Use [`delete_all_children`](Self::delete_all_children) when the removed
    /// subtrees should be destroyed and their arena storage reclaimed.
    fn remove_all_children(&self, app: &mut App) {
        let handle = self.as_dyn_element();
        app.elements.try_dispatch_mut(handle, |element, arena| {
            element.remove_all_children(arena, &mut app.gummy_tree, &mut app.event_queue, &mut app.focus)
        });
    }

    /// Deletes all direct children and their retained subtrees from the store.
    ///
    /// Unlike [`remove_all_children`](Self::remove_all_children), this
    /// invalidates every copy of each removed handle and reclaims its arena,
    /// layout, and accessibility storage.
    fn delete_all_children(&self, app: &mut App) {
        let handle = self.as_dyn_element();
        if app.contains(handle) {
            app.elements.delete_all_children(
                &mut app.gummy_tree,
                &mut app.by_internal_id,
                &mut app.event_queue,
                &mut app.focus,
                handle,
            );
        }
    }

    /// Swaps the element's children or returns a not found error if either child is missing.
    fn swap_child(&self, app: &mut App, child_1: DynElement, child_2: DynElement) -> Result<(), RetGuiError> {
        let handle = self.as_dyn_element();
        app.elements
            .try_dispatch_mut(handle, |element, arena| {
                element.swap_child(arena, &mut app.gummy_tree, child_1, child_2)
            })
            .unwrap_or(Err(RetGuiError::ElementNotFound))
    }

    /// Pushes a child.
    fn push(&self, app: &mut App, child: impl Element) {
        let child = child.as_dyn_element();
        push_child_to_element(&mut app.elements, &mut app.gummy_tree, self.as_dyn_element(), child);
    }

    /// Adds an event listener.
    fn add_event_listener(&self, app: &mut App, callback: EventCallbackKind, options: EventListenerOptions) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.add_event_listener(callback, options)
        });
    }

    /// Adds a pointer enter listener.
    fn add_pointer_enter_listener(
        &self,
        app: &mut App,
        on_pointer_enter: impl Fn(&mut PointerEnterEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_pointer_enter(Rc::new(on_pointer_enter))
        });
    }

    /// Adds a pointer leave listener.
    fn add_pointer_leave_listener(
        &self,
        app: &mut App,
        on_pointer_leave: impl Fn(&mut PointerLeaveEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_pointer_leave(Rc::new(on_pointer_leave))
        });
    }

    /// Adds a radio value changed listener.
    fn add_radio_value_changed_listener(
        &self,
        app: &mut App,
        on_radio_value_changed: impl Fn(&mut RadioValueChangedEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_radio_value_changed(Rc::new(on_radio_value_changed))
        });
    }

    /// Adds a checkbox toggled listener.
    fn add_checkbox_toggled_listener(
        &self,
        app: &mut App,
        on_checkbox_toggled: impl Fn(&mut CheckboxToggledEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_checkbox_toggled(Rc::new(on_checkbox_toggled))
        });
    }

    /// Adds a text input changed listener.
    fn add_text_input_changed_listener(
        &self,
        app: &mut App,
        on_text_input_changed: impl Fn(&mut TextInputChangedEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_text_input_changed(Rc::new(on_text_input_changed))
        });
    }

    /// Sets the element's user based id.
    fn set_id(&self, app: &mut App, id: &str) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| element.set_id(id));
    }

    /// Sets the accessibility name.
    fn set_accessibility_name(&self, app: &mut App, name: &str) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.element_data_mut().set_accessibility_name(name)
        });
    }

    /// Returns the element's user based id. This id is not used by RetGUI.
    fn id(&self, app: &App) -> Option<SmolStr> {
        with_element(self.as_dyn_element(), &app.elements, |element| element.get_id()).flatten()
    }

    /// Adds a pointer button down listener.
    fn add_pointer_button_down_listener(
        &self,
        app: &mut App,
        on_pointer_button_down: impl Fn(&mut PointerButtonEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_pointer_button_down(Rc::new(on_pointer_button_down))
        });
    }

    /// Adds a pointer button moved listener.
    fn add_pointer_moved_listener(
        &self,
        app: &mut App,
        on_pointer_moved: impl Fn(&mut PointerMovedEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_pointer_moved(Rc::new(on_pointer_moved))
        });
    }

    /// Adds a pointer button up listener.
    fn add_pointer_button_up_listener(
        &self,
        app: &mut App,
        on_pointer_button_up: impl Fn(&mut PointerButtonEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_pointer_button_up(Rc::new(on_pointer_button_up))
        });
    }

    /// Adds a click listener.
    fn add_click_listener(&self, app: &mut App, on_click: impl Fn(&mut ClickEvent, &mut App) + 'static) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_click(Rc::new(on_click))
        });
    }

    /// Adds a custom event listener.
    fn add_custom_event_listener(&self, app: &mut App, on_custom_event: impl Fn(&mut CustomEvent, &mut App) + 'static) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_custom_event(Rc::new(on_custom_event))
        });
    }

    /// Emits a custom event using the element as the target element.
    fn emit_custom_event<T: Any + 'static>(&self, app: &mut App, detail: T) {
        let handle = self.as_dyn_element();
        if app.contains(handle) {
            app.event_queue
                .push_back(EventKind::Custom(CustomEvent::new(handle, detail)));
        }
    }

    /// Adds a focus event listener.
    fn add_focus_listener(&self, app: &mut App, on_focus: impl Fn(&mut FocusEvent, &mut App) + 'static) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_focus(Rc::new(on_focus))
        });
    }

    /// Adds an unfocus event listener.
    fn add_unfocus_listener(&self, app: &mut App, on_unfocus: impl Fn(&mut UnfocusEvent, &mut App) + 'static) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_unfocus(Rc::new(on_unfocus))
        });
    }

    /// Adds a lost pointer capture event listener.
    fn add_lost_pointer_capture_listener(
        &self,
        app: &mut App,
        on_lost_pointer_capture: impl Fn(&mut PointerCaptureEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_lost_pointer_capture(Rc::new(on_lost_pointer_capture))
        });
    }

    /// Adds a got pointer capture event listener.
    fn add_got_pointer_capture_listener(
        &self,
        app: &mut App,
        on_got_pointer_capture: impl Fn(&mut PointerCaptureEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_got_pointer_capture(Rc::new(on_got_pointer_capture))
        });
    }

    /// Adds a keyboard input event listener.
    fn add_keyboard_input_listener(
        &self,
        app: &mut App,
        on_keyboard_input: impl Fn(&mut KeyboardEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_keyboard_input(Rc::new(on_keyboard_input))
        });
    }

    /// Adds a slider value changed event listener.
    fn add_slider_value_changed_listener(
        &self,
        app: &mut App,
        on_slider_value_changed: impl Fn(&mut SliderValueChangedEvent, &mut App) + 'static,
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_slider_value_changed(Rc::new(on_slider_value_changed))
        });
    }

    /// Adds a scroll event listener.
    fn add_scroll_listener(&self, app: &mut App, on_scroll: impl Fn(&mut ScrollEvent, &mut App) + 'static) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.on_scroll(Rc::new(on_scroll))
        });
    }

    /// Scrolls to a child based on the child's user id.
    fn scroll_to_child_by_id(&self, app: &mut App, id: &str) {
        let handle = self.as_dyn_element();
        app.elements.try_dispatch_mut(handle, |element, arena| {
            element.scroll_to_child_by_id_with_options(arena, &mut app.event_queue, id, ScrollOptions::default())
        });
    }

    /// Scrolls to a child based on the child's user id according to the scroll options.
    fn scroll_to_child_by_id_with_options(&self, app: &mut App, id: &str, options: ScrollOptions) {
        let handle = self.as_dyn_element();
        app.elements.try_dispatch_mut(handle, |element, arena| {
            element.scroll_to_child_by_id_with_options(arena, &mut app.event_queue, id, options)
        });
    }

    /// Scrolls to a specific y value in logical pixels.
    fn scroll_to(&self, app: &mut App, y: f32) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.scroll_to(&mut app.event_queue, y)
        });
    }

    /// Scrolls to the top of the element.
    fn scroll_to_top(&self, app: &mut App) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.scroll_to_top(&mut app.event_queue)
        });
    }

    /// Scrolls to the button of the element.
    fn scroll_to_bottom(&self, app: &mut App) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.scroll_to_bottom(&mut app.event_queue)
        });
    }

    /// Scrolls by a logical amount of pixels.
    fn scroll_by(&self, app: &mut App, y: f32) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.scroll_by(&mut app.event_queue, y)
        });
    }

    /// Returns the elements current scroll state.
    fn scroll_state(&self, app: &App) -> ScrollState {
        with_element(self.as_dyn_element(), &app.elements, |element| {
            element.get_scroll_state()
        })
        .unwrap_or_default()
    }

    /// Sets the layout algorith e.g. block, flex, etc.
    fn set_display(&self, app: &mut App, display: Display) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_display(&mut app.gummy_tree, display)
        });
    }

    /// Sets the box sizing e.g. content box/border box.
    fn set_box_sizing(&self, app: &mut App, box_sizing: BoxSizing) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_box_sizing(&mut app.gummy_tree, box_sizing)
        });
    }

    /// Sets the position of the element.
    ///
    /// Unlike HTML, this has no effect on the visual order of the element.
    fn set_position(&self, app: &mut App, position: Position) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_position(&mut app.gummy_tree, position)
        });
    }

    /// Puts the element on top of other elements.
    fn set_overlay(&self, app: &mut App, overlay: bool) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_overlay(overlay)
        });
    }

    /// Returns if the element is put on top of other elements.
    fn is_overlay(&self, app: &App) -> bool {
        with_element(self.as_dyn_element(), &app.elements, |element| {
            element.style().get_overlay()
        })
        .unwrap_or(false)
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin(&self, app: &mut App, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_margin(&mut app.gummy_tree, top, right, bottom, left)
        });
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin_all(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_margin_all(&mut app.gummy_tree, value)
        });
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin_vertical(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_margin_vertical(&mut app.gummy_tree, value)
        });
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin_horizontal(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_margin_horizontal(&mut app.gummy_tree, value)
        });
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding(&self, app: &mut App, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_padding(&mut app.gummy_tree, top, right, bottom, left)
        });
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding_all(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_padding_all(&mut app.gummy_tree, value)
        });
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding_vertical(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_padding_vertical(&mut app.gummy_tree, value)
        });
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding_horizontal(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_padding_horizontal(&mut app.gummy_tree, value)
        });
    }

    /// Sets the gap between children for flex/grid containers.
    fn set_gap(&self, app: &mut App, row_gap: Unit, column_gap: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_gap(&mut app.gummy_tree, row_gap, column_gap)
        });
    }

    /// Sets the row gap between children for flex/grid containers.
    fn set_row_gap(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_row_gap(&mut app.gummy_tree, value)
        });
    }

    /// Sets the column gap between children for flex/grid containers.
    fn set_column_gap(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_column_gap(&mut app.gummy_tree, value)
        });
    }

    /// Align the element relative to its sides. Only applies to positioned elements.
    fn set_inset(&self, app: &mut App, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_inset(&mut app.gummy_tree, top, right, bottom, left)
        });
    }

    /// Sets the minium width of the element.
    fn set_min_width(&self, app: &mut App, min_width: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_min_width(&mut app.gummy_tree, min_width)
        });
    }

    /// Sets the minium height of the element.
    fn set_min_height(&self, app: &mut App, min_height: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_min_height(&mut app.gummy_tree, min_height)
        });
    }

    /// Sets the width of the element.
    fn set_width(&self, app: &mut App, width: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_width(&mut app.gummy_tree, width)
        });
    }

    /// Sets the height of the element.
    fn set_height(&self, app: &mut App, height: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_height(&mut app.gummy_tree, height)
        });
    }

    /// Sets the max width of the element.
    fn set_max_width(&self, app: &mut App, max_width: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_max_width(&mut app.gummy_tree, max_width)
        });
    }

    /// Sets the max height of the element.
    fn set_max_height(&self, app: &mut App, max_height: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_max_height(&mut app.gummy_tree, max_height)
        });
    }

    /// Sets the wrapping behavior for flex elements.
    fn set_wrap(&self, app: &mut App, wrap: FlexWrap) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_wrap(&mut app.gummy_tree, wrap)
        });
    }

    /// Determines how flex/grid children are laid out on the cross axis.
    fn set_align_items(&self, app: &mut App, align_items: AlignItems) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_align_items(&mut app.gummy_tree, align_items)
        });
    }

    /// Overrides a parent's align_items.
    fn set_align_self(&self, app: &mut App, align_self: AlignSelf) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_align_self(&mut app.gummy_tree, align_self)
        });
    }

    fn set_align_content(&self, app: &mut App, align_content: AlignContent) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_align_content(&mut app.gummy_tree, align_content)
        });
    }

    fn set_justify_content(&self, app: &mut App, justify_content: JustifyContent) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_justify_content(&mut app.gummy_tree, justify_content)
        });
    }

    fn set_flex_direction(&self, app: &mut App, flex_direction: FlexDirection) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_flex_direction(&mut app.gummy_tree, flex_direction)
        });
    }

    fn set_flex_grow(&self, app: &mut App, flex_grow: f32) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_flex_grow(&mut app.gummy_tree, flex_grow)
        });
    }

    fn set_flex_shrink(&self, app: &mut App, flex_shrink: f32) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_flex_shrink(&mut app.gummy_tree, flex_shrink)
        });
    }

    fn set_flex_basis(&self, app: &mut App, flex_basis: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_flex_basis(&mut app.gummy_tree, flex_basis)
        });
    }

    fn set_order(&self, app: &mut App, order: i32) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_order(&mut app.gummy_tree, order)
        });
    }

    fn set_font_family(&self, app: &mut App, font_family: FontFamily) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_font_family(&mut app.gummy_tree, font_family)
        });
    }

    fn set_color(&self, app: &mut App, color: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_text_brush(&mut app.gummy_tree, Brush::Color(color))
        });
    }

    fn set_text_gradient(&self, app: &mut App, gradient: Gradient) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_text_brush(&mut app.gummy_tree, Brush::Gradient(gradient))
        });
    }

    fn set_background_color(&self, app: &mut App, background_color: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_background_brush(Brush::Color(background_color))
        });
    }

    fn set_background_gradient(&self, app: &mut App, gradient: Gradient) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_background_brush(Brush::Gradient(gradient))
        });
    }

    fn set_font_size(&self, app: &mut App, font_size: f32) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_font_size(&mut app.gummy_tree, font_size)
        });
    }

    fn set_line_height(&self, app: &mut App, line_height: f32) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_line_height(&mut app.gummy_tree, line_height)
        });
    }

    fn set_font_weight(&self, app: &mut App, font_weight: FontWeight) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_font_weight(&mut app.gummy_tree, font_weight)
        });
    }

    fn set_font_style(&self, app: &mut App, font_style: FontStyle) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_font_style(&mut app.gummy_tree, font_style)
        });
    }

    fn set_text_align(&self, app: &mut App, text_align: TextAlign) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_text_align(&mut app.gummy_tree, text_align)
        });
    }

    fn set_underline(&self, app: &mut App, thickness: Option<f32>, color: Color, offset: Option<f32>) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_underline(
                &mut app.gummy_tree,
                Some(Underline {
                    thickness,
                    brush: Brush::Color(color),
                    offset,
                }),
            )
        });
    }

    fn set_underline_gradient(&self, app: &mut App, thickness: Option<f32>, gradient: Gradient, offset: Option<f32>) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_underline(
                &mut app.gummy_tree,
                Some(Underline {
                    thickness,
                    brush: Brush::Gradient(gradient),
                    offset,
                }),
            )
        });
    }

    fn set_overflow(&self, app: &mut App, overflow_x: Overflow, overflow_y: Overflow) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_overflow(&mut app.gummy_tree, overflow_x, overflow_y)
        });
    }

    fn set_overflow_x(&self, app: &mut App, overflow_x: Overflow) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_overflow_x(&mut app.gummy_tree, overflow_x)
        });
    }

    fn set_overflow_y(&self, app: &mut App, overflow_y: Overflow) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_overflow_y(&mut app.gummy_tree, overflow_y)
        });
    }

    fn set_border_color(&self, app: &mut App, top: Color, right: Color, bottom: Color, left: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_color(top, right, bottom, left)
        });
    }

    fn set_border_color_all(&self, app: &mut App, value: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_color_all(value)
        });
    }

    fn set_border_color_vertical(&self, app: &mut App, value: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_color_vertical(value)
        });
    }

    fn set_border_color_horizontal(&self, app: &mut App, value: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_color_horizontal(value)
        });
    }

    fn set_border_width(&self, app: &mut App, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_width(&mut app.gummy_tree, top, right, bottom, left)
        });
    }

    fn set_border_width_all(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_width_all(&mut app.gummy_tree, value)
        });
    }

    fn set_border_width_vertical(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_width_vertical(&mut app.gummy_tree, value)
        });
    }

    fn set_border_width_horizontal(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_width_horizontal(&mut app.gummy_tree, value)
        });
    }

    fn set_outline_color(&self, app: &mut App, top: Color, right: Color, bottom: Color, left: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_outline_color(top, right, bottom, left)
        });
    }

    fn set_outline_color_all(&self, app: &mut App, value: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_outline_color_all(value)
        });
    }

    fn set_outline_color_vertical(&self, app: &mut App, value: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_outline_color_vertical(value)
        });
    }

    fn set_outline_color_horizontal(&self, app: &mut App, value: Color) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_outline_color_horizontal(value)
        });
    }

    fn set_outline_width(&self, app: &mut App, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_outline_width(top, right, bottom, left)
        });
    }

    fn set_outline_width_all(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_outline_width_all(value)
        });
    }

    fn set_outline_width_vertical(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_outline_width_vertical(value)
        });
    }

    fn set_outline_width_horizontal(&self, app: &mut App, value: Unit) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_outline_width_horizontal(value)
        });
    }

    fn set_border_radius(
        &self,
        app: &mut App,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_radius(top, right, bottom, left)
        });
    }

    fn set_border_radius_all(&self, app: &mut App, value: (f32, f32)) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_radius_all(value)
        });
    }

    fn set_border_radius_vertical(&self, app: &mut App, value: (f32, f32)) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_radius_vertical(value)
        });
    }

    fn set_border_radius_horizontal(&self, app: &mut App, value: (f32, f32)) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_border_radius_horizontal(value)
        });
    }

    fn set_scrollbar_color(&self, app: &mut App, scrollbar_color: ScrollbarColor) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_scrollbar_color(scrollbar_color)
        });
    }

    fn set_scrollbar_thumb_margin(&self, app: &mut App, top: f32, right: f32, bottom: f32, left: f32) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_scrollbar_thumb_margin(&mut app.gummy_tree, top, right, bottom, left)
        });
    }

    fn set_scrollbar_thumb_radius(
        &self,
        app: &mut App,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_scrollbar_thumb_radius(&mut app.gummy_tree, top, right, bottom, left)
        });
    }

    fn set_scrollbar_width(&self, app: &mut App, scrollbar_width: f32) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_scrollbar_width(&mut app.gummy_tree, scrollbar_width)
        });
    }

    /// Sets the list of animations.
    fn set_animations(&self, app: &mut App, animations: Vec<Animation>) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_animations(&mut app.pending_animation_updates, animations)
        });
    }

    /// Sets the box shadows on this element.
    fn set_box_shadows(&self, app: &mut App, box_shadows: Vec<BoxShadow>) {
        with_element_mut(self.as_dyn_element(), &mut app.elements, |element| {
            element.set_box_shadows(box_shadows)
        });
    }

    /// Focus the element.
    fn focus(&self, app: &mut App) {
        let handle = self.as_dyn_element();
        app.elements.try_dispatch_mut(handle, |element, arena| {
            element.focus(arena, &mut app.event_queue, &mut app.focus, app.focus_outline_visible)
        });
    }

    /// Returns whether the element current has focus.
    fn is_focused(&self, app: &App) -> bool {
        with_element(self.as_dyn_element(), &app.elements, |element| element.is_focused()).unwrap_or(false)
    }

    /// Unfocuses the element.
    fn unfocus(&self, app: &mut App) {
        let handle = self.as_dyn_element();
        app.elements.try_dispatch_mut(handle, |element, _arena| {
            element.unfocus(&mut app.event_queue, &mut app.focus)
        });
    }

    /// Get the elements box in logical pixels.
    fn computed_box_transformed(&self, app: &App) -> ElementBox {
        with_element(self.as_dyn_element(), &app.elements, |element| {
            element.get_computed_box_transformed()
        })
        .unwrap_or_default()
    }

    /// Returns whether the element has pointer capture.
    fn has_pointer_capture(&self, app: &App, pointer_id: PointerId) -> bool {
        with_element(self.as_dyn_element(), &app.elements, |element| {
            element.has_pointer_capture(&app.elements, pointer_id)
        })
        .unwrap_or(false)
    }

    /// Captures subsequent events for this pointer on the element.
    fn set_pointer_capture(&self, app: &mut App, pointer_id: PointerId) {
        app.elements.try_dispatch_mut(self.as_dyn_element(), |element, arena| {
            element.set_pointer_capture(arena, pointer_id)
        });
    }

    /// Releases this element's capture of the pointer.
    fn release_pointer_capture(&self, app: &mut App, pointer_id: PointerId) {
        app.elements.try_dispatch_mut(self.as_dyn_element(), |element, arena| {
            element.release_pointer_capture(arena, pointer_id)
        });
    }
}
