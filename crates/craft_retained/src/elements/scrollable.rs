use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use craft_primitives::geometry::{Point, Vec2};

use ui_events::ScrollDelta;
use ui_events::pointer::{PointerId, PointerType};

use crate::app::{queue_event, request_apply_layout};
use crate::elements::ElementInternals;
use crate::elements::element_data::ElementData;
use crate::events::{Event, EventKind};
use crate::layout::layout::{CssComputedBorder, Layout, draw_borders_generic};
use crate::style::{Overflow, Style};
use craft_primitives::geometry::borders::CssRoundedRect;
use craft_primitives::geometry::{Rectangle, Size};
use craft_renderer::RenderList;

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

pub(crate) fn scroll_to_bottom(data: &mut ElementData) {
    let bottom_y = data.layout.max_scroll_y;
    scroll_to(data, bottom_y);
}

pub(crate) fn scroll_to_top(data: &mut ElementData) {
    scroll_to(data, 0.0);
}

/// Scroll to y. A valid y is in the interval [0, max_scroll_y].
pub(crate) fn scroll_to(data: &mut ElementData, y: f32) {
    if !data.is_scrollable() {
        return;
    }

    data.layout.scroll_state.set_scroll_y(f32::max(0.0, y));
    let new_event = Event::new(data.me.upgrade().unwrap().clone());
    request_apply_layout(data.layout.taffy_node_id.unwrap());
    queue_event(new_event, EventKind::Scroll());
}

/// Scroll an amount y from the current scroll position.
pub(crate) fn scroll_by(data: &mut ElementData, y: f32) {
    scroll_to(data, data.scroll().scroll_y() + y);
}

/// Scrolls to a child with the `id` and uses level-order traversal.
pub(crate) fn scroll_to_child_by_id_with_options(data: &mut ElementData, id: &str, options: ScrollOptions) {
    let mut child_y: Option<f32> = None;
    if !data.is_scrollable() {
        return;
    }

    let mut queue: VecDeque<Rc<RefCell<dyn ElementInternals>>> = VecDeque::new();
    for child in data.children.as_slice() {
        queue.push_back(child.clone());
    }

    let top_py = data.layout.computed_box.padding_rectangle().top();

    while let Some(child) = queue.pop_front().clone() {
        let child = child.borrow();
        let element_data = child.element_data();
        if let Some(child_id) = element_data.id.as_ref()
            && child_id.as_str() == id
        {
            let box_model_selected = match options.to {
                ScrollToBox::BorderBox => element_data.layout.computed_box.border_rectangle(),
                ScrollToBox::MarginBox => element_data.layout.computed_box.margin_rectangle(),
                ScrollToBox::PaddingBox => element_data.layout.computed_box.padding_rectangle(),
                ScrollToBox::ContentBox => element_data.layout.computed_box.content_rectangle(),
            };
            let distance_from_parent = box_model_selected.y - top_py;
            child_y = Some(distance_from_parent);
            break;
        }

        for child in child.children() {
            queue.push_back(child.clone());
        }
    }

    if let Some(child_y) = child_y {
        let offset = options.offset.unwrap_or(Point::new(0.0, 0.0));
        scroll_to(data, child_y + offset.y as f32);
    }
}

/// Computes the scrollbar's tack and thumb layout.
pub(crate) fn apply_scroll_layout(style: &Style, layout: &mut Layout, taffy_layout: &taffy::Layout) {
    layout.scrollbar_thumb_margin = style.get_scrollbar_thumb_margin();
    layout.scrollbar_thumb_radius = style.get_scrollbar_thumb_radius();

    layout.scrollbar_size = Size::new(taffy_layout.scrollbar_size.width, taffy_layout.scrollbar_size.height);
    layout.computed_scrollbar_size = Size::new(taffy_layout.scroll_width(), taffy_layout.scroll_height());
    let state = &mut layout.scroll_state;

    if style.get_overflow()[1] != Overflow::Scroll {
        return;
    }

    let box_transformed = layout.computed_box_transformed;

    // Client Height = padding box height.
    let client_height = box_transformed.padding_rectangle().height;

    let mut content_height = layout.content_size.height;
    // Taffy is adding the top border and padding height to the content size.
    content_height -= box_transformed.border.top;
    content_height -= box_transformed.padding.top;

    // Content Size = overflowed content size + padding
    // Scroll Height = Content Size
    let scroll_height = (content_height + box_transformed.padding.bottom + box_transformed.padding.top).max(1.0);
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
        box_transformed.padding_rectangle().right() - scroll_track_width,
        box_transformed.padding_rectangle().top(),
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
}

pub struct HandleScrollLogicResult {
    pub request_apply_layout: bool,
    pub release_pointer_capture: bool,
    pub set_pointer_capture: bool,
}

pub(crate) fn handle_scroll_logic(element: &mut dyn ElementInternals, message: &EventKind, event: &mut Event) {
    let element_data = element.element_data_mut();
    let result = handle_scroll_logic_advance(&element_data.style, &mut element_data.layout, message, event);

    if result.request_apply_layout {
        request_apply_layout(element.element_data().layout.taffy_node_id());
    }

    if result.set_pointer_capture {
        element.set_pointer_capture(PointerId::new(1).unwrap())
    }

    if result.release_pointer_capture {
        element.release_pointer_capture(PointerId::new(1).unwrap());
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_scroll_logic_advance(
    style: &Style,
    layout: &mut Layout,
    message: &EventKind,
    event: &mut Event,
) -> HandleScrollLogicResult {
    let mut result = HandleScrollLogicResult {
        request_apply_layout: false,
        release_pointer_capture: false,
        set_pointer_capture: false,
    };

    if layout.is_scrollable_layout() && style.get_overflow()[1] == Overflow::Scroll {
        let state = &mut layout.scroll_state;
        match message {
            EventKind::PointerScroll(mouse_wheel) => {
                let delta = match mouse_wheel.delta {
                    ScrollDelta::LineDelta(_x, y) => y * style.get_font_size().max(12.0) * style.get_line_height(),
                    ScrollDelta::PixelDelta(physical) => physical.y as f32,
                    ScrollDelta::PageDelta(_x, y) => y,
                };
                let delta = -delta;
                // Todo: Scroll physics
                let max_scroll_y = layout.max_scroll_y;

                let current_scroll_y = state.scroll_y();
                state.set_scroll_y((current_scroll_y + delta).clamp(0.0, max_scroll_y));

                result.request_apply_layout = true;

                event.prevent_propagate();
                event.prevent_defaults();
            }
            EventKind::PointerButtonDown(pointer_button)
                if pointer_button.button == Some(ui_events::pointer::PointerButton::Primary) =>
            {
                // DEVICE(TOUCH): Handle scrolling within the content area on touch based input devices.
                if pointer_button.pointer.pointer_type == PointerType::Touch {
                    let container_rectangle = layout.computed_box_transformed.padding_rectangle();

                    let in_scroll_bar = layout
                        .computed_scroll_thumb
                        .contains(&pointer_button.state.logical_point());

                    if container_rectangle.contains(&pointer_button.state.logical_point()) && !in_scroll_bar {
                        state.scroll_click = Some(Point::new(
                            pointer_button.state.logical_point().x,
                            pointer_button.state.logical_point().y,
                        ));
                        event.prevent_propagate();
                        event.prevent_defaults();
                    }
                } else if layout
                    .computed_scroll_thumb
                    .contains(&pointer_button.state.logical_point())
                {
                    state.scroll_click = Some(Point::new(
                        pointer_button.state.logical_point().x,
                        pointer_button.state.logical_point().y,
                    ));

                    // FIXME: Turn pointer capture on with the correct device id.
                    //element.set_pointer_capture(PointerId::new(1).unwrap());

                    event.prevent_propagate();
                    event.prevent_defaults();

                    result.set_pointer_capture = true;
                } else if layout
                    .computed_scroll_track
                    .contains(&pointer_button.state.logical_point())
                {
                    let offset_y = pointer_button.state.position.y as f32 - layout.computed_scroll_track.y;

                    let percent = offset_y / layout.computed_scroll_track.height;
                    let scroll_y = percent * layout.max_scroll_y;

                    state.set_scroll_y(scroll_y.clamp(0.0, layout.max_scroll_y));

                    result.request_apply_layout = true;

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
                    let delta = (pointer_motion.current.position.y - click.y) as f32;

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
                    state.set_scroll_y((current_scroll_y + delta).clamp(0.0, max_scroll_y));
                    result.request_apply_layout = true;

                    state.scroll_click = Some(Point::new(click.x, pointer_motion.current.position.y));
                    event.prevent_propagate();
                    event.prevent_defaults();
                }
            }
            _ => {}
        }
    };

    result
}

pub fn draw_scrollbar(style: &Style, layout: &Layout, renderer: &mut RenderList, scale_factor: f64) {
    if !(layout.is_scrollable_layout() && style.get_overflow()[1] == Overflow::Scroll) {
        return;
    }

    let border_color = style.get_border_color();
    let scrollbar_color = style.get_scrollbar_color();
    let scrollbar_thumb_radius = style
        .get_scrollbar_thumb_radius()
        .map(|radii| Vec2::new(radii.0 as f64, radii.1 as f64));
    // let scrollbar_thumb_radius = self.element_data().current_style().
    let track_rect = layout.computed_scroll_track.scale(scale_factor);
    let thumb_rect = layout.computed_scroll_thumb.scale(scale_factor);

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
