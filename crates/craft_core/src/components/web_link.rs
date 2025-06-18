use crate::components::{Component, ComponentId, ComponentSpecification, Event};
use crate::elements::Text;
use crate::events::Message;
use crate::WindowContext;

#[derive(Default)]
pub struct WebLink;

#[derive(Default)]
pub struct WebLinkProps {
    pub(crate) href: String,
}

impl Component for WebLink {
    type GlobalState = ();
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
            open(props.href.as_str())
        }
    }
}

pub fn open(link: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            let _ = win.open_with_url(link);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        open::that(link).unwrap();
    }
}
