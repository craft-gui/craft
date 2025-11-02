use crate::elements::element_data::{ElementData, DUMMY_DEVICE_ID};
use crate::events::{CraftMessage, Event};
use craft_primitives::geometry::Point;
use craft_primitives::geometry::Rectangle;
use taffy::Overflow;
use ui_events::pointer::PointerType;
use ui_events::ScrollDelta;

#[derive(Debug, Clone, Default, Copy)]
pub struct ScrollState {
    pub(crate) scroll_y: f32,
    pub(crate) scroll_click: Option<Point>,
}