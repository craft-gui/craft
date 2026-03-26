use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use craft_retained::elements::{Container, Element, Window};
use craft_retained::{CraftOptions, craft_main, pct};

use crate::page_wrapper::PageWrapper;

mod navbar;
mod page_wrapper;
mod router;
mod theme;
mod index;
mod web_link;

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
            path = window
                .location()
                .pathname()
                .map(|s| {
                    let trimmed_path = s.trim_end_matches('/');
                    if trimmed_path.is_empty() {
                        "/".to_string()
                    } else {
                        trimmed_path.to_string()
                    }
                })
                .unwrap_or("/".to_string());
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

    pub fn load_route(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // NOTE: In Git Bash, use `cargo run -- //examples`.
            let route = std::env::args().nth(1).unwrap_or_else(|| "/".to_string());
            self.set_route(route.as_str());
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

fn main() {
    let options = CraftOptions {
        ..Default::default()
    };

    #[allow(unused_mut)]
    let mut global_state = Rc::new(RefCell::new(WebsiteGlobalState::default()));

    util::setup_logging();

    let mut page_wrapper = PageWrapper::new();

    let page = match global_state.borrow().route.as_str() {
        "/" => {
            index::index_page()
        }
        _ => {
            index::index_page()
        }
    };

    page_wrapper.set_content(page);

    craft_main(options);
}
