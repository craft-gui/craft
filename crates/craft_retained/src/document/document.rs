use std::collections::HashMap;
use ui_events::pointer::PointerId;

/// Stores window specific information like pointer captures, focus (soon), etc.
pub struct Document {
    /// Tracks elements that are *currently* pointer captured.
    pub(crate) pointer_captures: HashMap<PointerId, u64>,
    /// Tracks elements that *should* be pointer captured.
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