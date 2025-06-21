use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, Context, Event};
use craft::elements::Text;
use craft::events::Message;
use craft::WindowContext;

#[derive(Default)]
pub(crate) struct WebLink;

#[derive(Default)]
pub(crate) struct WebLinkProps {
    pub(crate) href: String,
}

impl Component for WebLink {
    type GlobalState = WebsiteGlobalState;
    type Props = WebLinkProps;
    type Message = ();

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        context.children().first().unwrap_or(&Text::new("Invalid Link").component()).clone()
    }

    fn update(context: &mut Context<Self>) {
        if context.message().clicked() {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url(context.props().href.as_str());
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                open::that(context.props().href.as_str()).unwrap();
            }
        }
    }
}
