use std::rc::Rc;

use retgui::elements::{Container, Element, State, Window};
use retgui::style::{Display, FlexDirection};
use retgui::{App, pct};

use crate::WebsiteGlobalState;
use crate::docs::docs;
use crate::examples::examples;
use crate::index::index_page;
use crate::navbar::navbar;
use crate::theme::BODY_BACKGROUND_COLOR;

pub type NavigateFn = Rc<dyn Fn(&str, &mut App) + 'static>;

pub struct Router {
    state: State<RouterState>,
}

struct RouterState {
    root: Option<Window>,
    global_state: State<WebsiteGlobalState>,
    index: Option<Container>,
    docs: Option<Container>,
    examples: Option<Container>,
}

impl Router {
    pub fn new(app: &mut App, global_state: State<WebsiteGlobalState>) -> Self {
        let state = app.insert_state(RouterState {
            root: None,
            global_state,
            index: None,
            docs: None,
            examples: None,
        });
        let navigate: NavigateFn = Rc::new(move |route, app| {
            navigate_to(state, app, route);
        });

        let navigation = navbar(app, navigate.clone());
        let root = Window::new(app, "RetGui GUI")
            .edit(app)
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(pct(100))
            .height(pct(100))
            .background_color(BODY_BACKGROUND_COLOR)
            .push(navigation)
            .finish();
        let index = index_page(app, navigate.clone());
        let docs = docs(app, navigate.clone());
        let examples = examples(app, global_state, navigate);

        let router = state.write(app);
        router.root = Some(root);
        router.index = Some(index);
        router.docs = Some(docs);
        router.examples = Some(examples);
        Self { state }
    }

    pub fn navigate(&self, app: &mut App) {
        let global_state = self.state.read(app).global_state;
        let route = global_state.read(app).get_route();
        navigate_to(self.state, app, &route);
    }
}

fn navigate_to(state: State<RouterState>, app: &mut App, route: &str) {
    let (global_state, root, page) = {
        let router = state.read(app);
        let base = route.split('/').find(|part| !part.is_empty()).unwrap_or("");
        let page = match base {
            "docs" => router.docs.expect("docs page was not initialized"),
            "examples" => router.examples.expect("examples page was not initialized"),
            _ => router.index.expect("index page was not initialized"),
        };
        let root = router.root.expect("router root was not initialized");
        (router.global_state, root, page)
    };

    global_state.write(app).set_route(route);
    if let Some(current) = root.children(app).get(1).copied() {
        root.remove_child(app, current).expect("failed to remove routed page");
    }
    root.edit(app).push(page).finish();
}
