use std::rc::Rc;

use peniko::kurbo::{Affine, Line};

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::Rectangle;

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
    pub min_x: f32,
    pub max_x: f32,
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
    pub normalized_coords: Vec<i16>,
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

    /// Overrides the brushes stored in individual text runs.
    fn override_brush(&self) -> Option<&Brush> {
        self.get_text_renderer()
            .and_then(|render| render.override_brush.as_ref())
    }

    /// Whether glyphs should be rasterized into and reused from the glyph atlas.
    fn use_glyph_cache(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct TextSnapshot {
    render: Rc<TextRender>,
    override_brush: Option<Brush>,
    use_glyph_cache: bool,
}

impl TextSnapshot {
    pub fn new(render: TextRender, use_glyph_cache: bool) -> Self {
        let override_brush = render.override_brush.clone();
        Self {
            render: Rc::new(render),
            override_brush,
            use_glyph_cache,
        }
    }

    pub fn from_shared(render: Rc<TextRender>, override_brush: Option<Brush>, use_glyph_cache: bool) -> Self {
        Self {
            render,
            override_brush,
            use_glyph_cache,
        }
    }
}

impl TextData for TextSnapshot {
    fn get_text_renderer(&self) -> Option<&TextRender> {
        Some(self.render.as_ref())
    }

    fn override_brush(&self) -> Option<&Brush> {
        self.override_brush.as_ref()
    }

    fn use_glyph_cache(&self) -> bool {
        self.use_glyph_cache
    }
}
