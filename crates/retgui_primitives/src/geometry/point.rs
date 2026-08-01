use dpi::PhysicalPosition;
pub use kurbo::Point;

/// A “Point-converter” extension trait.
pub trait PointConverter {
    /// “Constructor” for a kurbo::Point
    fn new(x: f32, y: f32) -> Self;

    /// Convert a winit position into a kurbo::Point
    fn from_physical_pos(pos: PhysicalPosition<f64>) -> Self;

    ///// Convert a taffy::Point into a kurbo::Point
    //fn from_taffy_point(p: taffy::Point<f32>) -> Self;
}
