use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification};
use craft::elements::{Container, ElementStyles, Text};
use craft::style::{AlignItems, Display, JustifyContent, Weight};
use craft::WindowContext;

#[derive(Default)]
pub(crate) struct Docs {}

impl Component for Docs {
    type Props = ();
    type GlobalState = WebsiteGlobalState;
    type Message = ();

    fn view(
        &self,
        _global_state: &WebsiteGlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext,
    ) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .width("100%")
            .height("100%")
            .push(Text::new("Coming Soon").font_size(48.0).font_weight(Weight::BOLD).component())
            .component()
    }
}
