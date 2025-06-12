pub mod text_context;
pub(crate) mod text_render_data;
mod editor;
mod parley_editor;

pub use parley;

use crate::style::Style;
pub use text_render_data::from_editor;
pub use text_render_data::TextRender;

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
