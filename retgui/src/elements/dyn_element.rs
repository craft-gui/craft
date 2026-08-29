//! Stores a generic Element.

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::elements::{AsElement, Element, ElementInternals};

#[derive(Clone)]
pub struct DynElement {
    pub inner: Rc<RefCell<dyn ElementInternals>>,
}

impl PartialEq for DynElement {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for DynElement {}

impl Element for DynElement {}

impl AsElement for DynElement {
    type Inner = dyn ElementInternals;

    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }

    fn with<R>(&self, callback: impl FnOnce(&Self::Inner) -> R) -> R {
        callback(&*self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut Self::Inner) -> R) -> R {
        callback(&mut *self.inner.borrow_mut())
    }
}

impl DynElement {
    pub const fn new(inner: Rc<RefCell<dyn ElementInternals>>) -> DynElement {
        Self { inner }
    }
}
