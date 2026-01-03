pub(crate) mod parley_editor;
pub mod text_context;
pub(crate) mod text_render_data;

use std::ops::Range;

pub use parley;
pub use text_render_data::from_editor;

use crate::style::TextStyleProperty;

#[derive(Clone, Default, PartialEq)]
pub struct RangedStyles {
    pub styles: Vec<(Range<usize>, TextStyleProperty)>,
}

impl RangedStyles {
    pub fn new(styles: Vec<(Range<usize>, TextStyleProperty)>) -> Self {
        Self { styles }
    }
}
