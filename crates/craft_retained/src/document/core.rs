use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Weak;

use ui_events::pointer::PointerId;

use crate::elements::ElementImpl;

/// Stores window specific information like pointer captures, focus (soon), etc.
pub struct Document {
    /// Tracks elements that are *currently* pointer captured.
    pub(crate) pointer_captures: HashMap<PointerId, Weak<RefCell<dyn ElementImpl>>>,
    /// Tracks elements that *should* be pointer captured.
    pub(crate) pending_pointer_captures: HashMap<PointerId, Weak<RefCell<dyn ElementImpl>>>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            pointer_captures: Default::default(),
            pending_pointer_captures: Default::default(),
        }
    }
}
