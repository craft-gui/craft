pub mod text_context;
pub(crate) mod text_render_data;

pub use parley;
use std::ops::Range;

use crate::style::TextStyleProperty;
pub use text_render_data::from_editor;

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