use craft_primitives::ColorBrush;
pub(crate) use craft_renderer::text_renderer_data::{TextRender, TextRenderGlyph, TextRenderLine};
use craft_renderer::text_renderer_data::{TextRenderItem, TextRenderItemLine};
use parley::{Layout, PositionedLayoutItem};
use peniko::kurbo::{Affine, Line};

pub fn from_editor(layout: &Layout<ColorBrush>) -> TextRender {
    let mut text_render = TextRender {
        lines: Vec::new(),
        cursor: None,
        override_brush: None,
    };

    for line in layout.lines() {
        let mut text_render_line = TextRenderLine {
            items: Vec::new(),
            selections: Vec::new(),
            backgrounds: Vec::new(),
        };

        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };

            let style = glyph_run.style();
            // We draw underlines under the text, then the strikethrough on top, following:
            // https://drafts.csswg.org/css-text-decor/#painting-order
            let underline: Option<TextRenderItemLine> = if let Some(underline) = &style.underline {
                let underline_brush = underline.brush;
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
                Some(TextRenderItemLine {
                    line,
                    width,
                    brush: underline_brush,
                })
            } else {
                None
            };

            let mut x = glyph_run.offset();
            let y = glyph_run.baseline();
            let run = glyph_run.run();
            let font = run.font();
            let font_size = run.font_size();
            let synthesis = run.synthesis();
            let glyph_xform = synthesis
                .skew()
                .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));

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
                let strikethrough_brush = strikethrough.brush;
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
                Some(TextRenderItemLine {
                    line,
                    width,
                    brush: strikethrough_brush,
                })
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
