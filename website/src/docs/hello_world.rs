use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentSpecification, Context, Props};
use craft::elements::{Container, ElementStyles, Text};
use craft::style::{Display, FlexDirection, Weight};
use crate::docs::docs_template;
use crate::docs::hello_world::counter::Counter;
use crate::docs::markdown_viewer::{MarkdownViewer, MarkdownViewerProps};

#[derive(Default)]
pub(crate) struct HelloWorldPage {
    
}

#[path = "../../../examples/counter/main.rs"]
mod counter;

impl Component for HelloWorldPage {
    type GlobalState = WebsiteGlobalState;
    type Props = ();
    type Message = ();

    fn view(_context: &mut Context<Self>) -> ComponentSpecification {
        docs_template()
            .push(MarkdownViewer::component().props(Props::new(MarkdownViewerProps {
                markdown_text: include_str!("markdown/hello_world/intro.md").to_string()
            })))
            .push(Counter::component())
            .component()
    }
}