//! Stores a generic Element.

use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::elements::{AsElement, Element, ElementInternals};

#[derive(Clone)]
pub struct DynElement {
    pub(crate) inner: Rc<RefCell<dyn ElementInternals>>,
}

impl PartialEq for DynElement {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for DynElement {}

impl Element for DynElement {}

impl AsElement for DynElement {
    fn with<R>(&self, callback: impl FnOnce(&dyn ElementInternals) -> R) -> R {
        callback(&*self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn ElementInternals) -> R) -> R {
        callback(&mut *self.inner.borrow_mut())
    }
}

impl DynElement {
    pub(crate) const fn new(inner: Rc<RefCell<dyn ElementInternals>>) -> DynElement {
        Self { inner }
    }

    pub(crate) fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }
}
