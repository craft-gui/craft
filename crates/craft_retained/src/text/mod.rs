pub mod text_context;
pub(crate) mod text_render_data;
pub(crate) mod parley_editor;

use std::ops::Range;
pub use parley;

use crate::style::{Style, TextStyleProperty};
pub use text_render_data::from_editor;

#[derive(PartialEq)]
pub(crate) struct TextStyle {
    pub(crate) font_size: f32,
}

impl From<&Style> for TextStyle {
    fn from(style: &Style) -> Self {
        TextStyle {
            font_size: style.font_size(),
        }
    }
}

#[derive(Clone)]
#[derive(Default)]
#[derive(PartialEq)]
pub struct RangedStyles  {
    pub styles: Vec<(Range<usize>, TextStyleProperty)>,
}

impl RangedStyles {
    pub fn new(styles: Vec<(Range<usize>, TextStyleProperty)>) -> Self {
        Self { 
            styles
        }
    }
}