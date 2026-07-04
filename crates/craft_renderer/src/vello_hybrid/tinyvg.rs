use std::sync::Arc;

use craft_primitives::Color;
use craft_primitives::geometry::Rectangle;
use craft_resource_manager::resource::Resource;
use craft_resource_manager::{ResourceId, ResourceManager};

use peniko::kurbo::{Affine, BezPath, Stroke};
use peniko::{Fill, kurbo};

use tinyvg_rs::color_table::ColorTable;
use tinyvg_rs::commands::{DrawCommand, Path, Style};
use tinyvg_rs::common::Unit;

use vello_hybrid::Scene;

use crate::vello_hybrid::brush_to_paint;
use crate::{Brush, tinyvg_helpers};

fn stroke_path(scene: &mut Scene, bez_path: &BezPath, line_width: f64, brush: &Brush) {
    scene.set_stroke(Stroke::new(line_width));
    scene.set_paint(brush_to_paint(brush));
    scene.stroke_path(bez_path);
}

fn fill_path(scene: &mut Scene, bez_path: &BezPath, brush: &Brush) {
    scene.set_paint(brush_to_paint(brush));
    scene.set_fill_rule(Fill::EvenOdd);
    scene.fill_path(bez_path);
}

pub(crate) fn draw_path(
    scene: &mut Scene,
    path: &Path,
    fill_style: &Style,
    line_width: Option<&Unit>,
    color_table: &ColorTable,
    override_color: &Option<Color>,
) {
    let (bezier_path, brush) = tinyvg_helpers::assemble_path(path, fill_style, color_table, override_color);

    if let Some(line_width) = line_width {
        stroke_path(scene, &bezier_path, line_width.0, &brush);
    } else {
        fill_path(scene, &bezier_path, &brush);
    }
}

pub(crate) fn draw_tiny_vg(
    scene: &mut Scene,
    rectangle: Rectangle,
    resource_manager: &Arc<ResourceManager>,
    resource_id: ResourceId,
    override_color: &Option<Color>,
) {
    let resource = resource_manager.get(&resource_id);
    if resource.is_none() {
        return;
    }
    let resource = resource.unwrap();

    if let Resource::TinyVg(resource) = resource.as_ref() {
        if resource.tinyvg.is_none() {
            return;
        }
        let tiny_vg = resource.tinyvg.as_ref().unwrap();

        let scene_state = scene.save_current_state();

        let vg_transform = Affine::IDENTITY;
        let mut svg_width = tiny_vg.header.width as f32;
        let mut svg_height = tiny_vg.header.height as f32;

        // The svg size could be 0 which means infinite size, so we'll just set it to the resolved width and height that Taffy gives us.
        if tiny_vg.header.width == 0 {
            svg_width = rectangle.width;
        }
        if tiny_vg.header.height == 0 {
            svg_height = rectangle.height;
        }

        let vg_transform = vg_transform.with_translation(kurbo::Vec2::new(rectangle.x as f64, rectangle.y as f64));
        let vg_transform = vg_transform.pre_scale_non_uniform(
            rectangle.width as f64 / svg_width as f64,
            rectangle.height as f64 / svg_height as f64,
        );

        scene.set_transform(scene_state.transform * vg_transform);

        for command in &tiny_vg.draw_commands {
            match command {
                DrawCommand::FillPolygon(data) => {
                    let mut path = BezPath::new();
                    if let Some(first) = data.points.first() {
                        path.move_to(tinyvg_helpers::to_kurbo_point(*first));
                        for point in data.points.iter().skip(1) {
                            path.line_to(tinyvg_helpers::to_kurbo_point(*point));
                        }
                        path.close_path();
                    }
                    let brush = tinyvg_helpers::get_brush(&data.style, &tiny_vg.color_table, override_color);
                    fill_path(scene, &path, &brush);
                }
                DrawCommand::FillRectangles(data) => {
                    let brush = tinyvg_helpers::get_brush(&data.style, &tiny_vg.color_table, override_color);
                    scene.set_paint(brush_to_paint(&brush));
                    scene.set_fill_rule(Fill::EvenOdd);

                    for rectangle in &data.rectangles {
                        let rect =
                            kurbo::Rect::new(rectangle.x.0, rectangle.y.0, rectangle.width.0, rectangle.height.0);
                        scene.fill_rect(&rect);
                    }
                }
                DrawCommand::FillPath(data) => {
                    draw_path(
                        scene,
                        &data.path,
                        &data.style,
                        None,
                        &tiny_vg.color_table,
                        override_color,
                    );
                }
                DrawCommand::DrawLines(data) => {
                    let brush = tinyvg_helpers::get_brush(&data.line_style, &tiny_vg.color_table, override_color);
                    let mut path = BezPath::new();

                    for line in &data.lines {
                        path.move_to(tinyvg_helpers::to_kurbo_point(line.start));
                        path.line_to(tinyvg_helpers::to_kurbo_point(line.end));
                    }

                    stroke_path(scene, &path, data.line_width.0, &brush);
                }
                DrawCommand::DrawLineLoop(data) => {
                    let brush = tinyvg_helpers::get_brush(&data.line_style, &tiny_vg.color_table, override_color);
                    let mut path = BezPath::new();

                    if let Some(first) = data.points.first() {
                        path.move_to(tinyvg_helpers::to_kurbo_point(*first));
                        for point in data.points.iter().skip(1) {
                            path.line_to(tinyvg_helpers::to_kurbo_point(*point));
                        }
                        path.close_path();
                    }

                    stroke_path(scene, &path, data.line_width.0, &brush);
                }
                DrawCommand::DrawLineStrip(data) => {
                    let brush = tinyvg_helpers::get_brush(&data.style, &tiny_vg.color_table, override_color);
                    let mut path = BezPath::new();

                    if let Some(first) = data.points.first() {
                        path.move_to(tinyvg_helpers::to_kurbo_point(*first));
                        for point in data.points.iter().skip(1) {
                            path.line_to(tinyvg_helpers::to_kurbo_point(*point));
                        }
                    }

                    stroke_path(scene, &path, data.line_width.0, &brush);
                }
                DrawCommand::DrawLinePath(data) => {
                    draw_path(
                        scene,
                        &data.path,
                        &data.style,
                        Some(&data.line_width),
                        &tiny_vg.color_table,
                        override_color,
                    );
                }
                DrawCommand::OutlineFillPolygon(data) => {
                    let mut path = BezPath::new();
                    if let Some(first) = data.points.first() {
                        path.move_to(tinyvg_helpers::to_kurbo_point(*first));
                        for point in data.points.iter().skip(1) {
                            path.line_to(tinyvg_helpers::to_kurbo_point(*point));
                        }
                        path.close_path();
                    }

                    let fill_brush = tinyvg_helpers::get_brush(&data.fill_style, &tiny_vg.color_table, override_color);
                    fill_path(scene, &path, &fill_brush);

                    let line_brush = tinyvg_helpers::get_brush(&data.line_style, &tiny_vg.color_table, override_color);
                    stroke_path(scene, &path, data.line_width.0, &line_brush);
                }
                DrawCommand::OutlineFillRectangles(data) => {
                    let fill_brush = tinyvg_helpers::get_brush(&data.fill_style, &tiny_vg.color_table, override_color);
                    let line_brush = tinyvg_helpers::get_brush(&data.line_style, &tiny_vg.color_table, override_color);

                    for rectangle in &data.rectangles {
                        let rect =
                            kurbo::Rect::new(rectangle.x.0, rectangle.y.0, rectangle.width.0, rectangle.height.0);
                        scene.set_paint(brush_to_paint(&fill_brush));
                        scene.set_fill_rule(Fill::EvenOdd);
                        scene.fill_rect(&rect);

                        scene.set_stroke(Stroke::new(data.line_width.0));
                        scene.set_paint(brush_to_paint(&line_brush));
                        scene.stroke_rect(&rect);
                    }
                }
                DrawCommand::OutlineFillPath(data) => {
                    draw_path(
                        scene,
                        &data.path,
                        &data.fill_style,
                        None,
                        &tiny_vg.color_table,
                        override_color,
                    );
                    draw_path(
                        scene,
                        &data.path,
                        &data.line_style,
                        Some(&data.line_width),
                        &tiny_vg.color_table,
                        override_color,
                    );
                }
                // This command only provides metadata for accessibility or text selection tools for the position and content
                // of text. A renderer can safely ignore this command since it must not have any effect on the resulting
                // graphic
                DrawCommand::TextHint(_data) => {}
            }
        }

        scene.restore_state(scene_state);
    }
}
