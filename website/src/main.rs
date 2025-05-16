mod about;
mod examples;
mod index;
mod navbar;
mod theme;
mod link;

use crate::about::About;
use crate::examples::Examples;
use crate::index::index_page;
use crate::navbar::Navbar;
use crate::theme::BODY_BACKGROUND_COLOR;
use craft::components::{Component, ComponentId, ComponentSpecification};
use craft::elements::Container;
use craft::elements::ElementStyles;
use craft::style::Display;
use craft::style::FlexDirection;
use craft::{craft_main, CraftOptions, WindowContext};

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

impl Component for Website {
    type GlobalState = WebsiteGlobalState;
    type Props = ();
    type Message = ();

    fn view(
        &self,
        global_state: &Self::GlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext
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
            _ => wrapper.push(index_page().key("index")).component(),
        }
    }
}

fn main() {
    craft_main(Website::component(), WebsiteGlobalState::default(), CraftOptions::basic("Craft"));
}
