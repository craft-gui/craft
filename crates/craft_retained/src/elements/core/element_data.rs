use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::elements::Element;

/// The trait to provide common element data.
pub trait ElementData {
    /// Get a shared reference to this element's common element data.
    fn element_data(&self) -> &crate::elements::element_data::ElementData;

    /// Get a mutable reference to this element's common element data.
    fn element_data_mut(&mut self) -> &mut crate::elements::element_data::ElementData;

    /// Returns a unique id for the element.
    fn id(&self) -> u64 {
        self.element_data().internal_id
    }

    /// Returns the element's parent element.
    fn parent(&self) -> Option<Weak<RefCell<dyn Element>>> {
        self.element_data().parent.clone()
    }

    /// Returns the element's children.
    fn children(&self) -> &[Rc<RefCell<dyn Element>>] {
        self.element_data().children.as_slice()
    }
}
