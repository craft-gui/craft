use crate::components::{Component, ComponentSpecification, Context};
use crate::elements::Text;

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

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        context.children().first().unwrap_or(&Text::new("Invalid Link").component()).clone()
    }

    fn update(context: &mut Context<Self>) {
        if context.message().clicked() {
            open(context.props().href.as_str())
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
