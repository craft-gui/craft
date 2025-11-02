use crate::animations::animation::ActiveAnimation;
use crate::elements::element_states::ElementState;
use crate::elements::Element;
use crate::layout::layout_item::LayoutItem;
use crate::style::Style;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use kurbo::Point;
use taffy::Overflow;
use ui_events::keyboard::KeyboardEvent;
use ui_events::pointer::{PointerButtonEvent, PointerType, PointerUpdate};
use ui_events::ScrollDelta;
use craft_primitives::geometry::Rectangle;
use crate::elements::element_id::create_unique_element_id;
use crate::elements::scroll_state::ScrollState;
use crate::events::{CraftMessage, Event, KeyboardInputHandler, PointerEventHandler, PointerUpdateHandler};
//use crate::events::PointerEventHandler;

#[derive(Clone)]
pub struct ElementData {
    pub current_state: ElementState,

    /// The style of the element.
    pub style: Style,

    pub layout_item: LayoutItem,

    /// The style of the element when it is hovered.
    pub hover_style: Option<Style>,

    /// The style of the element when it is pressed.
    pub pressed_style: Option<Style>,

    /// The style of the element when it is disabled.
    pub disabled_style: Option<Style>,

    /// The style of the element when it is focused.
    pub focused_style: Option<Style>,

    /// The children of the element.
    pub children: SmallVec<[Rc<RefCell<dyn Element>>; 4]>,

    /// A user-defined id for the element.
    pub id: Option<SmolStr>,

    pub(crate) internal_id: u64,

    pub(crate) hovered: bool,
    pub(crate) active: bool,
    /// Whether this element should receive pointer events regardless of hit testing.
    /// Useful for scroll thumbs.
    pub(crate) pointer_capture: HashMap<i64, bool>,
    pub(crate) focused: bool,
    pub(crate) animations: Option<FxHashMap<SmolStr, ActiveAnimation>>,
    pub(crate) parent: Option<Weak<RefCell<dyn Element>>>,
    pub on_pointer_button_down: Vec<PointerEventHandler>,
    pub on_pointer_button_up: Vec<PointerEventHandler>,
    pub on_pointer_moved: Vec<PointerUpdateHandler>,
    pub on_keyboard_input: Vec<KeyboardInputHandler>,
    pub(crate) scroll_state: Option<ScrollState>,
}

impl ElementData {

    pub fn new(scrollable: bool) -> Self {
        let mut default = Self::default();

        if scrollable {
            default.scroll_state = Some(ScrollState::default());
        }

        default
    }

    pub(crate) fn finalize_scroll(&mut self) {
        if let Some(state) = &mut self.scroll_state {
            if self.style.overflow()[1] != Overflow::Scroll {
                return;
            }
            let box_transformed = self.layout_item.computed_box_transformed;

            // Client Height = padding box height.
            let client_height = box_transformed.padding_rectangle().height;

            let mut content_height = self.layout_item.content_size.height;
            // Taffy is adding the top border and padding height to the content size.
            content_height -= box_transformed.border.top;
            content_height -= box_transformed.padding.top;

            // Content Size = overflowed content size + padding
            // Scroll Height = Content Size
            let scroll_height = content_height + box_transformed.padding.bottom + box_transformed.padding.top;
            let scroll_track_width = self.layout_item.scrollbar_size.width;

            // The scroll track height is the height of the padding box.
            let scroll_track_height = client_height;

            let max_scroll_y = (scroll_height - client_height).max(0.0);
            self.layout_item.max_scroll_y = max_scroll_y;

            self.layout_item.computed_scroll_track = Rectangle::new(
                box_transformed.padding_rectangle().right() - scroll_track_width,
                box_transformed.padding_rectangle().top(),
                scroll_track_width,
                scroll_track_height,
            );

            let visible_y = client_height / scroll_height;
            let scroll_thumb_height = scroll_track_height * visible_y;
            let remaining_height = scroll_track_height - scroll_thumb_height;
            let scroll_thumb_offset =
                if max_scroll_y != 0.0 { state.scroll_y / max_scroll_y * remaining_height } else { 0.0 };

            let thumb_margin = self.style.scrollbar_thumb_margin();
            let scroll_thumb_width = scroll_track_width - (thumb_margin.left + thumb_margin.right);
            let scroll_thumb_height = scroll_thumb_height - (thumb_margin.top + thumb_margin.bottom);
            self.layout_item.computed_scroll_thumb = self.layout_item.computed_scroll_track;
            self.layout_item.computed_scroll_thumb.x += thumb_margin.left;
            self.layout_item.computed_scroll_thumb.y += scroll_thumb_offset + thumb_margin.top;
            self.layout_item.computed_scroll_thumb.width = scroll_thumb_width;
            self.layout_item.computed_scroll_thumb.height = scroll_thumb_height;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn on_event(
        &mut self,
        message: &CraftMessage,
        event: &mut Event,
    ) {
        if self.is_scrollable() {
            if let Some(state) = &mut self.scroll_state {
                match message {
                    CraftMessage::PointerScroll(mouse_wheel) => {
                        let delta = match mouse_wheel.delta {
                            ScrollDelta::LineDelta(_x, y) => y * self.style.font_size().max(12.0) * self.style.line_height(),
                            ScrollDelta::PixelDelta(physical) => physical.y as f32,
                            ScrollDelta::PageDelta(_x, y) => y,
                        };
                        let delta = -delta;
                        // Todo: Scroll physics
                        let max_scroll_y = self.layout_item.max_scroll_y;

                        state.scroll_y = (state.scroll_y + delta).clamp(0.0, max_scroll_y);

                        event.prevent_propagate();
                        event.prevent_defaults();
                    }
                    CraftMessage::PointerButtonDown(pointer_button) => {
                        if pointer_button.button == Some(ui_events::pointer::PointerButton::Primary) {
                            // DEVICE(TOUCH): Handle scrolling within the content area on touch based input devices.
                            if pointer_button.pointer.pointer_type == PointerType::Touch {
                                let container_rectangle = self.layout_item.computed_box_transformed.padding_rectangle();

                                let in_scroll_bar =
                                    self.layout_item.computed_scroll_thumb.contains(&pointer_button.state.logical_point());

                                if container_rectangle.contains(&pointer_button.state.logical_point()) && !in_scroll_bar {
                                    state.scroll_click =
                                        Some(Point::new(pointer_button.state.position.x, pointer_button.state.logical_point().y));
                                    event.prevent_propagate();
                                    event.prevent_defaults();
                                }
                            } else if self.layout_item.computed_scroll_thumb.contains(&pointer_button.state.logical_point()) {
                                state.scroll_click =
                                    Some(Point::new(pointer_button.state.logical_point().x, pointer_button.state.logical_point().y));
                                // FIXME: Turn pointer capture on with the correct device id.
                                self.pointer_capture.insert(DUMMY_DEVICE_ID, true);

                                event.prevent_propagate();
                                event.prevent_defaults();
                            } else if self.layout_item.computed_scroll_track.contains(&pointer_button.state.logical_point()) {
                                let offset_y =
                                    pointer_button.state.position.y as f32 - self.layout_item.computed_scroll_track.y;

                                let percent = offset_y / self.layout_item.computed_scroll_track.height;
                                let scroll_y = percent * self.layout_item.max_scroll_y;

                                state.scroll_y = scroll_y.clamp(0.0, self.layout_item.max_scroll_y);

                                event.prevent_propagate();
                                event.prevent_defaults();
                            }
                        }
                    }
                    CraftMessage::PointerButtonUp(_pointer_button) => {
                        if state.scroll_click.is_some() {
                            state.scroll_click = None;
                            // FIXME: Turn pointer capture off with the correct device id.
                            self.pointer_capture.insert(DUMMY_DEVICE_ID, false);
                            event.prevent_propagate();
                            event.prevent_defaults();
                        }
                    }
                    CraftMessage::PointerMovedEvent(pointer_motion) => {
                        if let Some(click) = &state.scroll_click {
                            // Todo: Translate scroll wheel pixel to scroll position for diff.
                            let delta = (pointer_motion.current.position.y - click.y) as f32;

                            let max_scroll_y = self.layout_item.max_scroll_y;

                            let click_y_offset = self.layout_item.computed_scroll_track.height - self.layout_item.computed_scroll_thumb.height;
                            if click_y_offset <= 0.0 {
                                return;
                            }
                            let mut delta = max_scroll_y * (delta / (click_y_offset));

                            // DEVICE(TOUCH): Reverse the direction on touch based input devices.
                            if pointer_motion.pointer.pointer_type == PointerType::Touch {
                                delta = -delta;
                            }

                            state.scroll_y = (state.scroll_y + delta).clamp(0.0, max_scroll_y);
                            state.scroll_click = Some(Point::new(click.x, pointer_motion.current.position.y));
                            event.prevent_propagate();
                            event.prevent_defaults();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Default for ElementData {
    fn default() -> Self {
        Self {
            current_state: Default::default(),
            style: Default::default(),
            layout_item: Default::default(),
            hover_style: None,
            pressed_style: None,
            disabled_style: None,
            focused_style: None,
            children: Default::default(),
            id: None,
            internal_id: create_unique_element_id(),
            hovered: false,
            active: false,
            pointer_capture: Default::default(),
            focused: false,
            animations: None,
            parent: None,
            on_pointer_button_down: vec![],
            on_pointer_button_up: vec![],
            on_pointer_moved: vec![],
            on_keyboard_input: vec![],
            scroll_state: None,
        }
    }
}

impl ElementData {
    pub fn is_scrollable(&self) -> bool {
        self.style.overflow()[1] == taffy::Overflow::Scroll
    }

    pub fn current_style(&self) -> &Style {
        &self.style
    }

    pub fn current_style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    pub fn current_style_mut_no_fallback<'a>(&self) -> Option<&'a mut Style> {
        None
    }
}

// HACK: Remove this and all usages when pointer capture per device works.
pub(crate) const DUMMY_DEVICE_ID: i64 = -1;
