use std::collections::HashMap;
use ui_events::pointer::PointerId;

pub struct Document {
    pub(crate) pointer_captures: HashMap<PointerId, u64>,
    pub(crate) pending_pointer_captures: HashMap<PointerId, u64>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            pointer_captures: Default::default(),
            pending_pointer_captures: Default::default(),
        }
    }
}