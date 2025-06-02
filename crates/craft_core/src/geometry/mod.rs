pub(crate) mod borders;
pub(crate) mod corner;
pub(crate) mod cornerside;
mod element_box;
mod point;
mod rectangle;
pub(crate) mod side;
mod size;
mod trblrectangle;

pub use element_box::ElementBox;
pub use point::Point;
pub use point::PointConverter;
pub use rectangle::Rectangle;
pub use size::Size;
pub use trblrectangle::TrblRectangle;

pub type Border = TrblRectangle<f32>;
pub type Padding = TrblRectangle<f32>;
pub type Margin = TrblRectangle<f32>;

pub use glam;