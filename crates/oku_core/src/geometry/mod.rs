pub(crate) mod borders;
pub(crate) mod corner;
pub(crate) mod cornerside;
mod element_rectangle;
mod point;
mod rectangle;
pub(crate) mod side;
mod size;
mod trblrectangle;

pub use element_rectangle::ElementBox;
pub use point::Point;
pub use rectangle::Rectangle;
pub use size::Size;
pub use trblrectangle::TrblRectangle;

pub type Border = TrblRectangle;
pub type Padding = TrblRectangle;
pub type Margin = TrblRectangle;
