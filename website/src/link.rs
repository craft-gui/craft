use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentSpecification, Context};
use craft::elements::Text;

#[derive(Default)]
pub(crate) struct Link;

#[derive(Default)]
pub(crate) struct LinkProps {
    pub(crate) href: String,
}

impl Component for Link {
    type GlobalState = WebsiteGlobalState;
    type Props = LinkProps;
    type Message = ();

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        context.children().first().unwrap_or(&Text::new("Invalid Link").component()).clone()
    }

    fn update(context: &mut Context<Self>) {
        if context.message().clicked() {
            let href = context.props().href.clone();
            context.global_state_mut().set_route(href.as_str());
        }
    }
}
