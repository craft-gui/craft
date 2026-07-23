use craft_primitives::geometry::Rectangle;
use peniko::Color;
use peniko::kurbo::{Affine, Line};
use craft_primitives::brush::Brush;

#[derive(Debug, Clone, Copy, Default)]
pub struct TextScroll {
    pub scroll_y: f32,
    pub scroll_height: f32,
}

impl TextScroll {
    pub fn new(scroll_y: f32, scroll_height: f32) -> Self {
        Self {
            scroll_y,
            scroll_height,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextRender {
    pub lines: Vec<TextRenderLine>,
    pub cursor: Option<(Rectangle, Brush)>,
    pub override_brush: Option<Brush>,
}

#[derive(Clone, Debug)]
pub struct TextRenderLine {
    pub items: Vec<TextRenderItem>,
    pub selections: Vec<(Rectangle, Brush)>,
    pub backgrounds: Vec<(Rectangle, Brush)>,
    pub min_y: f32,
    pub max_y: f32,
}

#[derive(Clone, Debug)]
pub struct TextRenderItem {
    pub brush: Brush,
    #[allow(dead_code)]
    pub underline: Option<TextRenderItemLine>,
    #[allow(dead_code)]
    pub strikethrough: Option<TextRenderItemLine>,
    #[allow(dead_code)]
    pub glyph_transform: Option<Affine>,
    pub font_size: f32,
    pub glyphs: Vec<TextRenderGlyph>,
    pub font: peniko::FontData,
}

#[derive(Clone, Debug)]
pub struct TextRenderItemLine {
    pub brush: Brush,
    #[allow(dead_code)]
    pub line: Line,
    #[allow(dead_code)]
    pub width: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TextRenderGlyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

pub trait TextData {
    fn get_text_renderer(&self) -> Option<&TextRender>;
}
