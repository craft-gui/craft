use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use craft_retained::elements::{AsElement, ElementInternals};

use crate::elements::element::Element;

#[derive(Clone)]
pub struct Container {
    pub inner: craft_retained::elements::Container,
}

impl AsElement for Container {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }
}

impl Element for Container {}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Container {
    pub fn new() -> Self {
        Self {
            inner: craft_retained::elements::Container::new(),
        }
    }
}
