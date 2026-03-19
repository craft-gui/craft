use std::cell::RefCell;
use std::rc::Rc;

use crate::elements::ElementInternals;

/// Used as a super trait in `Element`, so that
/// the internal element can be retrieved.
pub trait AsElement {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>>;
}
