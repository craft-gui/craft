use glifo::Glyph;

use kurbo::{Affine, Stroke};

use peniko::kurbo::Shape;

use retgui_primitives::geometry::Rectangle;

use vello_common::{kurbo, peniko};

use vello_cpu::{RenderContext, Resources};

use crate::helpers::{brush_to_paint, text_bounds};
use crate::render_command::{DrawRectCmd, DrawTextCmd};
use crate::vello_cpu::draw_rect;

pub(crate) fn draw_text(cmd: &DrawTextCmd, scene: &mut RenderContext, resources: &mut Resources, window: &Rectangle) {
    let text_container = Rectangle::from_kurbo(cmd.transform.transform_rect_bbox(cmd.rect.to_kurbo()));
    let scroll = cmd.text_scroll.unwrap_or_default().scroll_y;
    let text_transform = Affine::default()
        .with_translation(kurbo::Vec2::new(cmd.rect.x as f64, cmd.rect.y as f64))
        .then_translate(kurbo::Vec2::new(0.0, -scroll as f64));

    let text_data = &cmd.data;
    let use_glyph_cache = text_data.use_glyph_cache();
    let text_render = text_data.get_text_renderer().expect("Text render not found");
    let override_brush = text_data.override_brush();

    let text_paint_bounds = text_bounds(&text_render.lines).unwrap_or_else(|| Rectangle::new(0.0, 0.0, 0.0, 0.0));

    for line in &text_render.lines {
        let scrolled_text_container_y = text_container.y - scroll;
        let line_top = scrolled_text_container_y + line.min_y;
        let line_bottom = scrolled_text_container_y + line.max_y;

        if line_bottom < window.y {
            continue;
        }
        if line_top > window.y + window.height {
            break;
        }

        // Draw background and selection
        for (background, color) in &line.backgrounds {
            let background_rect = Rectangle {
                x: background.x + cmd.rect.x,
                y: -scroll + background.y + cmd.rect.y,
                width: background.width,
                height: background.height,
            };
            draw_rect(
                scene,
                &DrawRectCmd {
                    rect: background_rect,
                    brush: color.clone(),
                    transform: cmd.transform,
                },
            );
        }

        for (selection, selection_color) in &line.selections {
            let selection_rect = Rectangle {
                x: selection.x + cmd.rect.x,
                y: -scroll + selection.y + cmd.rect.y,
                width: selection.width,
                height: selection.height,
            };
            draw_rect(
                scene,
                &DrawRectCmd {
                    rect: selection_rect,
                    brush: selection_color.clone(),
                    transform: cmd.transform,
                },
            );
        }

        scene.set_transform(cmd.transform * text_transform);
        scene.reset_paint_transform();

        // Draw the text
        for item in &line.items {
            if let Some(underline) = &item.underline {
                scene.set_stroke(Stroke::new(underline.width.into()));
                scene.set_paint(brush_to_paint(text_paint_bounds, &underline.brush));
                scene.stroke_path(&underline.line.to_path(0.1));
            }

            let brush = override_brush.unwrap_or(&item.brush);
            scene.set_paint(brush_to_paint(text_paint_bounds, brush));

            let glyph_run_builder = scene
                .glyph_run(resources, &item.font)
                .atlas_cache(use_glyph_cache)
                .font_size(item.font_size)
                .normalized_coords(&item.normalized_coords);
            glyph_run_builder.fill_glyphs(item.glyphs.iter().map(|glyph| Glyph {
                id: glyph.id,
                x: glyph.x,
                y: glyph.y,
            }));
        }
    }

    // Draw the cursor
    if cmd.show_cursor
        && let Some((cursor, cursor_brush)) = &text_render.cursor
    {
        let cursor_rect = Rectangle {
            x: cursor.x + cmd.rect.x,
            y: -scroll + cursor.y + cmd.rect.y,
            width: cursor.width,
            height: cursor.height,
        };
        draw_rect(
            scene,
            &DrawRectCmd {
                rect: cursor_rect,
                brush: cursor_brush.clone(),
                transform: cmd.transform,
            },
        );
    }
}
