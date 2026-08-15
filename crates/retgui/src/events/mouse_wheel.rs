use winit::event::{DeviceId, MouseScrollDelta, TouchPhase};

#[derive(Clone, Copy, Debug)]
pub struct MouseWheel {
    pub device_id: Option<DeviceId>,
    pub delta: MouseScrollDelta,
    pub phase: TouchPhase,
}

impl MouseWheel {
    pub fn new(device_id: Option<DeviceId>, delta: MouseScrollDelta, phase: TouchPhase) -> Self {
        Self {
            device_id,
            delta,
            phase,
        }
    }
}
