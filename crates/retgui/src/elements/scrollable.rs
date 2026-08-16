use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use issho::{AccessEvent, ScrollAmount, ScrollEvent};

use retgui_primitives::geometry::{Point, Vec2};

use ui_events::ScrollDelta;
use ui_events::keyboard::{Code, KeyState};
use ui_events::pointer::{PointerId, PointerType};

use crate::app::queue_event;
use crate::elements::ElementInternals;
use crate::elements::element_data::ElementData;
use crate::events::{Event, EventKind};
use crate::layout::layout::{CssComputedBorder, Layout, draw_borders_generic};
use crate::style::{Overflow, Style};
use retgui_primitives::geometry::borders::CssRoundedRect;
use retgui_primitives::geometry::{Rectangle, Size};
use retgui_renderer::renderer::Renderer;

/**

A scrollable gives an element the ability to scroll(transform) through overflowed children.
Internally when an element is created, it specifies if it is a scrollable. When an element specifics
that it is a scrollable, the element should call `on_scroll_events` in `on_events` and
`apply_scroll_layout` in apply_layout, so that scroll specific data is updated.

The element trait contains trait methods for user-level scroll methods,
but the internals of those APIs are defined in this file.
User API methods include:
    - scroll_to
    - scroll_by
    - scroll_to_child_by_id_with_options
    - scroll_to_top
    - scroll_to_bottom
**/

#[derive(Default, Clone, Copy)]
pub enum ScrollToBox {
    MarginBox,
    #[default]
    BorderBox,
    PaddingBox,
    ContentBox,
}

#[derive(Default, Clone, Copy)]
pub struct ScrollOptions {
    /// Which box the top of the scroll thumb will start at.
    pub to: ScrollToBox,
    pub offset: Option<Point>,
    // todo: Add an option to align the element itself in the scroll container.
}

impl ScrollOptions {
    pub fn new(to: ScrollToBox, offset: Point) -> Self {
        ScrollOptions {
            to,
            offset: Some(offset),
        }
    }
}

/// Stores state for elements with a scrollbar.
#[derive(Debug, Clone, Default, Copy)]
pub struct ScrollState {
    /// The total amount of vertical scroll.
    scroll_y: f32,

    /// Where the scrollbar was clicked.
    pub(crate) scroll_click: Option<Point>,

    // True if the scroll changes are new.
    is_new: bool,
}

impl ScrollState {
    /// Returns the total amount of vertical scroll.
    pub fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    pub fn mark_old(&mut self) {
        self.is_new = false;
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    /// Sets the total amount of vertical scroll.
    ///
    /// # Panics
    ///
    /// This function will panic if `scroll_y` is less than zero.
    pub fn set_scroll_y(&mut self, scroll_y: f32) {
        if scroll_y < 0.0 {
            panic!("Scroll cannot be negative.");
        }
        self.is_new = true;
        self.scroll_y = scroll_y;
    }
}

pub(crate) fn scroll_to_bottom(data: &mut ElementData) -> bool {
    let bottom_y = data.layout.max_scroll_y;
    scroll_to(data, bottom_y)
}

pub(crate) fn scroll_to_top(data: &mut ElementData) -> bool {
    scroll_to(data, 0.0)
}

/// Scroll to y. A valid y is in the interval [0, max_scroll_y].
pub(crate) fn scroll_to(data: &mut ElementData, y: f32) -> bool {
    if !data.is_scrollable() {
        return false;
    }

    let changed = set_scroll_y(&mut data.layout, y);
    data.apply_accessibility_scroll_data();

    let new_event = Event::new(data.me.upgrade().unwrap().clone());
    queue_event(new_event, EventKind::Scroll());
    changed
}

/// Scroll an amount y from the current scroll position.
pub(crate) fn scroll_by(data: &mut ElementData, y: f32) -> bool {
    scroll_to(data, data.scroll().scroll_y() + y)
}

/// Scrolls to a child with the `id` and uses level-order traversal.
pub(crate) fn scroll_to_child_by_id_with_options(data: &mut ElementData, id: &str, options: ScrollOptions) -> bool {
    let mut child_y: Option<f32> = None;
    if !data.is_scrollable() {
        return false;
    }

    let mut queue: VecDeque<(Rc<RefCell<dyn ElementInternals>>, Point)> = VecDeque::new();
    for child in data.children.as_slice() {
        let position = child.borrow().element_data().layout.local_box_in_parent().position;
        queue.push_back((child.clone(), position));
    }

    let top_py = data.layout.local_box().padding_rectangle().top();

    while let Some((child, offset)) = queue.pop_front() {
        let child = child.borrow();
        let element_data = child.element_data();
        if let Some(child_id) = element_data.id.as_ref()
            && child_id.as_str() == id
        {
            let local_box = element_data.layout.local_box();
            let box_model_selected = match options.to {
                ScrollToBox::BorderBox => local_box.border_rectangle(),
                ScrollToBox::MarginBox => local_box.margin_rectangle(),
                ScrollToBox::PaddingBox => local_box.padding_rectangle(),
                ScrollToBox::ContentBox => local_box.content_rectangle(),
            };
            let distance_from_parent = offset.y as f32 + box_model_selected.y - top_py;
            child_y = Some(distance_from_parent);
            break;
        }

        for descendant in child.children() {
            let local_position = descendant.borrow().element_data().layout.local_box_in_parent().position;
            queue.push_back((
                descendant.clone(),
                Point::new(offset.x + local_position.x, offset.y + local_position.y),
            ));
        }
    }

    if let Some(child_y) = child_y {
        let offset = options.offset.unwrap_or(Point::new(0.0, 0.0));
        scroll_to(data, child_y + offset.y as f32)
    } else {
        false
    }
}

/// Computes the scrollbar's tack and thumb layout.
pub(crate) fn apply_scroll_layout(style: &Style, layout: &mut Layout, gummy_layout: &gummy::Layout) {
    layout.scrollbar_thumb_margin = style.get_scrollbar_thumb_margin();
    layout.scrollbar_thumb_radius = style.get_scrollbar_thumb_radius();

    layout.scrollbar_size = Size::new(gummy_layout.scrollbar_size.width, gummy_layout.scrollbar_size.height);
    layout.computed_scrollbar_size = Size::new(gummy_layout.scroll_width(), gummy_layout.scroll_height());
    let state = &mut layout.scroll_state;

    if style.get_overflow()[1] != Overflow::Scroll {
        return;
    }

    let local_box = layout.computed_box;

    // Client Height = padding box height.
    let client_height = local_box.padding_rectangle().height;

    let mut content_height = layout.content_size.height;
    // Gummy is adding the top border and padding height to the content size.
    content_height -= local_box.border.top;
    content_height -= local_box.padding.top;

    // Content Size = overflowed content size + padding
    // Scroll Height = Content Size
    let scroll_height = (content_height + local_box.padding.bottom + local_box.padding.top).max(1.0);
    let scroll_track_width = layout.scrollbar_size.width;

    // The scroll track height is the height of the padding box.
    let scroll_track_height = client_height;

    let max_scroll_y = (scroll_height - client_height).max(0.0);
    layout.max_scroll_y = max_scroll_y;
    // The scroll amount can be updated by the user, but it should be clamped here when
    // the computed max scroll height is calculated.
    state.set_scroll_y(state.scroll_y().min(max_scroll_y));
    state.mark_old();

    layout.computed_scroll_track = Rectangle::new(
        local_box.padding_rectangle().right() - scroll_track_width,
        local_box.padding_rectangle().top(),
        scroll_track_width,
        scroll_track_height,
    );

    let visible_y = (client_height / scroll_height).clamp(0.0, 1.0);
    let scroll_thumb_height = scroll_track_height * visible_y;
    let scroll_thumb_height = scroll_thumb_height.max(15.0);
    let remaining_height = scroll_track_height - scroll_thumb_height;
    let scroll_thumb_offset = if max_scroll_y != 0.0 {
        (state.scroll_y() / max_scroll_y) * remaining_height
    } else {
        0.0
    };

    let thumb_margin = layout.scrollbar_thumb_margin;
    let scroll_thumb_width = scroll_track_width - (thumb_margin.left + thumb_margin.right);
    let scroll_thumb_height = (scroll_thumb_height - (thumb_margin.top + thumb_margin.bottom)).max(0.0);

    layout.computed_scroll_thumb = layout.computed_scroll_track;
    layout.computed_scroll_thumb.x += thumb_margin.left;
    layout.computed_scroll_thumb.y += scroll_thumb_offset + thumb_margin.top;
    layout.computed_scroll_thumb.width = scroll_thumb_width;
    layout.computed_scroll_thumb.height = scroll_thumb_height;
    update_scroll_thumb_position(layout);
    layout.scroll_state.mark_old();
}

/// Updates a scroll offset and its thumb geometry without invalidating layout.
/// Child movement is applied later by the draw traversal's local transform.
pub(crate) fn set_scroll_y(layout: &mut Layout, y: f32) -> bool {
    let y = y.max(0.0);
    let y = if layout.computed_scroll_track.height > 0.0 {
        y.min(layout.max_scroll_y)
    } else {
        y
    };

    if layout.scroll_state.scroll_y() == y {
        return false;
    }

    layout.scroll_state.set_scroll_y(y);
    update_scroll_thumb_position(layout);
    if layout.computed_scroll_track.height > 0.0 {
        layout.scroll_state.mark_old();
    }
    true
}

fn update_scroll_thumb_position(layout: &mut Layout) {
    let margin = layout.scrollbar_thumb_margin;
    let travel =
        (layout.computed_scroll_track.height - layout.computed_scroll_thumb.height - margin.top - margin.bottom)
            .max(0.0);
    let offset = if layout.max_scroll_y > 0.0 {
        layout.scroll_state.scroll_y() / layout.max_scroll_y * travel
    } else {
        0.0
    };
    layout.computed_scroll_thumb.y = layout.computed_scroll_track.y + margin.top + offset;
}

pub struct HandleScrollLogicResult {
    pub scroll_changed: bool,
    pub release_pointer_capture: bool,
    pub set_pointer_capture: bool,
    pub pointer_id: Option<PointerId>,
}

pub(crate) fn handle_scroll_logic(element: &mut dyn ElementInternals, message: &EventKind, event: &mut Event) {
    handle_scroll_logic_internal(element, message, event, true);
}

fn handle_scroll_logic_internal(
    element: &mut dyn ElementInternals,
    message: &EventKind,
    event: &mut Event,
    focus_on_pointer_down: bool,
) {
    let focus_on_pointer_down = focus_on_pointer_down
        && element.element_data().is_scrollable()
        && matches!(
            message,
            EventKind::PointerButtonDown(pointer_button)
                if pointer_button.button == Some(ui_events::pointer::PointerButton::Primary)
        );

    let result = {
        let element_data = element.element_data_mut();
        handle_scroll_logic_advance(&element_data.style, &mut element_data.layout, message, event)
    };

    if result.scroll_changed {
        element.element_data_mut().apply_accessibility_scroll_data();
        element.request_window_redraw();

        if matches!(message, EventKind::KeyboardInputEvent(_)) {
            queue_event(Event::new(element.to_rc()), EventKind::Scroll());
        }
    }

    if result.set_pointer_capture {
        element.set_pointer_capture(result.pointer_id.unwrap())
    }

    if result.release_pointer_capture {
        element.release_pointer_capture(result.pointer_id.unwrap());
    }

    if focus_on_pointer_down {
        element.focus();
        event.prevent_propagate();
    }
}

pub(crate) fn handle_scroll_logic_advance(
    style: &Style,
    layout: &mut Layout,
    message: &EventKind,
    event: &mut Event,
) -> HandleScrollLogicResult {
    let mut result = HandleScrollLogicResult {
        scroll_changed: false,
        release_pointer_capture: false,
        set_pointer_capture: false,
        pointer_id: message.pointer_id(),
    };

    if layout.is_scrollable_layout() && style.get_overflow()[1] == Overflow::Scroll {
        let world_box = layout.world_box();
        let world_scroll_thumb = layout.world_scroll_thumb();
        let world_scroll_track = layout.world_scroll_track();
        let page_height = layout.local_box().padding_rectangle().height.max(0.0);
        let state = &mut layout.scroll_state;
        match message {
            EventKind::PointerScroll(mouse_wheel) => {
                let delta = scroll_delta_y_in_logical_pixels(
                    mouse_wheel.delta,
                    mouse_wheel.state.scale_factor,
                    style.get_font_size().max(12.0) * style.get_line_height(),
                );
                let delta = -delta;
                // Todo: Scroll physics
                let max_scroll_y = layout.max_scroll_y;

                let current_scroll_y = state.scroll_y();
                let new_scroll_y = (current_scroll_y + delta).clamp(0.0, max_scroll_y);
                if new_scroll_y != current_scroll_y {
                    state.set_scroll_y(new_scroll_y);
                    result.scroll_changed = true;
                }

                event.prevent_propagate();
                event.prevent_defaults();
            }
            EventKind::PointerButtonDown(pointer_button)
                if pointer_button.button == Some(ui_events::pointer::PointerButton::Primary) =>
            {
                // DEVICE(TOUCH): Handle scrolling within the content area on touch based input devices.
                if pointer_button.pointer.pointer_type == PointerType::Touch {
                    let container_rectangle = world_box.padding_rectangle();

                    let in_scroll_bar = world_scroll_thumb.contains(&pointer_button.state.logical_point());

                    if container_rectangle.contains(&pointer_button.state.logical_point()) && !in_scroll_bar {
                        state.scroll_click = Some(Point::new(
                            pointer_button.state.logical_point().x,
                            pointer_button.state.logical_point().y,
                        ));
                        result.set_pointer_capture = true;
                        event.prevent_propagate();
                        event.prevent_defaults();
                    }
                } else if world_scroll_thumb.contains(&pointer_button.state.logical_point()) {
                    state.scroll_click = Some(Point::new(
                        pointer_button.state.logical_point().x,
                        pointer_button.state.logical_point().y,
                    ));

                    event.prevent_propagate();
                    event.prevent_defaults();

                    result.set_pointer_capture = true;
                } else if world_scroll_track.contains(&pointer_button.state.logical_point()) {
                    let offset_y = pointer_button.state.logical_point().y as f32 - world_scroll_track.y;

                    let percent = offset_y / world_scroll_track.height;
                    let scroll_y = percent * layout.max_scroll_y;

                    let new_scroll_y = scroll_y.clamp(0.0, layout.max_scroll_y);
                    if new_scroll_y != state.scroll_y() {
                        state.set_scroll_y(new_scroll_y);
                        result.scroll_changed = true;
                    }

                    event.prevent_propagate();
                    event.prevent_defaults();
                }
            }
            EventKind::PointerButtonUp(_pointer_button) if state.scroll_click.is_some() => {
                state.scroll_click = None;
                event.prevent_propagate();
                event.prevent_defaults();

                result.release_pointer_capture = true;
            }
            EventKind::PointerMovedEvent(pointer_motion) => {
                if let Some(click) = state.scroll_click {
                    // Todo: Translate scroll wheel pixel to scroll position for diff.
                    let pointer_position = pointer_motion.current.logical_point();
                    let delta = (pointer_position.y - click.y) as f32;

                    let max_scroll_y = layout.max_scroll_y;

                    let click_y_offset = layout.computed_scroll_track.height - layout.computed_scroll_thumb.height;
                    if click_y_offset <= 0.0 {
                        return result;
                    }
                    let mut delta = max_scroll_y * (delta / (click_y_offset));

                    // DEVICE(TOUCH): Reverse the direction on touch based input devices.
                    if pointer_motion.pointer.pointer_type == PointerType::Touch {
                        delta = -delta;
                    }

                    let current_scroll_y = state.scroll_y();
                    let new_scroll_y = (current_scroll_y + delta).clamp(0.0, max_scroll_y);
                    if new_scroll_y != current_scroll_y {
                        state.set_scroll_y(new_scroll_y);
                        result.scroll_changed = true;
                    }

                    state.scroll_click = Some(Point::new(click.x, pointer_position.y));
                    event.prevent_propagate();
                    event.prevent_defaults();
                }
            }
            EventKind::KeyboardInputEvent(keyboard_event)
                if keyboard_event.state == KeyState::Down
                    && !keyboard_event.modifiers.ctrl()
                    && !keyboard_event.modifiers.alt()
                    && !keyboard_event.modifiers.meta()
                    && layout.max_scroll_y > 0.0 =>
            {
                let current_scroll_y = state.scroll_y();
                let line_height = style.get_font_size().max(12.0) * style.get_line_height();
                let target_scroll_y = match keyboard_event.code {
                    Code::ArrowUp => Some(current_scroll_y - line_height),
                    Code::ArrowDown => Some(current_scroll_y + line_height),
                    Code::PageUp => Some(current_scroll_y - page_height),
                    Code::PageDown => Some(current_scroll_y + page_height),
                    Code::Home => Some(0.0),
                    Code::End => Some(layout.max_scroll_y),
                    _ => None,
                };

                if let Some(target_scroll_y) = target_scroll_y {
                    event.prevent_propagate();
                    event.prevent_defaults();

                    let new_scroll_y = target_scroll_y.clamp(0.0, layout.max_scroll_y);
                    if new_scroll_y != current_scroll_y {
                        state.set_scroll_y(new_scroll_y);
                        result.scroll_changed = true;
                    }
                }
            }
            _ => {}
        }
    };

    if result.scroll_changed {
        update_scroll_thumb_position(layout);
        layout.scroll_state.mark_old();
    }

    result
}

pub(crate) fn handle_accessibility_scroll_event(element: &mut dyn ElementInternals, event: &AccessEvent) {
    match event {
        AccessEvent::Scroll(scroll_event) => {
            if scroll_from_accessibility(element.element_data_mut(), *scroll_event) {
                element.request_window_redraw();
            }
        }
        AccessEvent::ScrollIntoView => {
            scroll_into_view(element);
        }
        _ => {}
    }
}

fn scroll_from_accessibility(data: &mut ElementData, event: ScrollEvent) -> bool {
    if !data.is_scrollable() {
        return false;
    }

    let current = data.layout.scroll_state.scroll_y();
    let viewport_height = data.layout.local_box().padding_rectangle().height.max(0.0);
    let line_height = data.style.get_font_size().max(12.0) * data.style.get_line_height();
    let target = match event.vertical {
        ScrollAmount::SmallIncrement => current + line_height,
        ScrollAmount::LargeIncrement => current + viewport_height,
        ScrollAmount::SmallDecrement => current - line_height,
        ScrollAmount::LargeDecrement => current - viewport_height,
        ScrollAmount::NoChange => return false,
        ScrollAmount::GoToPercentage(percentage) => {
            // UIA uses -1 (ScrollPatternNoScroll) when an axis should not change.
            if !percentage.is_finite() || percentage < 0.0 {
                return false;
            }
            data.layout.max_scroll_y * (percentage.clamp(0.0, 100.0) as f32 / 100.0)
        }
    };

    scroll_to(data, target)
}

fn scroll_into_view(element: &mut dyn ElementInternals) -> bool {
    let target = element.element_data().layout.world_box().border_rectangle();
    let mut ancestor = element.element_data().parent.clone();

    while let Some(ancestor_weak) = ancestor {
        let Some(ancestor_rc) = ancestor_weak.upgrade() else {
            return false;
        };

        let (next_ancestor, current, target_scroll) = {
            let ancestor = ancestor_rc.borrow();
            let data = ancestor.element_data();
            let next_ancestor = data.parent.clone();

            if !data.is_scrollable() {
                (next_ancestor, 0.0, None)
            } else {
                let viewport = data.layout.world_box().padding_rectangle();
                let current = data.layout.scroll_state.scroll_y();
                let delta = if target.top() < viewport.top() {
                    target.top() - viewport.top()
                } else if target.bottom() > viewport.bottom() {
                    target.bottom() - viewport.bottom()
                } else {
                    0.0
                };
                (next_ancestor, current, Some(current + delta))
            }
        };

        if let Some(target_scroll) = target_scroll {
            if target_scroll == current {
                return false;
            }
            let changed = scroll_to(ancestor_rc.borrow_mut().element_data_mut(), target_scroll);
            if changed {
                ancestor_rc.borrow().request_window_redraw();
            }
            return changed;
        }

        ancestor = next_ancestor;
    }

    false
}

fn scroll_delta_y_in_logical_pixels(delta: ScrollDelta, scale_factor: f64, logical_line_height: f32) -> f32 {
    match delta {
        ScrollDelta::LineDelta(_x, y) => y * logical_line_height,
        ScrollDelta::PixelDelta(physical) => (physical.y / scale_factor) as f32,
        ScrollDelta::PageDelta(_x, y) => y,
    }
}

pub fn draw_scrollbar(style: &Style, layout: &Layout, renderer: &mut dyn Renderer, scale_factor: f64) {
    if !(layout.is_scrollable_layout() && style.get_overflow()[1] == Overflow::Scroll) {
        return;
    }

    let border_color = style.get_border_color();
    let scrollbar_brush = style.get_scrollbar_brush();
    let scrollbar_thumb_radius = style
        .get_scrollbar_thumb_radius()
        .map(|radii| Vec2::new(radii.0 as f64 * scale_factor, radii.1 as f64 * scale_factor));
    // let scrollbar_thumb_radius = self.element_data().current_style().
    let track_rect = layout.computed_scroll_track.scale(scale_factor);
    let thumb_rect = layout.computed_scroll_thumb.scale(scale_factor);

    let border_spec = CssRoundedRect::new(thumb_rect.to_kurbo(), [0.0, 0.0, 0.0, 0.0], scrollbar_thumb_radius);
    let computed_border_spec = CssComputedBorder::new(border_spec);

    renderer.draw_rect(track_rect, scrollbar_brush.track_color);
    draw_borders_generic(
        renderer,
        &computed_border_spec,
        border_color.to_array(),
        scrollbar_brush.thumb_color,
    );
}
