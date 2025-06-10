use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification};
use craft::elements::{Container, ElementStyles, Text};
use craft::style::{Display, FlexDirection, Weight};
use craft::WindowContext;

#[derive(Default)]
pub(crate) struct InstallationPage {

}

impl Component for InstallationPage {
    type GlobalState = WebsiteGlobalState;
    type Props = ();
    type Message = ();

    fn view(&self, _global_state: &Self::GlobalState, _props: &Self::Props, _children: Vec<ComponentSpecification>, _id: ComponentId, _window: &WindowContext) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .push(Text::new("Installation").font_size(32.0).margin("0px", "0px", "25px", "0px").font_weight(Weight::BOLD))
            .push(Text::new("Coming Soon!").font_size(16.0))
            .component()
    }
}