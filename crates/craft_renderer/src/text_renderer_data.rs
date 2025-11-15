use craft_primitives::geometry::Rectangle;
use peniko::Color;
use peniko::kurbo::{Affine, Line};
use craft_primitives::ColorBrush;

#[derive(Clone, Debug)]
pub struct TextRender {
    pub lines: Vec<TextRenderLine>,
    pub cursor: Option<(Rectangle, Color)>,
    pub override_brush: Option<ColorBrush>,
}

#[derive(Clone, Debug)]
pub struct TextRenderLine {
    pub items: Vec<TextRenderItem>,
    pub selections: Vec<(Rectangle, Color)>,
    pub backgrounds: Vec<(Rectangle, Color)>,
}

#[derive(Clone, Debug)]
pub struct TextRenderItem {
    pub brush: ColorBrush,
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

#[derive(Clone, Copy, Debug)]
pub struct TextRenderItemLine {
    pub brush: ColorBrush,
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