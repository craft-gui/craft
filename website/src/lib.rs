mod about;
mod examples;
mod index;
mod navbar;
mod theme;

use crate::about::About;
use crate::examples::Examples;
use crate::index::index_page;
use crate::navbar::Navbar;
use crate::theme::BODY_BACKGROUND_COLOR;
use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::Container;
use oku::elements::ElementStyles;
use oku::events::Event;
use oku::style::Display;
use oku::style::FlexDirection;
use oku::{oku_main_with_options, OkuOptions, RendererType};

pub(crate) struct WebsiteGlobalState {
    /// The current route that we are viewing.
    pub(crate) route: String,
}

impl Default for WebsiteGlobalState {
    fn default() -> Self {
        WebsiteGlobalState {
            route: "/".to_string(),
        }
    }
}

#[derive(Default)]
pub(crate) struct Website {}

impl Component<WebsiteGlobalState> for Website {
    type Props = ();

    fn view(
        state: &Self,
        global_state: &WebsiteGlobalState,
        props: &Self::Props,
        children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        let wrapper = Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width("100%")
            .height("100%")
            .push(Navbar::component())
            .background(BODY_BACKGROUND_COLOR);

        match global_state.route.as_str() {
            "/examples" => wrapper.push(Examples::component().key("examples")).component(),
            "/about" => wrapper.push(About::component().key("about")).component(),
            "/" | _ => wrapper.push(index_page().key("index")).component(),
        }
    }

    fn update(
        _state: &mut Self,
        global_state: &mut WebsiteGlobalState,
        _props: &Self::Props,
        _message: Event,
    ) -> UpdateResult {
        UpdateResult::default()
    }
}

fn main() {
    oku_main_with_options(
        Website::component(),
        Box::new(WebsiteGlobalState::default()),
        Some(OkuOptions {
            renderer: RendererType::default(),
            ..Default::default()
        }),
    )
}
