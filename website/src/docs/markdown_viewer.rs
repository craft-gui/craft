use crate::docs::docs_template;
use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentSpecification, Context};
use craft::elements::Text;
use craft::events::{CraftMessage, Message};

#[derive(Default)]
pub(crate) struct MarkdownViewer {
    markdown: Option<ComponentSpecification>
}

#[derive(Default)]
pub(crate) struct MarkdownViewerProps {
    pub(crate) markdown_text: String,
}

impl Component for MarkdownViewer {
    type GlobalState = WebsiteGlobalState;
    type Props = MarkdownViewerProps;
    type Message = ();

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        if let Some(markdown) = &context.state().markdown {
            docs_template()
                .push(markdown.clone())
                .component()
        } else {
            docs_template()
                .push(Text::new("Loading..."))
                .component()
        }
    }

    fn update(context: &mut Context<Self>) {
        if let Message::CraftMessage(CraftMessage::LinkClicked(link)) = context.message() {
            craft::components::open(link);
            context.event_mut().prevent_propagate();
        }

        if let Message::CraftMessage(CraftMessage::Initialized) = context.message() {
            let installation = craft::markdown::render_markdown(context.props().markdown_text.as_str());
            context.state_mut().markdown = Some(installation);
        }
    }
}