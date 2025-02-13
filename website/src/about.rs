use crate::WebsiteGlobalState;
use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::{Container, ElementStyles, Text};
use oku::events::Event;
use oku::renderer::color::palette;
use oku::style::{AlignItems, Display, JustifyContent, Weight};

#[derive(Default)]
pub(crate) struct About {
}

impl Component<WebsiteGlobalState> for About {
    type Props = ();

    fn view(state: &Self, global_state: &WebsiteGlobalState, props: &Self::Props, children: Vec<ComponentSpecification>) -> ComponentSpecification {
       Container::new()
            .display(Display::Flex)
           .align_items(AlignItems::Center)
           .justify_content(JustifyContent::Center)
            .width("100%")
            .height("100%")
            .push(
                Text::new("Coming Soon...").font_size(48.0).font_weight(Weight::BOLD).color(palette::css::WHITE).component()
            ).component()
    }

    fn update(state: &mut Self, _global_state: &mut WebsiteGlobalState, _props: &Self::Props, event: Event) -> UpdateResult {
        UpdateResult::default()
    }
}