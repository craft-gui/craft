use crate::theme::NAVBAR_BACKGROUND_COLOR;
use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, Event};
use craft::elements::{Container, ElementStyles, Text};
use craft::events::Message;
use craft::style::{AlignItems, Display, JustifyContent, Weight};
use craft::{rgb, WindowContext};

#[derive(Default)]
pub(crate) struct Navbar {}

fn create_link(label: &str, route: &str) -> Text {
    Text::new(label)
        .id(format!("route_{}", route).as_str())
        .margin("0px", "12px", "0px", "0px") // Default Margin
        .font_size(16.0)
        .disable_selection()
        .color(rgb(220, 220, 220))
        .hovered()
        .color(rgb(255, 255, 255))
        .margin("0px", "12px", "0px", "0px") // Default Margin
        .font_size(16.0)
        .disable_selection()
        .normal()
}

impl Component for Navbar {
    type GlobalState = WebsiteGlobalState;
    type Props = ();
    type Message = ();

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext
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
                            .margin("0px", "24px", "0px", "0px")
                            .hovered().font_size(32.0)
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
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        message: &Message,
    ) {
        if message.clicked() {
            if let Some(current_target) = event.target.as_ref() {
                if let Some(id) = current_target.get_id() {
                    if id.starts_with("route_") {
                        let route = id.trim_start_matches("route_");
                        global_state.route = route.to_string();
                        event.prevent_propagate();
                    }
                }
            }
        }
    }
}
