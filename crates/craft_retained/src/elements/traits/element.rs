use std::cell::RefCell;
use std::rc::Rc;

use craft_primitives::Color;
use craft_primitives::geometry::ElementBox;
use ui_events::pointer::PointerId;
use winit::dpi::PhysicalPosition;
use winit::event::WindowEvent::{CursorMoved, MouseInput};
use winit::event::{DeviceId, ElementState, MouseButton};

use crate::CraftError;
use crate::app::queue_window_event;
use crate::elements::scrollable::{ScrollOptions, ScrollState};
use crate::elements::{AsElement, ElementInternals};
use crate::events::{KeyboardInputHandler, PointerCaptureHandler, PointerEnterHandler, PointerEventHandler, PointerLeaveHandler, PointerUpdateHandler, ScrollHandler};
use crate::style::{AlignItems, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, Underline, Unit};

/// Exposes a fluent/builder-pattern like API for elements.
/// Setters in this trait return Self and have no prefix.
/// Getters in this trait return specific data and have a get prefix.
pub trait Element: Clone + AsElement {
    fn get_children(&self) -> Vec<Rc<RefCell<dyn ElementInternals>>> {
        self.as_element_rc().borrow().children().to_vec()
    }

    fn get_previous_sibling(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        self.as_element_rc().borrow().get_previous_sibling()
    }

    fn get_next_sibling(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        self.as_element_rc().borrow().get_next_sibling()
    }

    fn get_parent(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        let parent = self.as_element_rc().borrow().parent();
        if let Some(parent) = parent {
            parent.upgrade().ok_or(CraftError::ElementNotFound)
        } else {
            Err(CraftError::ElementNotFound)
        }
    }

    fn get_first_child(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        self.as_element_rc().borrow().get_first_child()
    }

    fn get_last_child(&self) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        self.as_element_rc().borrow().get_last_child()
    }

    fn remove_child(
        &self,
        child: Rc<RefCell<dyn ElementInternals>>,
    ) -> Result<Rc<RefCell<dyn ElementInternals>>, CraftError> {
        self.as_element_rc().borrow_mut().remove_child(child)
    }

    fn remove_all_children(&self) {
        self.as_element_rc().borrow_mut().remove_all_children()
    }

    fn swap_child(
        &self,
        child_1: Rc<RefCell<dyn ElementInternals>>,
        child_2: Rc<RefCell<dyn ElementInternals>>,
    ) -> Result<(), CraftError> {
        self.as_element_rc().borrow_mut().swap_child(child_1, child_2)
    }

    fn push(self, child: impl AsElement) -> Self {
        let child_rc = child.as_element_rc();
        self.as_element_rc().borrow_mut().push(child_rc);
        self
    }

    fn on_pointer_enter(self, on_pointer_enter: PointerEnterHandler) -> Self {
        self.as_element_rc().borrow_mut().on_pointer_enter(on_pointer_enter);
        self
    }

    fn on_pointer_leave(self, on_pointer_leave: PointerLeaveHandler) -> Self {
        self.as_element_rc().borrow_mut().on_pointer_leave(on_pointer_leave);
        self
    }

    fn id(self, id: &str) -> Self {
        self.as_element_rc().borrow_mut().set_id(id);
        self
    }

    fn on_pointer_button_down(self, on_pointer_button_down: PointerEventHandler) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .on_pointer_button_down(on_pointer_button_down);
        self
    }

    fn on_pointer_moved(self, on_pointer_moved: PointerUpdateHandler) -> Self {
        self.as_element_rc().borrow_mut().on_pointer_moved(on_pointer_moved);
        self
    }

    fn on_pointer_button_up(self, on_pointer_button_up: PointerEventHandler) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .on_pointer_button_up(on_pointer_button_up);
        self
    }

    fn on_lost_pointer_capture(self, on_lost_pointer_capture: PointerCaptureHandler) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .on_lost_pointer_capture(on_lost_pointer_capture);
        self
    }

    fn on_got_pointer_capture(self, on_got_pointer_capture: PointerCaptureHandler) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .on_got_pointer_capture(on_got_pointer_capture);
        self
    }

    fn on_keyboard_input(self, on_keyboard_input: KeyboardInputHandler) -> Self {
        self.as_element_rc().borrow_mut().on_keyboard_input(on_keyboard_input);
        self
    }

    fn on_scroll(self, on_scroll: ScrollHandler) -> Self {
        self.as_element_rc().borrow_mut().on_scroll(on_scroll);
        self
    }

    fn scroll_to_child_by_id(self, id: &str) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .scroll_to_child_by_id_with_options(id, ScrollOptions::default());
        self
    }

    fn scroll_to_child_by_id_with_options(self, id: &str, options: ScrollOptions) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .scroll_to_child_by_id_with_options(id, options);
        self
    }
    fn scroll_to(self, y: f32) -> Self {
        self.as_element_rc().borrow_mut().scroll_to(y);
        self
    }

    fn scroll_to_top(self) -> Self {
        self.as_element_rc().borrow_mut().scroll_to_top();
        self
    }

    fn scroll_to_bottom(self) -> Self {
        self.as_element_rc().borrow_mut().scroll_to_bottom();
        self
    }

    fn scroll_by(self, y: f32) -> Self {
        self.as_element_rc().borrow_mut().scroll_by(y);
        self
    }

    fn get_scroll_state(&self) -> ScrollState {
        self.as_element_rc().borrow_mut().get_scroll_state()
    }

    fn display(self, display: Display) -> Self {
        self.as_element_rc().borrow_mut().set_display(display);
        self
    }

    fn box_sizing(self, box_sizing: BoxSizing) -> Self {
        self.as_element_rc().borrow_mut().set_box_sizing(box_sizing);
        self
    }

    fn position(self, position: Position) -> Self {
        self.as_element_rc().borrow_mut().set_position(position);
        self
    }

    fn margin(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_margin(top, right, bottom, left);
        self
    }

    fn padding(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_padding(top, right, bottom, left);
        self
    }

    fn gap(self, row_gap: Unit, column_gap: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_gap(row_gap, column_gap);
        self
    }

    fn inset(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_inset(top, right, bottom, left);
        self
    }

    fn min_width(self, min_width: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_min_width(min_width);
        self
    }

    fn min_height(self, min_height: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_min_height(min_height);
        self
    }

    fn width(self, width: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_width(width);
        self
    }

    fn height(self, height: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_height(height);
        self
    }

    fn max_width(self, max_width: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_max_width(max_width);
        self
    }

    fn max_height(self, max_height: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_max_height(max_height);
        self
    }

    fn wrap(self, wrap: FlexWrap) -> Self {
        self.as_element_rc().borrow_mut().set_wrap(wrap);
        self
    }

    fn align_items(self, align_items: Option<AlignItems>) -> Self {
        self.as_element_rc().borrow_mut().set_align_items(align_items);
        self
    }

    fn justify_content(self, justify_content: Option<JustifyContent>) -> Self {
        self.as_element_rc().borrow_mut().set_justify_content(justify_content);
        self
    }

    fn flex_direction(self, flex_direction: FlexDirection) -> Self {
        self.as_element_rc().borrow_mut().set_flex_direction(flex_direction);
        self
    }

    fn flex_grow(self, flex_grow: f32) -> Self {
        self.as_element_rc().borrow_mut().set_flex_grow(flex_grow);
        self
    }

    fn flex_shrink(self, flex_shrink: f32) -> Self {
        self.as_element_rc().borrow_mut().set_flex_shrink(flex_shrink);
        self
    }

    fn flex_basis(self, flex_basis: Unit) -> Self {
        self.as_element_rc().borrow_mut().set_flex_basis(flex_basis);
        self
    }

    fn font_family(self, font_family: FontFamily) -> Self {
        self.as_element_rc().borrow_mut().set_font_family(font_family);
        self
    }

    fn color(self, color: Color) -> Self {
        self.as_element_rc().borrow_mut().set_color(color);
        self
    }

    fn background_color(self, background_color: Color) -> Self {
        self.as_element_rc().borrow_mut().set_background_color(background_color);
        self
    }

    fn font_size(self, font_size: f32) -> Self {
        self.as_element_rc().borrow_mut().set_font_size(font_size);
        self
    }

    fn line_height(self, line_height: f32) -> Self {
        self.as_element_rc().borrow_mut().set_line_height(line_height);
        self
    }

    fn font_weight(self, font_weight: FontWeight) -> Self {
        self.as_element_rc().borrow_mut().set_font_weight(font_weight);
        self
    }

    fn font_style(self, font_style: FontStyle) -> Self {
        self.as_element_rc().borrow_mut().set_font_style(font_style);
        self
    }

    fn underline(self, underline: Option<Underline>) -> Self {
        self.as_element_rc().borrow_mut().set_underline(underline);
        self
    }

    fn overflow(self, overflow_x: Overflow, overflow_y: Overflow) -> Self {
        self.as_element_rc().borrow_mut().set_overflow(overflow_x, overflow_y);
        self
    }

    fn border_color(self, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .set_border_color(top, right, bottom, left);
        self
    }

    fn border_width(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .set_border_width(top, right, bottom, left);
        self
    }

    fn border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .set_border_radius(top, right, bottom, left);
        self
    }

    fn scrollbar_color(self, scrollbar_color: ScrollbarColor) -> Self {
        self.as_element_rc().borrow_mut().set_scrollbar_color(scrollbar_color);
        self
    }

    fn scrollbar_thumb_margin(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .set_scrollbar_thumb_margin(top, right, bottom, left);
        self
    }

    fn set_scrollbar_thumb_radius(
        self,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .set_scrollbar_thumb_radius(top, right, bottom, left);
        self
    }

    fn scrollbar_width(self, selection_color: Color) -> Self {
        self.as_element_rc().borrow_mut().set_selection_color(selection_color);
        self
    }

    fn box_shadows(self, box_shadows: Vec<BoxShadow>) -> Self {
        self.as_element_rc().borrow_mut().set_box_shadows(box_shadows);
        self
    }

    fn focus(self) -> Self {
        self.as_element_rc().borrow_mut().focus();
        self
    }

    fn is_focused(&self) -> bool {
        self.as_element_rc().borrow_mut().is_focused()
    }

    fn unfocus(self) -> Self {
        self.as_element_rc().borrow_mut().unfocus();
        self
    }

    fn get_computed_box_transformed(&self) -> ElementBox {
        self.as_element_rc().borrow().get_computed_box_transformed()
    }

    fn has_pointer_capture(&self, pointer_id: PointerId) -> bool {
        self.as_element_rc().borrow().has_pointer_capture(pointer_id)
    }

    #[allow(async_fn_in_trait)]
    async fn click(&self) {
        let pos = self
            .as_element_rc()
            .borrow()
            .get_computed_box_transformed()
            .padding_rectangle();
        let mouse_move = CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(pos.x as f64, pos.y as f64),
        };
        let mouse_down = MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        };
        let mouse_up = MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Released,
            button: MouseButton::Left,
        };
        let window_id = self.as_element_rc().borrow().get_winit_window().unwrap().id();
        queue_window_event(window_id, mouse_move);
        queue_window_event(window_id, mouse_down);
        queue_window_event(window_id, mouse_up);
    }
}
