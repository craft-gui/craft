use parley::{FontContext, TextStyle, TreeBuilder};
use craft_primitives::ColorBrush;

pub struct TextContext {
    pub font_context: FontContext,
    pub layout_context: parley::LayoutContext<ColorBrush>,
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

    pub fn tree_builder<'a>(
        &'a mut self,
        scale: f32,
        raw_style: &TextStyle<'_, ColorBrush>,
    ) -> TreeBuilder<'a, ColorBrush> {
        self.layout_context.tree_builder(&mut self.font_context, scale, true, raw_style)
    }
}
