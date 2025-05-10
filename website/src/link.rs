use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use craft::elements::Text;
use craft::events::Event;
use craft::WindowContext;

#[derive(Default)]
pub(crate) struct Link;

#[derive(Default)]
pub(crate) struct LinkProps {
    pub(crate) href: String,
}

impl Component<WebsiteGlobalState> for Link {
    type Props = LinkProps;

    fn view(
        _state: &Self,
        _global_state: &WebsiteGlobalState,
        _props: &Self::Props,
        children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window_context: &WindowContext
    ) -> ComponentSpecification {
        children.get(0).unwrap_or(&Text::new("Invalid Link").component()).clone()
    }

    fn update(
        _state: &mut Self,
        _global_state: &mut WebsiteGlobalState,
        props: &Self::Props,
        event: Event,
        _window_context: &mut WindowContext
    ) -> UpdateResult {
        if event.message.clicked() {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url(props.href.as_str());
                }
            }

            #[cfg(not(target_arch = "wasm32"))] {
                open::that(props.href.as_str()).unwrap();
            }
        }
        UpdateResult::default()
    }
}