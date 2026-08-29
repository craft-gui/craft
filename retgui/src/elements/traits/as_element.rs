use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::elements::ElementInternals;

/// Used as a super trait in `Element`, so that the inner element can be retrieved.
pub trait AsElement {
    type Inner: ElementInternals + ?Sized;

    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>>;
    fn borrow(&self) -> Ref<'_, dyn ElementInternals>;
    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals>;

    /// Borrows the concrete element internals for the duration of `callback`.
    ///
    /// This is useful when reading data by reference would otherwise require cloning it.
    fn with<R>(&self, callback: impl FnOnce(&Self::Inner) -> R) -> R;

    /// Mutably borrows the concrete element internals for the duration of `callback`.
    fn with_mut<R>(&self, callback: impl FnOnce(&mut Self::Inner) -> R) -> R;
}