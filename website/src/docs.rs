use retgui::elements::{Container, Element};
use retgui::style::{Display, FlexDirection, Overflow, Unit};
use retgui::{App, pct};

use crate::router::NavigateFn;

pub(crate) fn docs(app: &mut App, _navigate_fn: NavigateFn) -> Container {
    let content = Container::new(app)
        .edit(app)
        .display(Display::Flex)
        .width(pct(100))
        .margin(Unit::Px(0.0), Unit::Auto, Unit::Px(0.0), Unit::Auto)
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .finish();
    Container::new(app)
        .edit(app)
        .width(pct(100))
        .overflow(Overflow::Visible, Overflow::Scroll)
        .push(content)
        .finish()
}
