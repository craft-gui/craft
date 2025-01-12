use winit::dpi::PhysicalPosition;

/// A structure representing a point in 2D space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// The x-coordinate of the point.
    pub x: f32,
    /// The y-coordinate of the point.
    pub y: f32,
}

impl Point {
    /// Creates a new `Point` with the given x and y coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - A float representing the x-coordinate of the point.
    /// * `y` - A float representing the y-coordinate of the point.
    ///
    /// # Returns
    ///
    /// A `Point` instance with the specified coordinates.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl From<PhysicalPosition<f64>> for Point {
    fn from(position: PhysicalPosition<f64>) -> Self {
        Point::new(position.x as f32, position.y as f32)
    }
}

impl From<taffy::Point<f32>> for Point {
    fn from(point: taffy::Point<f32>) -> Self {
        Point::new(point.x, point.y)
    }
}