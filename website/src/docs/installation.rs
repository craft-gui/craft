use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, Event};
use craft::elements::{Container, ElementStyles, Text};
use craft::events::{CraftMessage, Message};
use craft::style::{Display, FlexDirection};
use craft::WindowContext;

#[derive(Default)]
pub(crate) struct InstallationPage {
    markdown: Option<ComponentSpecification>
}

impl Component for InstallationPage {
    type GlobalState = WebsiteGlobalState;
    type Props = ();
    type Message = ();

    fn view(&self, _global_state: &Self::GlobalState, _props: &Self::Props, _children: Vec<ComponentSpecification>, _id: ComponentId, _window: &WindowContext) -> ComponentSpecification {
        if let Some(markdown) = &self.markdown {
            Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(markdown.clone())
                .component()
        } else {
            Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .push(Text::new("Loading installation instructions..."))
                .component()
        }
    }

    fn update(&mut self, _global_state: &mut Self::GlobalState, _props: &Self::Props, event: &mut Event, message: &Message) {
        if let Message::CraftMessage(CraftMessage::LinkClicked(link)) = message {
            craft::components::open(link);
            event.prevent_propagate();
        }

        if let Message::CraftMessage(CraftMessage::Initialized) = message {
            let installation = craft::markdown::render_markdown(include_str!("markdown/installation.md"));
            self.markdown = Some(installation);
        }
    }
}