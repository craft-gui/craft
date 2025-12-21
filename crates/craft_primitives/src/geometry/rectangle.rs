use crate::geometry::Point;
use peniko::kurbo;
use dpi;

/// A structure representing a rectangle in 2D space.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
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
    #[inline(always)]
    pub fn contains(&self, point: &Point) -> bool {
        point.x as f32 >= self.left()
            && point.x as f32 <= self.right()
            && point.y as f32 >= self.top()
            && point.y as f32 <= self.bottom()
    }

    pub fn scale(&self, scale_factor: f64) -> Self {
        Rectangle {
            x: dpi::PhysicalUnit::from_logical::<f32, f32>(self.x, scale_factor).0,
            y: dpi::PhysicalUnit::from_logical::<f32, f32>(self.y, scale_factor).0,
            width: dpi::PhysicalUnit::from_logical::<f32, f32>(self.width, scale_factor).0,
            height: dpi::PhysicalUnit::from_logical::<f32, f32>(self.height, scale_factor).0,
        }
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
    #[inline(always)]
    pub fn position(&self) -> Point {
        Point::new(self.x as f64, self.y as f64)
    }

    /// Returns the y-coordinate of the top edge of the rectangle.
    #[inline(always)]
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Returns the x-coordinate of the right edge of the rectangle.
    #[inline(always)]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Returns the y-coordinate of the bottom edge of the rectangle.
    #[inline(always)]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Returns the x-coordinate of the left edge of the rectangle.
    #[inline(always)]
    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn to_kurbo(&self) -> kurbo::Rect {
        kurbo::Rect::new(self.x as f64, self.y as f64, self.right() as f64, self.bottom() as f64)
    }

    pub fn from_kurbo(rect: kurbo::Rect) -> Self {
        Rectangle {
            x: rect.x0 as f32,
            y: rect.y0 as f32,
            width: rect.width() as f32,
            height: rect.height() as f32,
        }
    }


    pub fn intersection(&self, other: &Rectangle) -> Option<Rectangle> {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = self.right().min(other.right());
        let y1 = self.bottom().min(other.bottom());

        if x0 < x1 && y0 < y1 {
            Some(Rectangle::new(x0, y0, x1 - x0, y1 - y0))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn intersects(&self, other: &Rectangle) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }
}

/*impl From<taffy::Rect<f32>> for Rectangle {
    fn from(rect: taffy::Rect<f32>) -> Self {
        Rectangle {
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        }
    }
}

impl From<parley::Rect> for Rectangle {
    fn from(rect: parley::Rect) -> Self {
        Rectangle {
            x: rect.x0 as f32,
            y: rect.y0 as f32,
            width: (rect.x1 - rect.x0) as f32,
            height: (rect.y1 - rect.y0) as f32,
        }
    }
}
*/