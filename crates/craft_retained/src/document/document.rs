use std::collections::HashMap;
use ui_events::pointer::PointerId;

pub struct Document {
    /// NOTE: The pointer capture code is old, it still needs to be properly implemented.
    /// Stores a pointer device id and their pointer captured element.
    pub(crate) pointer_captures: HashMap<PointerId, u64>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            pointer_captures: Default::default(),
        }
    }
}