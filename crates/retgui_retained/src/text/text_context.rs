use retgui_primitives::brush::Brush;
use parley::{FontContext, TextStyle, TreeBuilder};

pub struct TextContext {
    pub font_context: FontContext,
    pub layout_context: parley::LayoutContext<Brush>,
}

impl Default for TextContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TextContext {
    pub fn new() -> Self {
        Self {
            font_context: Default::default(),
            layout_context: Default::default(),
        }
    }

    pub fn tree_builder<'a>(&'a mut self, scale: f32, raw_style: &TextStyle<'_, '_, Brush>) -> TreeBuilder<'a, Brush> {
        self.layout_context
            .tree_builder(&mut self.font_context, scale, true, raw_style)
    }
}
