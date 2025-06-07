mod about;
mod examples;
mod index;
mod link;
mod navbar;
mod theme;

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
use craft::WindowContext;
use craft::{craft_main, CraftOptions};
use craft::geometry::Size;

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
        _window: &WindowContext,
    ) -> ComponentSpecification {
        let wrapper = Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width("100%")
            .height("100%")
            .push(Navbar::component())
            .background(BODY_BACKGROUND_COLOR);

        match global_state.route.as_str() {
            "/examples" => wrapper.push(Examples::component().key("examples")),
            "/about" => wrapper.push(About::component().key("about")),
            _ => wrapper.push(index_page().key("index")),
        }
        .component()
    }
}

fn main() {
    let window_title = "Craft";
    
    #[cfg(not(target_arch = "wasm32"))]
    let options = CraftOptions {
        window_title: window_title.to_string(),
        window_size: Some(Size::new(1600.0, 900.0)),
        ..Default::default()
    };

    #[cfg(target_arch = "wasm32")]
    let options = CraftOptions::basic(window_title);
    
    craft_main(Website::component(), WebsiteGlobalState::default(), options);
}
