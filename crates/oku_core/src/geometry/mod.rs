
pub(crate) mod borders;
pub(crate) mod corner;
pub(crate) mod cornerside;
pub(crate) mod side;
mod size;
mod point;
mod rectangle;
mod trblrectangle;
mod element_rectangle;

pub use rectangle::Rectangle;
pub use point::Point;
pub use size::Size;
pub use trblrectangle::TrblRectangle;
pub use element_rectangle::ElementRectangle;

pub type Border = TrblRectangle;
pub type Padding = TrblRectangle;
pub type Margin = TrblRectangle;
