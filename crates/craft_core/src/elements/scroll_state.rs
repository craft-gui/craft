use crate::components::Event;
use crate::elements::base_element_state::{BaseElementState, DUMMY_DEVICE_ID};
use crate::elements::element_data::ElementData;
use crate::events::CraftMessage;
use crate::geometry::Point;
use crate::geometry::Rectangle;
use taffy::Overflow;
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
                        ScrollDelta::LineDelta(_x, y) => y * element.style.font_size().max(12.0) * 1.2,
                        ScrollDelta::PixelDelta(_x, y) => y as f32,
                        ScrollDelta::PageDelta(_x, y) => y,
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

                            let in_scroll_bar =
                                element.layout_item.computed_scroll_thumb.contains(&pointer_button.state.position);

                            if container_rectangle.contains(&pointer_button.state.position) && !in_scroll_bar {
                                self.scroll_click =
                                    Some(Point::new(pointer_button.state.position.x, pointer_button.state.position.y));
                                event.prevent_propagate();
                                event.prevent_defaults();
                                return;
                            }
                        } else if element.layout_item.computed_scroll_thumb.contains(&pointer_button.state.position) {
                            self.scroll_click =
                                Some(Point::new(pointer_button.state.position.x, pointer_button.state.position.y));
                            // FIXME: Turn pointer capture on with the correct device id.
                            base_state.pointer_capture.insert(DUMMY_DEVICE_ID, true);

                            event.prevent_propagate();
                            event.prevent_defaults();
                        } else if element.layout_item.computed_scroll_track.contains(&pointer_button.state.position) {
                            let offset_y =
                                pointer_button.state.position.y as f32 - element.layout_item.computed_scroll_track.y;

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

                        let click_y_offset = element.layout_item.computed_scroll_track.height - element.layout_item.computed_scroll_thumb.height;
                        if click_y_offset <= 0.0 {
                            return;
                        }
                        let mut delta = max_scroll_y * (delta / (click_y_offset));

                        // DEVICE(TOUCH): Reverse the direction on touch based input devices.
                        if pointer_motion.pointer.pointer_type == PointerType::Touch {
                            delta = -delta;
                        }

                        self.scroll_y = (self.scroll_y + delta).clamp(0.0, max_scroll_y);
                        self.scroll_click = Some(Point::new(click.x, pointer_motion.current.position.y));
                        event.prevent_propagate();
                        event.prevent_defaults();
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn finalize_layout(&mut self, element_data: &mut ElementData) {
        if element_data.style.overflow()[1] != Overflow::Scroll {
            return;
        }
        let box_transformed = element_data.layout_item.computed_box_transformed;

        // Client Height = padding box height.
        let client_height = box_transformed.padding_rectangle().height;

        let mut content_height = element_data.layout_item.content_size.height;
        // Taffy is adding the top border and padding height to the content size.
        content_height -= box_transformed.border.top;
        content_height -= box_transformed.padding.top;

        // Content Size = overflowed content size + padding
        // Scroll Height = Content Size
        let scroll_height = content_height + box_transformed.padding.bottom + box_transformed.padding.top;
        let scroll_track_width = element_data.layout_item.scrollbar_size.width;

        // The scroll track height is the height of the padding box.
        let scroll_track_height = client_height;

        let max_scroll_y = (scroll_height - client_height).max(0.0);
        element_data.layout_item.max_scroll_y = max_scroll_y;

        element_data.layout_item.computed_scroll_track = Rectangle::new(
            box_transformed.padding_rectangle().right() - scroll_track_width,
            box_transformed.padding_rectangle().top(),
            scroll_track_width,
            scroll_track_height,
        );

        let visible_y = client_height / scroll_height;
        let scroll_thumb_height = scroll_track_height * visible_y;
        let remaining_height = scroll_track_height - scroll_thumb_height;
        let scroll_thumb_offset =
            if max_scroll_y != 0.0 { self.scroll_y / max_scroll_y * remaining_height } else { 0.0 };

        let thumb_margin = element_data.style.scrollbar_thumb_margin();
        let scroll_thumb_width = scroll_track_width - (thumb_margin.left + thumb_margin.right);
        let scroll_thumb_height = scroll_thumb_height - (thumb_margin.top + thumb_margin.bottom);
        element_data.layout_item.computed_scroll_thumb = element_data.layout_item.computed_scroll_track;
        element_data.layout_item.computed_scroll_thumb.x += thumb_margin.left;
        element_data.layout_item.computed_scroll_thumb.y += scroll_thumb_offset + thumb_margin.top;
        element_data.layout_item.computed_scroll_thumb.width = scroll_thumb_width;
        element_data.layout_item.computed_scroll_thumb.height = scroll_thumb_height;
    }
}
