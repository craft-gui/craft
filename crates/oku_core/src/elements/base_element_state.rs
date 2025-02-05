use std::collections::HashMap;
use winit::event::DeviceId;
use crate::elements::element_states::ElementState;

#[derive(Debug, Default, Clone)]
pub struct BaseElementState {
    #[allow(dead_code)]
    pub(crate) current_state: ElementState,
    /// Whether this element should receive pointer events regardless of hit testing.
    /// Useful for scroll thumbs.
    pub(crate) pointer_capture: HashMap<i64, bool>,
}

// HACK: Remove this and all usages when pointer capture per device works. 
pub(crate) const DUMMY_DEVICE_ID: i64 = -1;