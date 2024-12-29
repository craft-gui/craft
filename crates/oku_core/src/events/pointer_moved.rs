use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, PointerSource};
use crate::geometry::Point;

#[derive(Clone, Debug)]
pub struct PointerMoved {
    pub device_id: Option<DeviceId>,

    /// (x,y) coordinates in pixels relative to the top-left corner of the window. Because the
    /// range of this data is limited by the display area and it may have been
    /// transformed by the OS to implement effects such as pointer acceleration, it
    /// should not be used to implement non-pointer-like interactions such as 3D camera
    /// control. For that, consider [`DeviceEvent::PointerMotion`].
    ///
    /// ## Platform-specific
    ///
    /// **Web:** Doesn't take into account CSS [`border`], [`padding`], or [`transform`].
    ///
    /// [`border`]: https://developer.mozilla.org/en-US/docs/Web/CSS/border
    /// [`padding`]: https://developer.mozilla.org/en-US/docs/Web/CSS/padding
    /// [`transform`]: https://developer.mozilla.org/en-US/docs/Web/CSS/transform
    pub position: Point,

    pub source: PointerSource,
    pub primary: bool,
}

impl PointerMoved {
    pub fn new(
        device_id: Option<DeviceId>,
        position: PhysicalPosition<f64>,
        source: PointerSource,
        primary: bool,
    ) -> Self {
        Self {
            device_id,
            position: position.into(),
            source,
            primary,
        }
    }
}
