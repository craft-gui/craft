pub use parley;

pub use text_render_data::from_editor;

use std::ops::Range;

use crate::style::TextStyleProperty;

pub(crate) mod parley_editor;
pub mod text_commands;
pub mod text_context;
pub(crate) mod text_render_data;

#[derive(Clone, Default, PartialEq)]
pub struct RangedStyles {
    pub styles: Vec<(Range<usize>, TextStyleProperty)>,
}

impl RangedStyles {
    pub fn new(styles: Vec<(Range<usize>, TextStyleProperty)>) -> Self {
        Self { styles }
    }
}
