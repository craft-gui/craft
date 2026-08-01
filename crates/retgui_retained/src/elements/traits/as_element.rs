use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::elements::ElementInternals;

/// Used as a super trait in `Element`, so that the inner element can be retrieved.
pub trait AsElement {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>>;
    fn borrow(&self) -> Ref<'_, dyn ElementInternals>;
    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals>;
}
