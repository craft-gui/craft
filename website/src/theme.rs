use craft::elements::{Container, ElementStyles};
use craft::style::Unit;
use craft::Color;

pub(crate) const BODY_BACKGROUND_COLOR: Color = Color::from_rgb8(255, 255, 255);

pub(crate) const NAVBAR_BACKGROUND_COLOR: Color = Color::from_rgb8(255, 255, 255);
pub(crate) const NAVBAR_TEXT_COLOR: Color = Color::from_rgb8(50, 50, 50);
pub(crate) const NAVBAR_TEXT_HOVERED_COLOR: Color = Color::from_rgb8(0, 0, 0);

pub(crate) const ACTIVE_LINK_COLOR: Color = Color::from_rgb8(42, 108, 200);
pub(crate) const DEFAULT_LINK_COLOR: Color = Color::from_rgb8(102, 102, 102);


pub(crate) const WRAPPER_MAX_WIDTH: Unit = Unit::Px(1300.0);
pub(crate) const WRAPPER_MARGIN_LEFT: Unit = Unit::Auto;
pub(crate) const WRAPPER_MARGIN_RIGHT: Unit = Unit::Auto;
pub(crate) const WRAPPER_PADDING_LEFT: Unit = Unit::Px(20.0);
pub(crate) const WRAPPER_PADDING_RIGHT: Unit = Unit::Px(20.0);


pub(crate) const MOBILE_MEDIA_QUERY_WIDTH: f32 = 850.0;

pub(crate) fn wrapper() -> Container {
    Container::new()
        .margin(Unit::Px(0.0), WRAPPER_MARGIN_RIGHT, Unit::Px(0.0), WRAPPER_MARGIN_LEFT)
        .padding(Unit::Px(0.0), WRAPPER_PADDING_RIGHT, Unit::Px(0.0), WRAPPER_PADDING_LEFT)
        .width("100%")
        .max_width(WRAPPER_MAX_WIDTH)
}