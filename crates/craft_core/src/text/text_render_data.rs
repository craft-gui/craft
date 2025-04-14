use peniko::kurbo::{Affine, Line};

#[derive(Clone, Debug)]
pub struct TextRender {
    pub(crate) lines: Vec<TextRenderLine>,
}

#[derive(Clone, Debug)]
pub struct TextRenderLine {
    pub(crate) items: Vec<TextRenderItem>,
}

#[derive(Clone, Debug)]
pub struct TextRenderItem {
    pub(crate) underline: Option<TextRenderItemLine>,
    pub(crate) strikethrough: Option<TextRenderItemLine>,
    pub(crate) glyph_transform: Option<Affine>,
    pub(crate) font_size: f32,
    pub(crate) glyphs: Vec<TextRenderGlyph>,
    pub(crate) font: parley::Font,
}

#[derive(Clone, Copy, Debug)]
pub struct TextRenderItemLine {
    pub(crate) line: Line,
    pub(crate) width: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TextRenderGlyph {
    pub(crate) id: parley::swash::GlyphId,
    pub(crate) x: f32,
    pub(crate) y: f32,
}