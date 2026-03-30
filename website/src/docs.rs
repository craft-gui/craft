use craft_retained::elements::{Container, Element};
use craft_retained::pct;
use craft_retained::style::{Display, FlexDirection, Overflow, Unit};

use crate::router::NavigateFn;

pub(crate) fn docs(_navigate_fn: NavigateFn) -> Container {
    Container::new()
        .width(pct(100))
        .overflow(Overflow::Visible, Overflow::Scroll)
        .push(
            Container::new()
                .display(Display::Flex)
                .width(pct(100))
                .margin(Unit::Px(0.0), Unit::Auto, Unit::Px(0.0), Unit::Auto)
                .flex_direction(FlexDirection::Column)
                .flex_grow(1.0),
        )
}
