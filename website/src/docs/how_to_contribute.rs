use crate::docs::docs_template;
use crate::docs::markdown_viewer::{MarkdownViewer, MarkdownViewerProps};
use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentSpecification, Context, Props};

#[derive(Default)]
pub(crate) struct HowToContributePage {

}

impl Component for HowToContributePage {
    type GlobalState = WebsiteGlobalState;
    type Props = ();
    type Message = ();

    fn view(_context: &mut Context<Self>) -> ComponentSpecification {
        docs_template()
            .push(MarkdownViewer::component().props(Props::new(MarkdownViewerProps {
                markdown_text: include_str!("markdown/how_to_contribute.md").to_string()
            })))
            .component()
    }
}