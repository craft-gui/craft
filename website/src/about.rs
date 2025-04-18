use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use craft::elements::{Container, ElementStyles, Text};
use craft::events::Event;
use craft::palette;
use craft::style::{AlignItems, Display, JustifyContent, Weight};

#[derive(Default)]
pub(crate) struct About {}

impl Component<WebsiteGlobalState> for About {
    type Props = ();

    fn view(
        _state: &Self,
        _global_state: &WebsiteGlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
    ) -> ComponentSpecification {
        Container::new()
            .display(Display::Flex)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .width("100%")
            .height("100%")
            .push(
                Text::new("Coming Soon...")
                    .font_size(48.0)
                    .font_weight(Weight::BOLD)
                    .component(),
            )
            .component()
    }

    fn update(
        _state: &mut Self,
        _global_state: &mut WebsiteGlobalState,
        _props: &Self::Props,
        _event: Event,
    ) -> UpdateResult {
        UpdateResult::default()
    }
}
