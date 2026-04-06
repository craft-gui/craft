//! Stores a generic Element.

use std::cell::RefCell;
use std::rc::Rc;

use crate::elements::{AsElement, Element, ElementInternals};

#[derive(Clone)]
pub struct DynElement {
    pub inner: Rc<RefCell<dyn ElementInternals>>,
}

impl Element for DynElement {}

impl AsElement for DynElement {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }
}

impl DynElement {
    pub const fn new(inner: Rc<RefCell<dyn ElementInternals>>) -> DynElement {
        Self { inner }
    }
}
