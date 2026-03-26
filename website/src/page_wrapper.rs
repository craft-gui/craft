use std::sync::Arc;
use craft_retained::elements::{Container, Element, Window};
use craft_retained::pct;
use craft_retained::style::{Display, FlexDirection};

use crate::navbar::navbar;
use crate::theme::BODY_BACKGROUND_COLOR;

pub struct PageWrapper {
    root: Window,
}

impl PageWrapper {
    pub fn new() -> Self {
        Self {
            root: Window::new("Craft Gui")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .width(pct(100))
                .height(pct(100))
                .push(navbar())
                .background_color(BODY_BACKGROUND_COLOR),
        }
    }

    pub fn set_content(&mut self, container: Container) {
        if let Some(current_content) = self.root.get_children().get(1) {
            self.root.remove_child(current_content.clone()).expect("Failed to remove child");
        }
        self.root.clone().push(container);
    }

    pub fn window(&self) -> Arc<craft_retained::WinitWindow> {
        self.root.inner.borrow().winit_window().expect("No widow")
    }
}
