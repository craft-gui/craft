use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentSpecification, Context};
use craft::elements::{Container, ElementStyles, Text};
use craft::style::{Display, FlexDirection, Weight};

#[derive(Default)]
pub(crate) struct StylingPage {

}

impl Component for StylingPage {
    type GlobalState = WebsiteGlobalState;
    type Props = ();
    type Message = ();

    fn view(_context: &mut Context<Self>) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .push(Text::new("Styling").font_size(32.0).margin("0px", "0px", "25px", "0px").font_weight(Weight::BOLD))
            .push(Text::new("Coming Soon!").font_size(16.0))
            .component()
    }
}