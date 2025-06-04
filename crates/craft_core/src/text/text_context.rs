use parley::{FontContext, TextStyle, TreeBuilder};

pub struct TextContext {
    pub font_context: FontContext,
    pub layout_context: parley::LayoutContext<ColorBrush>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorBrush {
    pub(crate) color: peniko::Color,
}

impl ColorBrush {
    pub fn new(color: peniko::Color) -> Self {
        Self { color }
    }
}

impl Default for ColorBrush {
    fn default() -> Self {
        Self {
            color: peniko::Color::BLACK,
        }
    }
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
