use crate::elements::text_input::editor::Editor;
use crate::elements::text_input::plain_text_editor::PlainEditor;
use crate::renderer::color::palette;
use parley::{GlyphRun, Style};
use peniko::Brush;
use vello::{
    kurbo::{Affine, Line, Stroke},
    peniko::Fill,
    Scene,
};

pub(crate) fn draw_cursor(scene: &mut Scene, transform: &Affine, editor: &Editor) {
    if editor.cursor_visible {
        if let Some(cursor) = editor.editor.cursor_geometry(1.0) {
            scene.fill(Fill::NonZero, *transform, palette::css::BLACK, None, &cursor);
        }
    }
}

pub(crate) fn draw_selection(scene: &mut Scene, transform: &Affine, plain_editor: &PlainEditor<Brush>) {
    for rect in plain_editor.selection_geometry().iter() {
        scene.fill(Fill::NonZero, *transform, palette::css::STEEL_BLUE, None, &rect);
    }
}

pub(crate) fn draw_underline(scene: &mut Scene, transform: &Affine, glyph_run: &GlyphRun<Brush>, style: &Style<Brush>) {
    // We draw underlines under the text, then the strikethrough on top, following:
    // https://drafts.csswg.org/css-text-decor/#painting-order
    if let Some(underline) = &style.underline {
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
        scene.stroke(&Stroke::new(width.into()), *transform, underline_brush, None, &line);
    }
}

pub(crate) fn draw_strikethrough(
    scene: &mut Scene,
    transform: &Affine,
    glyph_run: &GlyphRun<Brush>,
    style: &Style<Brush>,
) {
    if let Some(strikethrough) = &style.strikethrough {
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
        scene.stroke(&Stroke::new(width.into()), *transform, strikethrough_brush, None, &line);
    }
}

pub(crate) fn draw_glyphs(scene: &mut Scene, transform: &Affine, glyph_run: &GlyphRun<Brush>, style: &Style<Brush>) {
    let mut x = glyph_run.offset();
    let y = glyph_run.baseline();
    let run = glyph_run.run();
    let font = run.font();
    let font_size = run.font_size();
    let synthesis = run.synthesis();
    let glyph_xform = synthesis.skew().map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));
    scene
        .draw_glyphs(font)
        .brush(&style.brush)
        .hint(true)
        .transform(*transform)
        .glyph_transform(glyph_xform)
        .font_size(font_size)
        .normalized_coords(run.normalized_coords())
        .draw(
            Fill::NonZero,
            glyph_run.glyphs().map(|glyph| {
                let gx = x + glyph.x;
                let gy = y - glyph.y;
                x += glyph.advance;
                vello::Glyph {
                    id: glyph.id as _,
                    x: gx,
                    y: gy,
                }
            }),
        );
}
