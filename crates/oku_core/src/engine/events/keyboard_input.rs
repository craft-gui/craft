use winit::event::{DeviceId, KeyEvent};

#[derive(Clone, Debug)]
pub struct KeyboardInput {
    pub device_id: Option<DeviceId>,
    pub event: KeyEvent,
    
    /// If `true`, the event was generated synthetically by winit
    /// in one of the following circumstances:
    ///
    /// * Synthetic key press events are generated for all keys pressed when a window gains
    ///   focus. Likewise, synthetic key release events are generated for all keys pressed when
    ///   a window goes out of focus. ***Currently, this is only functional on X11 and
    ///   Windows***
    ///
    /// Otherwise, this value is always `false`.
    pub is_synthetic: bool,
}

impl KeyboardInput {
    pub fn new(device_id: Option<DeviceId>, event: KeyEvent, is_synthetic: bool) -> KeyboardInput {
        Self {
            device_id,
            event,
            is_synthetic,
        }
    }
}