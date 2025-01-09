mod style;
mod taffy_conversions;
pub(crate) mod style_flags;

pub use style::*;
use crate::geometry::Rectangle;

enum StyleValue {
    Rectangle(Rectangle)
}

struct StyleValueInfo {
    is_modified: bool,
    value: StyleValue,
}

impl StyleField for StyleValueInfo {
    fn update(&mut self, value: StyleValue) {
        self.is_modified = true;
        self.value = value;
    }
}

trait StyleField where Self: Sized {
    fn update(&mut self, style_value: StyleValue);
}