use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use craft_retained::elements::{Container, Element, Window};
use craft_retained::pct;
use craft_retained::style::{Display, FlexDirection};

use crate::navbar::navbar;
use crate::theme::BODY_BACKGROUND_COLOR;
use crate::{docs, index, WebsiteGlobalState};
use crate::docs::docs;
use crate::index::index_page;

#[derive(Clone)]
pub struct Router {
    pub root: Window,
    global_state: Rc<RefCell<WebsiteGlobalState>>,
    index: Container,
    docs: Container,
}

pub type NavigateFn = Box<dyn Fn(&str) + 'static>;

impl Router {
    pub fn new(global_state: Rc<RefCell<WebsiteGlobalState>>) -> Self {
        let window = Window::new("Craft Gui")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(pct(100))
            .height(pct(100))
            .push(navbar())
            .background_color(BODY_BACKGROUND_COLOR);
        Self {
            root: window.clone(),
            index: index_page(Box::new(|path| {
                println!("navigating");
            })),
            docs: docs(Box::new(|path| {

            })),
            global_state,
        }
    }

    pub fn set_content(&self, container: Container) {
        if let Some(current_content) = self.root.get_children().get(1) {
            self.root.remove_child(current_content.clone()).expect("Failed to remove child");
        }
        self.root.clone().push(container);
    }

    pub fn navigate(&self) {
        let page = match self.global_state.borrow().route.as_str() {
            "/" => {
                self.index.clone()
            }
            "/docs" => {
                self.docs.clone()
            }
            _ => {
                self.index.clone()
            }
        };

        self.set_content(page);
    }

    pub fn window(&self) -> Arc<craft_retained::WinitWindow> {
        self.root.inner.borrow().winit_window().expect("No widow")
    }
}
