use dpi;

use crate::geometry::{Point, Rectangle};

/// A structure representing a circle in 2D space.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Circle {
    /// The center x-coordinate of the circle.
    pub x: f32,
    /// The center y-coordinate of the circle.
    pub y: f32,
    /// The radius of the circle..
    pub radius: f32,
}

impl Circle {
    /// Checks if the circle contains a given point.
    ///
    /// # Arguments
    ///
    /// * `point` - A reference to a `Point` to check.
    ///
    /// # Returns
    ///
    /// `true` if the circle contains the point, `false` otherwise.
    #[inline(always)]
    pub fn contains(&self, point: &Point) -> bool {
        (point.x - self.x as f64) * (point.x - self.x as f64) + (point.y - self.y as f64) * (point.x - self.x as f64)
            < (self.radius * self.radius) as f64
    }

    pub fn scale(&self, scale_factor: f64) -> Self {
        Circle {
            x: dpi::PhysicalUnit::from_logical::<f32, f32>(self.x, scale_factor).0,
            y: dpi::PhysicalUnit::from_logical::<f32, f32>(self.y, scale_factor).0,
            radius: dpi::PhysicalUnit::from_logical::<f32, f32>(self.radius, scale_factor).0,
        }
    }
    
    /// Creates a new `Circle` with the given position and size.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the top-left corner of the circle.
    /// * `y` - The y-coordinate of the top-left corner of the circle.
    /// * `radius` - The radius of the circle.
    ///
    /// # Returns
    ///
    /// A `Circle` instance with the specified position and size.
    pub fn new(x: f32, y: f32, radius: f32) -> Self {
        Circle {
            x,
            y,
            radius,
        }
    }

    /// Returns the position of the top-left corner of the circle.
    #[inline(always)]
    pub fn position(&self) -> Point {
        Point::new(self.x as f64, self.y as f64)
    }

    pub fn to_kurbo(&self) -> kurbo::Circle {
        kurbo::Circle::new((self.x as f64, self.y as f64), self.radius as f64)
    }

    pub fn from_kurbo(circle: kurbo::Circle) -> Self {
        Circle {
            x: circle.center.x as f32,
            y: circle.center.y as f32,
            radius: circle.radius as f32,
        }
    }

    #[inline(always)]
    pub fn intersects(&self, other: &Circle) -> bool {
        (other.x - self.x) * (other.x - self.x) + (other.y - self.y) * (other.y - self.y) < (self.radius + other.radius) * (self.radius + other.radius)
    }

    pub fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.x - self.radius, self.y - self.radius, self.radius * 2.0, self.radius * 2.0)
    }

    #[inline(always)]
    pub fn intersects_rect(&self, other: &Rectangle) -> bool {
        self.bounding_box().intersects(other)
    }

    pub fn expand(&self, radius: f32) -> Self {
        Circle::new(
            self.x,
            self.y,
            self.radius + radius,
        )
    }
}