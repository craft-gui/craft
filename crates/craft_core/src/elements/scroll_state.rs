use crate::components::Event;
use crate::elements::base_element_state::{BaseElementState, DUMMY_DEVICE_ID};
use crate::elements::element_data::ElementData;
use crate::events::CraftMessage;
use crate::geometry::Point;
use ui_events::pointer::PointerType;
use ui_events::ScrollDelta;

#[derive(Debug, Clone, Default, Copy)]
pub struct ScrollState {
    pub(crate) scroll_y: f32,
    pub(crate) scroll_click: Option<Point>,
}

impl ScrollState {
    pub(crate) fn on_event(
        &mut self,
        message: &CraftMessage,
        element: &ElementData,
        base_state: &mut BaseElementState,
        event: &mut Event,
    ) {
        if element.is_scrollable() {
            match message {
                CraftMessage::PointerScroll(mouse_wheel) => {
                    let delta = match mouse_wheel.delta {
                        ScrollDelta::LineDelta(_x, y) => {
                            y * element.style.font_size().max(12.0) * 1.2
                        },
                        ScrollDelta::PixelDelta(_x, y) => {
                            y as f32
                        },
                        ScrollDelta::PageDelta(_x, y) => {
                            y
                        }
                    };
                    let delta = -delta;
                    // Todo: Scroll physics
                    let max_scroll_y = element.layout_item.max_scroll_y;

                    self.scroll_y = (self.scroll_y + delta).clamp(0.0, max_scroll_y);

                    event.prevent_propagate();
                    event.prevent_defaults();
                }
                CraftMessage::PointerButtonDown(pointer_button) => {
                    if pointer_button.is_primary() {
                        // DEVICE(TOUCH): Handle scrolling within the content area on touch based input devices.
                        if pointer_button.pointer.pointer_type == PointerType::Touch {
                            let container_rectangle = element.layout_item.computed_box_transformed.padding_rectangle();

                            let in_scroll_bar = element.layout_item.computed_scroll_thumb.contains(&pointer_button.state.position);

                            if container_rectangle.contains(&pointer_button.state.position) && !in_scroll_bar {
                                self.scroll_click = Some(Point::new(pointer_button.state.position.x, pointer_button.state.position.y));
                                event.prevent_propagate();
                                event.prevent_defaults();
                                return;
                            }
                        } else if element.layout_item.computed_scroll_thumb.contains(&pointer_button.state.position) {
                            self.scroll_click = Some(Point::new(pointer_button.state.position.x, pointer_button.state.position.y));
                            // FIXME: Turn pointer capture on with the correct device id.
                            base_state.pointer_capture.insert(DUMMY_DEVICE_ID, true);

                            event.prevent_propagate();
                            event.prevent_defaults();
                        } else if element.layout_item.computed_scroll_track.contains(&pointer_button.state.position) {
                            let offset_y = pointer_button.state.position.y as f32 - element.layout_item.computed_scroll_track.y;

                            let percent = offset_y / element.layout_item.computed_scroll_track.height;
                            let scroll_y = percent * element.layout_item.max_scroll_y;

                            self.scroll_y = scroll_y.clamp(0.0, element.layout_item.max_scroll_y);

                            event.prevent_propagate();
                            event.prevent_defaults();
                        }
                    }
                }
                CraftMessage::PointerButtonUp(_pointer_button) => {
                    if self.scroll_click.is_some() {
                        self.scroll_click = None;
                        // FIXME: Turn pointer capture off with the correct device id.
                        base_state.pointer_capture.insert(DUMMY_DEVICE_ID, false);
                        event.prevent_propagate();
                        event.prevent_defaults();
                    }
                }
                CraftMessage::PointerMovedEvent(pointer_motion) => {
                    if let Some(click) = self.scroll_click {
                        // Todo: Translate scroll wheel pixel to scroll position for diff.
                        let delta = (pointer_motion.current.position.y - click.y) as f32;

                        let max_scroll_y = element.layout_item.max_scroll_y;

                        let mut delta = max_scroll_y
                            * (delta / (element.layout_item.computed_scroll_track.height - element.layout_item.computed_scroll_thumb.height));

                        // DEVICE(TOUCH): Reverse the direction on touch based input devices.
                        if pointer_motion.pointer.pointer_type == PointerType::Touch {
                            delta = -delta;
                        }

                        self.scroll_y = (self.scroll_y + delta).clamp(0.0, max_scroll_y);
                        self.scroll_click = Some(Point::new(click.x, pointer_motion.current.position.y));
                        event.prevent_propagate();
                        event.prevent_defaults();
                    }
                },
                _ => {  }
            }
        }
    }
}
