use winit::dpi::PhysicalPosition;

pub use kurbo::Point;

/// A “Point-converter” extension trait.
pub trait PointConverter {
    /// “Constructor” for a kurbo::Point
    fn new(x: f32, y: f32) -> Self;

    /// Convert a winit position into a kurbo::Point
    fn from_physical_pos(pos: PhysicalPosition<f64>) -> Self;

    /// Convert a taffy::Point into a kurbo::Point
    fn from_taffy_point(p: taffy::Point<f32>) -> Self;
}

impl PointConverter for Point {
    fn new(x: f32, y: f32) -> Self {
        Point {
            x: x as f64,
            y: y as f64,
        }
    }

    fn from_physical_pos(pos: PhysicalPosition<f64>) -> Self {
        Point { x: pos.x, y: pos.y }
    }

    fn from_taffy_point(p: taffy::Point<f32>) -> Self {
        Point {
            x: p.x as f64,
            y: p.y as f64,
        }
    }
}
