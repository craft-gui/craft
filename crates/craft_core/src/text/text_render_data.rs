use parley::{Layout, PositionedLayoutItem};
use peniko::kurbo::{Affine, Line};
use crate::geometry::Rectangle;
use crate::text::text_context::ColorBrush;

#[derive(Clone, Debug)]
pub struct TextRender {
    pub(crate) lines: Vec<TextRenderLine>,
}

#[derive(Clone, Debug)]
pub struct TextRenderLine {
    pub(crate) items: Vec<TextRenderItem>,
    pub(crate) selections: Vec<Rectangle>,
}

#[derive(Clone, Debug)]
pub struct TextRenderItem {
    pub(crate) brush: ColorBrush,
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

pub fn from_editor(layout: &Layout<ColorBrush>) -> TextRender {
    let mut text_render = TextRender { lines: Vec::new() };

    for line in layout.lines() {
        let mut text_render_line = TextRenderLine { items: Vec::new(), selections: Vec::new(), };

        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };

            let style = glyph_run.style();
            // We draw underlines under the text, then the strikethrough on top, following:
            // https://drafts.csswg.org/css-text-decor/#painting-order
            let underline: Option<TextRenderItemLine> = if let Some(underline) = &style.underline {
                let underline_brush = &style.brush;
                let run_metrics = glyph_run.run().metrics();
                let offset = match underline.offset {
                    Some(offset) => offset,
                    None => run_metrics.underline_offset,
                };
                let width = match underline.size {
                    Some(size) => size,
                    None => run_metrics.underline_size,
                };
                // The `offset` is the distance from the baseline to the top of the underline
                // so we move the line down by half the width
                // Remember that we are using a y-down coordinate system
                // If there's a custom width, because this is an underline, we want the custom
                // width to go down from the default expectation
                let y = glyph_run.baseline() - offset + width / 2.;

                let line = Line::new(
                    (glyph_run.offset() as f64, y as f64),
                    ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
                );
                Some(TextRenderItemLine { line, width })
            } else {
                None
            };

            let mut x = glyph_run.offset();
            let y = glyph_run.baseline();
            let run = glyph_run.run();
            let font = run.font();
            let font_size = run.font_size();
            let synthesis = run.synthesis();
            let glyph_xform = synthesis.skew().map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));

            let glyphs = glyph_run.glyphs().map(|glyph| {
                let gx = x + glyph.x;
                let gy = y - glyph.y;
                x += glyph.advance;
                TextRenderGlyph {
                    id: glyph.id,
                    x: gx,
                    y: gy,
                }
            });

            let strikethrough = if let Some(strikethrough) = &style.strikethrough {
                let strikethrough_brush = &style.brush;
                let run_metrics = glyph_run.run().metrics();
                let offset = match strikethrough.offset {
                    Some(offset) => offset,
                    None => run_metrics.strikethrough_offset,
                };
                let width = match strikethrough.size {
                    Some(size) => size,
                    None => run_metrics.strikethrough_size,
                };
                // The `offset` is the distance from the baseline to the *top* of the strikethrough
                // so we calculate the middle y-position of the strikethrough based on the font's
                // standard strikethrough width.
                // Remember that we are using a y-down coordinate system
                let y = glyph_run.baseline() - offset + run_metrics.strikethrough_size / 2.;

                let line = Line::new(
                    (glyph_run.offset() as f64, y as f64),
                    ((glyph_run.offset() + glyph_run.advance()) as f64, y as f64),
                );
                Some(TextRenderItemLine { line, width })
            } else {
                None
            };

            let text_render_item = TextRenderItem {
                brush: style.brush,
                underline,
                strikethrough,
                glyph_transform: glyph_xform,
                font_size,
                glyphs: glyphs.collect(),
                font: font.clone(),
            };

            text_render_line.items.push(text_render_item);
        }
        text_render.lines.push(text_render_line);
    }
    
    text_render
}