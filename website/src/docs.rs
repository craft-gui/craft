use retgui::elements::{Container, Element, Elements};
use retgui::pct;
use retgui::style::{Display, FlexDirection, Overflow, Unit};

use crate::router::NavigateFn;

pub(crate) fn docs(elements: &mut Elements, _navigate_fn: NavigateFn) -> Container {
    let content = Container::new(elements)
        .edit(elements)
        .display(Display::Flex)
        .width(pct(100))
        .margin(Unit::Px(0.0), Unit::Auto, Unit::Px(0.0), Unit::Auto)
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .finish();
    Container::new(elements)
        .edit(elements)
        .width(pct(100))
        .overflow(Overflow::Visible, Overflow::Scroll)
        .push(content)
        .finish()
}
