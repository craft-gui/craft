use std::rc::Rc;

use retgui::elements::{Container, Element, Elements, State, Window};
use retgui::pct;
use retgui::style::{Display, FlexDirection};

use crate::WebsiteGlobalState;
use crate::docs::docs;
use crate::examples::examples;
use crate::index::index_page;
use crate::navbar::navbar;
use crate::theme::BODY_BACKGROUND_COLOR;

pub type NavigateFn = Rc<dyn Fn(&str, &mut Elements) + 'static>;

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
    pub fn new(elements: &mut Elements, global_state: State<WebsiteGlobalState>) -> Self {
        let state = elements.insert_state(RouterState {
            root: None,
            global_state,
            index: None,
            docs: None,
            examples: None,
        });
        let navigate: NavigateFn = Rc::new(move |route, elements| {
            navigate_to(state, elements, route);
        });

        let navigation = navbar(elements, navigate.clone());
        let root = Window::new(elements, "RetGui GUI")
            .edit(elements)
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(pct(100))
            .height(pct(100))
            .background_color(BODY_BACKGROUND_COLOR)
            .push(navigation)
            .finish();
        let index = index_page(elements, navigate.clone());
        let docs = docs(elements, navigate.clone());
        let examples = examples(elements, global_state, navigate);

        let router = state.write(elements);
        router.root = Some(root);
        router.index = Some(index);
        router.docs = Some(docs);
        router.examples = Some(examples);
        Self { state }
    }

    pub fn navigate(&self, elements: &mut Elements) {
        let global_state = self.state.read(elements).global_state;
        let route = global_state.read(elements).get_route();
        navigate_to(self.state, elements, &route);
    }
}

fn navigate_to(state: State<RouterState>, elements: &mut Elements, route: &str) {
    let (global_state, root, page) = {
        let router = state.read(elements);
        let base = route.split('/').find(|part| !part.is_empty()).unwrap_or("");
        let page = match base {
            "docs" => router.docs.expect("docs page was not initialized"),
            "examples" => router.examples.expect("examples page was not initialized"),
            _ => router.index.expect("index page was not initialized"),
        };
        let root = router.root.expect("router root was not initialized");
        (router.global_state, root, page)
    };

    global_state.write(elements).set_route(route);
    if let Some(current) = root.children(elements).get(1).copied() {
        root.remove_child(elements, current)
            .expect("failed to remove routed page");
    }
    root.edit(elements).push(page).finish();
}
