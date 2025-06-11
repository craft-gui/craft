use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, Event};
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

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        _props: &Self::Props,
        children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext,
    ) -> ComponentSpecification {
        children.first().unwrap_or(&Text::new("Invalid Link").component()).clone()
    }

    fn update(
        &mut self,
        _global_state: &mut Self::GlobalState,
        props: &Self::Props,
        _event: &mut Event,
        message: &Message,
    ) {
        if message.clicked() {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url(props.href.as_str());
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                open::that(props.href.as_str()).unwrap();
            }
        }
    }
}
