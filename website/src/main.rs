use std::cell::RefCell;
use std::rc::Rc;

use retgui::{RetGuiOptions, retgui_main};

use crate::router::Router;

thread_local! {
    // `winit` returns from `run_app` immediately on the web. Keep the router
    // alive after `main` returns so its weak navigation callbacks still work.
    static ROUTER: RefCell<Option<Rc<RefCell<Router>>>> = const { RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    // `Closure` removes its JavaScript callback when dropped.
    static POPSTATE_HANDLER: RefCell<Option<web_sys::wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>> =
        const { RefCell::new(None) };
}

mod docs;
mod examples;
mod index;
mod link;
mod navbar;
mod router;
mod theme;
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
        #[cfg(target_arch = "wasm32")]
        {
            // Reading the initial URL must not add a duplicate history entry.
            self.route = self.get_route();
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
    util::setup_logging();

    let global_state = Rc::new(RefCell::new(WebsiteGlobalState::default()));
    global_state.borrow_mut().load_route();
    let page_wrapper = Router::new(global_state.clone());
    page_wrapper.borrow().navigate();

    #[cfg(target_arch = "wasm32")]
    install_popstate_handler(global_state, Rc::downgrade(&page_wrapper));

    ROUTER.with_borrow_mut(|router| *router = Some(page_wrapper));
    retgui_main(RetGuiOptions::default());
}

#[cfg(target_arch = "wasm32")]
fn install_popstate_handler(global_state: Rc<RefCell<WebsiteGlobalState>>, router: std::rc::Weak<RefCell<Router>>) {
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::wasm_bindgen::closure::Closure;

    let handler = Closure::new(move |_event: web_sys::Event| {
        let route = global_state.borrow().get_route();
        global_state.borrow_mut().route = route;

        if let Some(router) = router.upgrade() {
            router.borrow().navigate();
        }
    });

    web_sys::window()
        .expect("No window available.")
        .add_event_listener_with_callback("popstate", handler.as_ref().unchecked_ref())
        .expect("Failed to install popstate handler");

    POPSTATE_HANDLER.with_borrow_mut(|slot| *slot = Some(handler));
}
