use crate::elements::Element;
use crate::events::{CraftMessage, Event};
use kurbo::Point;
use ui_events::pointer::{PointerId, PointerType};
use ui_events::ScrollDelta;

#[allow(clippy::too_many_arguments)]
pub(crate) fn on_scroll_events(element: &mut dyn Element, message: &CraftMessage, event: &mut Event) {
    let element_data = element.element_data_mut();

    if element_data.is_scrollable()
        && let Some(state) = &mut element_data.scroll_state
    {
        match message {
            CraftMessage::PointerScroll(mouse_wheel) => {
                let delta = match mouse_wheel.delta {
                    ScrollDelta::LineDelta(_x, y) => {
                        y * element_data.style.font_size().max(12.0) * element_data.style.line_height()
                    }
                    ScrollDelta::PixelDelta(physical) => physical.y as f32,
                    ScrollDelta::PageDelta(_x, y) => y,
                };
                let delta = -delta;
                // Todo: Scroll physics
                let max_scroll_y = element_data.layout_item.max_scroll_y;

                let current_scroll_y = state.scroll_y();
                state.set_scroll_y((current_scroll_y + delta).clamp(0.0, max_scroll_y));

                event.prevent_propagate();
                event.prevent_defaults();
            }
            CraftMessage::PointerButtonDown(pointer_button) => {
                if pointer_button.button == Some(ui_events::pointer::PointerButton::Primary) {
                    // DEVICE(TOUCH): Handle scrolling within the content area on touch based input devices.
                    if pointer_button.pointer.pointer_type == PointerType::Touch {
                        let container_rectangle = element_data.layout_item.computed_box_transformed.padding_rectangle();

                        let in_scroll_bar = element_data
                            .layout_item
                            .computed_scroll_thumb
                            .contains(&pointer_button.state.logical_point());

                        if container_rectangle.contains(&pointer_button.state.logical_point()) && !in_scroll_bar {
                            state.scroll_click = Some(Point::new(
                                pointer_button.state.position.x,
                                pointer_button.state.logical_point().y,
                            ));
                            event.prevent_propagate();
                            event.prevent_defaults();
                        }
                    } else if element_data
                        .layout_item
                        .computed_scroll_thumb
                        .contains(&pointer_button.state.logical_point())
                    {
                        state.scroll_click = Some(Point::new(
                            pointer_button.state.logical_point().x,
                            pointer_button.state.logical_point().y,
                        ));

                        // FIXME: Turn pointer capture on with the correct device id.
                        element.set_pointer_capture(PointerId::new(1).unwrap());

                        event.prevent_propagate();
                        event.prevent_defaults();
                    } else if element_data
                        .layout_item
                        .computed_scroll_track
                        .contains(&pointer_button.state.logical_point())
                    {
                        let offset_y =
                            pointer_button.state.position.y as f32 - element_data.layout_item.computed_scroll_track.y;

                        let percent = offset_y / element_data.layout_item.computed_scroll_track.height;
                        let scroll_y = percent * element_data.layout_item.max_scroll_y;

                        state.set_scroll_y(scroll_y.clamp(0.0, element_data.layout_item.max_scroll_y));

                        event.prevent_propagate();
                        event.prevent_defaults();
                    }
                }
            }
            CraftMessage::PointerButtonUp(_pointer_button) => {
                if state.scroll_click.is_some() {
                    state.scroll_click = None;
                    // FIXME: Turn pointer capture off with the correct device id.
                    element.release_pointer_capture(PointerId::new(1).unwrap());
                    event.prevent_propagate();
                    event.prevent_defaults();
                }
            }
            CraftMessage::PointerMovedEvent(pointer_motion) => {
                if let Some(click) = state.scroll_click {
                    // Todo: Translate scroll wheel pixel to scroll position for diff.
                    let delta = (pointer_motion.current.position.y - click.y) as f32;

                    let max_scroll_y = element_data.layout_item.max_scroll_y;

                    let click_y_offset = element_data.layout_item.computed_scroll_track.height
                        - element_data.layout_item.computed_scroll_thumb.height;
                    if click_y_offset <= 0.0 {
                        return;
                    }
                    let mut delta = max_scroll_y * (delta / (click_y_offset));

                    // DEVICE(TOUCH): Reverse the direction on touch based input devices.
                    if pointer_motion.pointer.pointer_type == PointerType::Touch {
                        delta = -delta;
                    }

                    let current_scroll_y = state.scroll_y();
                    state.set_scroll_y((current_scroll_y + delta).clamp(0.0, max_scroll_y));
                    state.scroll_click = Some(Point::new(click.x, pointer_motion.current.position.y));
                    event.prevent_propagate();
                    event.prevent_defaults();
                }
            }
            _ => {}
        }
    }
}
