use parley::{FontContext, TextStyle, TreeBuilder};

pub(crate) struct TextContext {
    font_context: FontContext,
    layout_context: parley::LayoutContext,
    scale: f32,
}

pub(crate) type BrushType = [u8; 4];

impl TextContext {
    pub fn new() -> Self {
        Self {
            font_context: Default::default(),
            layout_context: Default::default(),
            scale: 1.0,
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn tree_builder<'a>(&'a mut self, raw_style: &TextStyle<'_, BrushType>) -> TreeBuilder<'a, BrushType> {
        self.layout_context.tree_builder(&mut self.font_context, self.scale, raw_style)
    }
}