use std::cmp;
use std::sync::Arc;
use cosmic_text::{Buffer, Cursor, Edit, Editor, LayoutRun};
use cosmic_text::fontdb::ID;
use unicode_segmentation::UnicodeSegmentation;
use vello::Glyph;
use vello::kurbo::{Point, Rect, Size};
use vello::peniko::Color;

pub(crate) struct CosmicFontBlobAdapter {
    font: Arc<cosmic_text::Font>,
}

/// Adapter to allow `cosmic_text::Font` to be used as a Blob.
impl CosmicFontBlobAdapter {
    pub(crate) fn new(font: Arc<cosmic_text::Font>) -> Self {
        Self { font }
    }
}

impl AsRef<[u8]> for CosmicFontBlobAdapter {
    fn as_ref(&self) -> &[u8] {
        self.font.data()
    }
}

pub(crate) struct BufferGlyphs {
    pub(crate) font_size: f32,
    pub(crate) glyph_highlight_color: Color,
    pub(crate) cursor_color: Color,
    pub(crate) buffer_lines: Vec<BufferLine>,
}

pub(crate) struct BufferLine {
    pub(crate) glyph_highlights: Vec<Rect>,
    pub(crate) cursor: Option<Rect>,
    pub(crate) glyph_runs: Vec<BufferGlyphRun>,
}

pub(crate) struct BufferGlyphRun {
    pub(crate) font: ID,
    pub(crate) glyphs: Vec<Glyph>,
    pub(crate) glyph_color: Color,
}

pub(crate) struct EditorInfo {
    cursor_color: Color,
    selection_color: Color,
    selected_text_color: Color,
    selection_bounds: Option<(Cursor, Cursor)>,
    cursor: Cursor,
}

impl EditorInfo {
    fn new(
        editor: &Editor,
        cursor_color: Color,
        selection_color: Color,
        selected_text_color: Color,
    ) -> Self {
        Self {
            cursor_color,
            selection_color,
            selected_text_color,
            selection_bounds: editor.selection_bounds(),
            cursor: editor.cursor(),
        }
    }
}

pub(crate) fn create_glyphs_for_editor(
    buffer: &Buffer,
    editor: &Editor,
    text_color: Color,
    cursor_color: Color,
    selection_color: Color,
    selected_text_color: Color,
) -> BufferGlyphs {
    create_glyphs(
        buffer,
        text_color,
        Some(EditorInfo::new(
            editor,
            cursor_color,
            selection_color,
            selected_text_color,
        )),
    )
}

pub(crate) fn create_glyphs(
    buffer: &Buffer,
    text_color: Color,
    editor_info: Option<EditorInfo>,
) -> BufferGlyphs {
    // Get the laid out glyphs and convert them to Glyphs for vello

    let mut last_font: Option<(ID, Color)> = None;

    let mut buffer_glyphs = BufferGlyphs {
        font_size: buffer.metrics().font_size,
        glyph_highlight_color: Color::WHITE,
        cursor_color: Color::BLACK,
        buffer_lines: vec![],
    };

    if let Some(editor_info) = &editor_info {
        buffer_glyphs.cursor_color = editor_info.cursor_color;
        buffer_glyphs.glyph_highlight_color = editor_info.selection_color;
    }

    for layout_run in buffer.layout_runs() {
        let mut current_glyphs: Vec<Glyph> = vec![];
        let line_i = layout_run.line_i;
        let line_y = layout_run.line_y as f64;
        let line_top = layout_run.line_top as f64;
        let line_height = layout_run.line_height as f64;

        let mut buffer_line = BufferLine {
            glyph_highlights: vec![],
            cursor: None,
            glyph_runs: vec![],
        };

        if let Some(editor_info) = &editor_info {
            // Highlight selection
            if let Some((start, end)) = editor_info.selection_bounds {
                if line_i >= start.line && line_i <= end.line {
                    let mut range_opt = None;
                    for glyph in layout_run.glyphs.iter() {
                        // Guess x offset based on characters
                        let cluster = &layout_run.text[glyph.start..glyph.end];
                        let total = cluster.grapheme_indices(true).count();
                        let mut c_x = glyph.x;
                        let c_w = glyph.w / total as f32;
                        for (i, c) in cluster.grapheme_indices(true) {
                            let c_start = glyph.start + i;
                            let c_end = glyph.start + i + c.len();
                            if (start.line != line_i || c_end > start.index)
                                && (end.line != line_i || c_start < end.index)
                            {
                                range_opt = match range_opt.take() {
                                    Some((min, max)) => Some((
                                        cmp::min(min, c_x as i32),
                                        cmp::max(max, (c_x + c_w) as i32),
                                    )),
                                    None => Some((c_x as i32, (c_x + c_w) as i32)),
                                };
                            } else if let Some((min, max)) = range_opt.take() {
                                buffer_line.glyph_highlights.push(Rect::from_origin_size(
                                    Point::new(min as f64, line_top),
                                    Size::new(cmp::max(0, max - min) as f64, line_height),
                                ));
                            }
                            c_x += c_w;
                        }
                    }

                    if layout_run.glyphs.is_empty() && end.line > line_i {
                        // Highlight all internal empty lines
                        range_opt = Some((0, buffer.size().0.unwrap_or(0.0) as i32));
                    }

                    if let Some((mut min, mut max)) = range_opt.take() {
                        if end.line > line_i {
                            // Draw to end of line
                            if layout_run.rtl {
                                min = 0;
                            } else {
                                max = buffer.size().0.unwrap_or(0.0) as i32;
                            }
                        }
                        buffer_line.glyph_highlights.push(Rect::from_origin_size(
                            Point::new(min as f64, line_top),
                            Size::new(cmp::max(0, max - min) as f64, line_height),
                        ));
                    }
                }
            }

            // Cursor
            if let Some((x, y)) = cursor_position(&editor_info.cursor, &layout_run) {
                buffer_line.cursor = Some(Rect::from_origin_size(
                    Point::new(x as f64, y as f64),
                    Size::new(1.0, line_height),
                ));
            }
        }

        for glyph in layout_run.glyphs {
            let mut glyph_color = match glyph.color_opt {
                Some(color) => Color::from_rgba8(color.r(), color.g(), color.b(), color.a()),
                None => text_color,
            };

            if let Some(editor_info) = &editor_info {
                if text_color != editor_info.selected_text_color {
                    if let Some((start, end)) = editor_info.selection_bounds {
                        if line_i >= start.line
                            && line_i <= end.line
                            && (start.line != line_i || glyph.end > start.index)
                            && (end.line != line_i || glyph.start < end.index)
                        {
                            glyph_color = editor_info.selected_text_color;
                        }
                    }
                }
            }

            if let Some((last_font, last_glyph_color)) = last_font {
                if last_font != glyph.font_id || last_glyph_color != glyph_color {
                    buffer_line.glyph_runs.push(BufferGlyphRun {
                        font: last_font,
                        glyphs: current_glyphs,
                        glyph_color: last_glyph_color,
                    });
                    current_glyphs = vec![];
                }
            }

            last_font = Some((glyph.font_id, glyph_color));
            current_glyphs.push(Glyph {
                x: glyph.x,
                y: glyph.y + line_y as f32,
                id: glyph.glyph_id as u32,
            });
        }
        if !current_glyphs.is_empty() {
            let (last_font, last_color) = last_font.unwrap();
            buffer_line.glyph_runs.push(BufferGlyphRun {
                font: last_font,
                glyphs: current_glyphs,
                glyph_color: last_color,
            });
        }

        buffer_glyphs.buffer_lines.push(buffer_line);
    }

    buffer_glyphs
}

// Copied directly from cosmic_text.
fn cursor_position(cursor: &Cursor, run: &LayoutRun) -> Option<(i32, i32)> {
    let (cursor_glyph, cursor_glyph_offset) = cursor_glyph_opt(cursor, run)?;
    let x = match run.glyphs.get(cursor_glyph) {
        Some(glyph) => {
            // Start of detected glyph
            if glyph.level.is_rtl() {
                (glyph.x + glyph.w - cursor_glyph_offset) as i32
            } else {
                (glyph.x + cursor_glyph_offset) as i32
            }
        }
        None => match run.glyphs.last() {
            Some(glyph) => {
                // End of last glyph
                if glyph.level.is_rtl() {
                    glyph.x as i32
                } else {
                    (glyph.x + glyph.w) as i32
                }
            }
            None => {
                // Start of empty line
                0
            }
        },
    };

    Some((x, run.line_top as i32))
}

// Copied directly from cosmic_text.
fn cursor_glyph_opt(cursor: &Cursor, run: &LayoutRun) -> Option<(usize, f32)> {
    if cursor.line == run.line_i {
        for (glyph_i, glyph) in run.glyphs.iter().enumerate() {
            if cursor.index == glyph.start {
                return Some((glyph_i, 0.0));
            } else if cursor.index > glyph.start && cursor.index < glyph.end {
                // Guess x offset based on characters
                let mut before = 0;
                let mut total = 0;

                let cluster = &run.text[glyph.start..glyph.end];
                for (i, _) in cluster.grapheme_indices(true) {
                    if glyph.start + i < cursor.index {
                        before += 1;
                    }
                    total += 1;
                }

                let offset = glyph.w * (before as f32) / (total as f32);
                return Some((glyph_i, offset));
            }
        }
        match run.glyphs.last() {
            Some(glyph) => {
                if cursor.index == glyph.end {
                    return Some((run.glyphs.len(), 0.0));
                }
            }
            None => {
                return Some((0, 0.0));
            }
        }
    }
    None
}