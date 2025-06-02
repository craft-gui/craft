pub mod text_context;
pub(crate) mod text_render_data;
pub use parley;

pub use text_render_data::TextRender;
pub use text_render_data::from_editor;
use crate::style::Style;

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