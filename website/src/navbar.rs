use crate::theme::NAVBAR_BACKGROUND_COLOR;
use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use craft::elements::{Container, ElementStyles, Text};
use craft::events::Event;
use craft::style::{AlignItems, Display, JustifyContent, Weight};
use craft::Color;

#[derive(Default)]
pub(crate) struct Navbar {}

fn create_link(label: &str, route: &str) -> Text {
    Text::new(label)
        .id(format!("route_{}", route).as_str())
        .margin("0px", "12px", "0px", "0px") // Default Margin
        .font_size(16.0)
        .color(Color::from_rgb8(220, 220, 220)) // Light text for readability
}

impl Component<WebsiteGlobalState> for Navbar {
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
            .justify_content(JustifyContent::SpaceBetween)
            .align_items(AlignItems::Center)
            .width("100%")
            .padding("5px", "25px", "5px", "25px")
            .background(NAVBAR_BACKGROUND_COLOR)
            // Left
            .push(
                Container::new()
                    .display(Display::Flex)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .push(
                        create_link("Craft", "/")
                            .font_size(32.0)
                            .font_weight(Weight::BOLD)
                            .margin("0px", "24px", "0px", "0px"),
                    )
                    .push(create_link("Home", "/").margin("0px", "12px", "0px", "0px"))
                    .push(create_link("Examples", "/examples").margin("0px", "12px", "0px", "0px"))
                    .push(create_link("About", "/about").margin("0px", "12px", "0px", "0px")),
            )
            .component()
    }

    fn update(
        _state: &mut Self,
        global_state: &mut WebsiteGlobalState,
        _props: &Self::Props,
        event: Event,
    ) -> UpdateResult {
        if event.message.clicked() && event.current_target.is_some() {
            let current_target = event.current_target.as_ref().unwrap();
            if current_target.starts_with("route_") {
                global_state.route = current_target.replace("route_", "").to_string();
            }
        }

        UpdateResult::default()
    }
}
