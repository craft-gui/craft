mod examples;
mod index;
mod web_link;
mod navbar;
mod theme;
mod router;
mod docs;
mod link;

use crate::index::index_page;
use crate::navbar::Navbar;
use crate::router::resolve_route;
use crate::theme::BODY_BACKGROUND_COLOR;
use craft::components::{Component, ComponentSpecification, Context};
use craft::elements::Container;
use craft::elements::ElementStyles;
#[cfg(not(target_arch = "wasm32"))]
use craft::geometry::Size;
use craft::style::Display;
use craft::style::FlexDirection;
use craft::{craft_main, CraftOptions};

pub(crate) struct WebsiteGlobalState {
    /// The current route that we are viewing.
    route: String,
}

impl WebsiteGlobalState {
    pub(crate) fn get_route(&self) -> String {
        #[cfg(target_arch = "wasm32")]
        let path: String;
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().expect("No window available.");
            path = window.location().pathname().map(
                |s|  {
                    let trimmed_path = s.trim_end_matches('/');
                    if trimmed_path.is_empty() {
                        "/".to_string()
                    } else {
                        trimmed_path.to_string()
                    }
                }
            ).unwrap_or("/".to_string());
        }
        #[cfg(not(target_arch = "wasm32"))]
        let path = self.route.clone();
        path
    }

    pub(crate) fn set_route(&mut self, route: &str) {
        self.route = route.to_string();

        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let history = window.history().unwrap();
            
            history
                .push_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(route))
                .unwrap();
        }
    }
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

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        let wrapper = Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width("100%")
            .height("100%")
            .push(Navbar::component())
            .background(BODY_BACKGROUND_COLOR);


        let path = context.global_state().get_route();
        let matched_mapped_path = resolve_route(path.as_str(), context.window());
        if let Some(rule) = matched_mapped_path {
            wrapper.push(rule.component_specification)
        } else {
            wrapper.push(index_page(context.window()).key("index"))
        }.component()
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

    #[allow(unused_mut)]
    let mut global_state = WebsiteGlobalState::default();
    #[cfg(not(target_arch = "wasm32"))]
    {
        // NOTE: In Git Bash, use `cargo run -- //examples`.
        let route = std::env::args().nth(1).unwrap_or_else(|| "/".to_string());
        global_state.set_route(route.as_str());
    }

    util::setup_logging();

    craft_main(Website::component(), global_state, options);
}
