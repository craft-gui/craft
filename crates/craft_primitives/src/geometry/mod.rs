pub use kurbo::{Affine, BezPath, Shape, Vec2};

pub use crate::geometry::element_box::ElementBox;
pub use crate::geometry::point::{Point, PointConverter};
pub use crate::geometry::rectangle::Rectangle;
pub use crate::geometry::size::Size;
pub use crate::geometry::trblrectangle::TrblRectangle;

pub mod borders;

mod element_box;
mod point;
mod rectangle;
mod size;
mod trblrectangle;

pub type Border = TrblRectangle<f32>;
pub type Padding = TrblRectangle<f32>;
pub type Margin = TrblRectangle<f32>;
