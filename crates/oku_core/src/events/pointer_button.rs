use crate::geometry::Point;
use winit::dpi::PhysicalPosition;
use winit::event::{ButtonSource, DeviceId, ElementState};

#[derive(Clone, Copy, Debug)]
pub struct PointerButton {
    pub device_id: Option<DeviceId>,
    pub state: ElementState,

    /// The position of the pointer when the button was pressed.
    ///
    /// ## Platform-specific
    ///
    /// - **Orbital: Always emits `(0., 0.)`.
    /// - **Web:** Doesn't take into account CSS [`border`], [`padding`], or [`transform`].
    ///
    /// [`border`]: https://developer.mozilla.org/en-US/docs/Web/CSS/border
    /// [`padding`]: https://developer.mozilla.org/en-US/docs/Web/CSS/padding
    /// [`transform`]: https://developer.mozilla.org/en-US/docs/Web/CSS/transform
    pub position: Point,

    pub button: ButtonSource,
    pub primary: bool,
}

impl PointerButton {
    pub fn new(
        device_id: Option<DeviceId>,
        state: ElementState,
        position: PhysicalPosition<f64>,
        button: ButtonSource,
        primary: bool,
    ) -> Self {
        Self {
            device_id,
            state,
            position: position.into(),
            button,
            primary,
        }
    }
}
