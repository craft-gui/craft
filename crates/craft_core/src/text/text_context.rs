use parley::{FontContext, TextStyle, TreeBuilder};

pub(crate) struct TextContext {
    pub font_context: FontContext,
    pub layout_context: parley::LayoutContext<ColorBrush>,
    pub scale: f32,
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

impl TextContext {
    pub fn new() -> Self {
        Self {
            font_context: Default::default(),
            layout_context: Default::default(),
            scale: 1.0,
        }
    }

    #[allow(dead_code)]
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn tree_builder<'a>(&'a mut self, raw_style: &TextStyle<'_, ColorBrush>) -> TreeBuilder<'a, ColorBrush> {
        self.layout_context.tree_builder(&mut self.font_context, self.scale, true, raw_style)
    }
}