use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, Context, Event};
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

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        if let Some(markdown) = &context.state().markdown {
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

    fn update(context: &mut Context<Self>) {
        if let Message::CraftMessage(CraftMessage::LinkClicked(link)) = context.message() {
            craft::components::open(link);
            context.event_mut().prevent_propagate();
        }

        if let Message::CraftMessage(CraftMessage::Initialized) = context.message() {
            let installation = craft::markdown::render_markdown(include_str!("markdown/installation.md"));
            context.state_mut().markdown = Some(installation);
        }
    }
}