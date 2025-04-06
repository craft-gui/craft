use crate::geometry::Point;
use peniko::kurbo;

/// A structure representing a rectangle in 2D space.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rectangle {
    /// The x-coordinate of the top-left corner of the rectangle.
    pub x: f32,
    /// The y-coordinate of the top-left corner of the rectangle.
    pub y: f32,
    /// The width of the rectangle.
    pub width: f32,
    /// The height of the rectangle.
    pub height: f32,
}

impl Rectangle {
    /// Checks if the rectangle contains a given point.
    ///
    /// # Arguments
    ///
    /// * `point` - A reference to a `Point` to check.
    ///
    /// # Returns
    ///
    /// `true` if the rectangle contains the point, `false` otherwise.
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.left() && point.x <= self.right() && point.y >= self.top() && point.y <= self.bottom()
    }
}

impl Rectangle {
    /// Creates a new `Rectangle` with the given position and size.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the top-left corner of the rectangle.
    /// * `y` - The y-coordinate of the top-left corner of the rectangle.
    /// * `width` - The width of the rectangle.
    /// * `height` - The height of the rectangle.
    ///
    /// # Returns
    ///
    /// A `Rectangle` instance with the specified position and size.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the position of the top-left corner of the rectangle.
    pub fn position(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Returns the y-coordinate of the top edge of the rectangle.
    #[inline]
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Returns the x-coordinate of the right edge of the rectangle.
    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Returns the y-coordinate of the bottom edge of the rectangle.
    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Returns the x-coordinate of the left edge of the rectangle.
    #[inline]
    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn to_kurbo(&self) -> kurbo::Rect {
        kurbo::Rect::new(self.x as f64, self.y as f64, self.right() as f64, self.bottom() as f64)
    }
}

impl From<taffy::Rect<f32>> for Rectangle {
    fn from(rect: taffy::Rect<f32>) -> Self {
        Rectangle {
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        }
    }
}
