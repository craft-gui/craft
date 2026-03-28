use craft_retained::elements::{Container, Element, Window};
use craft_retained::pct;
use craft_retained::style::{Display, FlexDirection};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use crate::docs::docs;
use crate::index::index_page;
use crate::navbar::navbar;
use crate::theme::BODY_BACKGROUND_COLOR;
use crate::{WebsiteGlobalState, docs, index};

#[derive(Clone)]
pub struct Router {
    pub root: Window,
    global_state: Rc<RefCell<WebsiteGlobalState>>,
    index: Container,
    docs: Container,
}

pub type NavigateFn = Rc<dyn Fn(&str) + 'static>;

impl Router {
    pub fn new(global_state: Rc<RefCell<WebsiteGlobalState>>) -> Rc<RefCell<Self>> {
        let state = global_state.clone();
        Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
            let me = me.clone();

            let navigate_logic: NavigateFn = Rc::new(move |route| {
                state.borrow_mut().set_route(route);
                if let Some(router) = me.upgrade() {
                    router.borrow().navigate();
                }
            });

            let window = Window::new("Craft Gui")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .width(pct(100))
                .height(pct(100))
                .push(navbar(navigate_logic.clone()))
                .background_color(BODY_BACKGROUND_COLOR);

            RefCell::new(Self {
                root: window.clone(),
                index: index_page(navigate_logic.clone()),
                docs: docs(navigate_logic.clone()),
                global_state: global_state.clone(),
            })
        })
    }

    fn set_content(&self, container: Container) {
        if let Some(current_content) = self.root.get_children().get(1) {
            self.root
                .remove_child(current_content.clone())
                .expect("Failed to remove child");
        }
        self.root.clone().push(container);
    }

    pub fn navigate(&self) {
        let page = match self.global_state.borrow().route.as_str() {
            "/" => self.index.clone(),
            "/docs" => self.docs.clone(),
            _ => self.index.clone(),
        };

        self.set_content(page);
    }

    /*pub fn window(&self) -> Arc<craft_retained::WinitWindow> {
        self.root.inner.borrow().winit_window().expect("No widow")
    }*/
}
