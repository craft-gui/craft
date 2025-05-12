use crate::geometry::Rectangle;
use crate::renderer::tinyvg_helpers::TinyVgHelpers;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use peniko::kurbo::{Affine, Line, Stroke};
use peniko::{kurbo, Color, Fill};
use std::sync::Arc;
use tinyvg_rs::color_table::ColorTable;
use tinyvg_rs::commands::{DrawCommand, Path, PathCommand, Segment, Style};
use tinyvg_rs::common::Unit;
use vello::Scene;

pub(crate) fn draw_path(scene: &mut Scene, path: &Path, fill_style: &Style, line_width: Option<&Unit>, color_table: &ColorTable, affine: &Affine, override_color: &Option<Color>) {
    let (bezier_path, brush) = TinyVgHelpers::assemble_path(path, fill_style, color_table, override_color);
    
    if let Some(line_width) = line_width {
        scene.stroke(
            &Stroke::new(line_width.0),
            *affine,
            &brush,
            None,
            &bezier_path,
        );
    } else {
        scene.fill(
            Fill::EvenOdd,
            *affine,
            &brush,
            None,
            &bezier_path,
        );
    }
}

pub(crate) fn draw_tiny_vg(scene: &mut Scene, rectangle: Rectangle, resource_manager: Arc<ResourceManager>, resource_identifier: ResourceIdentifier, override_color: &Option<Color>) {
    let resource = resource_manager.resources.get(&resource_identifier);
    if let Some(resource) = resource {
    if let Resource::TinyVg(resource) = resource.as_ref() {
        if resource.tinyvg.is_none() {
            return;
        }
        let tiny_vg = resource.tinyvg.as_ref().unwrap();

        let mut affine = Affine::IDENTITY;
        let mut svg_width = tiny_vg.header.width as f32;
        let mut svg_height = tiny_vg.header.height as f32;
        
        // The svg size could be 0 which means infinite size, so we'll just set it to the resolved width and height that Taffy gives us.
        if tiny_vg.header.width == 0 {
            svg_width = rectangle.width;
        }
        if tiny_vg.header.height == 0 {
            svg_height = rectangle.height;
        }
        
        affine = affine.with_translation(kurbo::Vec2::new(rectangle.x as f64, rectangle.y as f64));
        affine = affine.pre_scale_non_uniform(
            rectangle.width as f64 / svg_width as f64,
            rectangle.height as f64 / svg_height as f64,
        );
        
        for command in &tiny_vg.draw_commands {
            match command {
                DrawCommand::FillPolygon(data) => {
                    let start = data.points[0];
                    let mut segment = Segment {
                        start,
                        path_commands: vec![],
                    };
                    for point in &data.points {
                        segment.path_commands.push(PathCommand::Line(*point, None));
                    }
                    segment.path_commands.push(PathCommand::ClosePath);
                    let path = Path {
                        segments: vec![segment],
                    };
                    draw_path(scene, &path, &data.style, None, &tiny_vg.color_table, &affine, override_color);
                }
                DrawCommand::FillRectangles(data) => {
                    let brush = TinyVgHelpers::get_brush(&data.style, &tiny_vg.color_table, override_color);
                    for rectangle in &data.rectangles {
                        let rectangle = kurbo::Rect::new(rectangle.x.0, rectangle.y.0, rectangle.height.0, rectangle.height.0);
                        scene.fill(Fill::EvenOdd, affine, &brush, None, &rectangle);
                    }
                }
                DrawCommand::FillPath(data) => {
                    draw_path(scene, &data.path, &data.style, None, &tiny_vg.color_table, &affine, override_color);
                }
                DrawCommand::DrawLines(data) => {
                    let brush = TinyVgHelpers::get_brush(&data.line_style, &tiny_vg.color_table, override_color);

                    for line in &data.lines {
                        let line = Line::new(TinyVgHelpers::to_kurbo_point(line.start), TinyVgHelpers::to_kurbo_point(line.end));
                        scene.stroke(&Stroke::new(data.line_width.0), affine, &brush, None, &line);
                    }
                }
                DrawCommand::DrawLineLoop(data) => {
                    let brush = TinyVgHelpers::get_brush(&data.line_style, &tiny_vg.color_table, override_color);

                    let mut start = data.points[0];
                    for point in &data.points {
                        let line = Line::new(TinyVgHelpers::to_kurbo_point(start), TinyVgHelpers::to_kurbo_point(*point));
                        scene.stroke(&Stroke::new(data.line_width.0), affine, &brush, None, &line);
                        start = *point;
                    }
                }
                DrawCommand::DrawLineStrip(data) => {
                    let brush = TinyVgHelpers::get_brush(&data.style, &tiny_vg.color_table, override_color);

                    let mut start = data.points[0];
                    for point in &data.points {
                        let line = Line::new(TinyVgHelpers::to_kurbo_point(start), TinyVgHelpers::to_kurbo_point(*point));
                        scene.stroke(&Stroke::new(data.line_width.0), affine, &brush, None, &line);
                        start = *point;
                    }
                }
                DrawCommand::DrawLinePath(data) => {
                    draw_path(scene, &data.path, &data.style, Some(&data.line_width), &tiny_vg.color_table, &affine, override_color);
                }
                DrawCommand::OutlineFillPolygon(data) => {
                    let start = data.points[0];
                    let mut segment = Segment {
                        start,
                        path_commands: vec![],
                    };
                    for point in &data.points {
                        segment.path_commands.push(PathCommand::Line(*point, None));
                    }
                    segment.path_commands.push(PathCommand::ClosePath);
                    let path = Path {
                        segments: vec![segment],
                    };
                    draw_path(scene, &path, &data.fill_style, None, &tiny_vg.color_table, &affine, override_color);
                    draw_path(scene, &path, &data.line_style, Some(&data.line_width), &tiny_vg.color_table, &affine, override_color);
                }
                DrawCommand::OutlineFillRectangles(data) => {
                    let fill_brush = TinyVgHelpers::get_brush(&data.fill_style, &tiny_vg.color_table, override_color);
                    let line_brush = TinyVgHelpers::get_brush(&data.line_style, &tiny_vg.color_table, override_color);
                    for rectangle in &data.rectangles {
                        let rectangle = kurbo::Rect::new(rectangle.x.0, rectangle.y.0, rectangle.height.0, rectangle.height.0);
                        scene.fill(Fill::EvenOdd, affine, &fill_brush, None, &rectangle);
                        scene.stroke(&Stroke::new(data.line_width.0), affine, &line_brush, None, &rectangle);
                    }
                }
                DrawCommand::OutlineFillPath(data) => {
                    draw_path(scene, &data.path, &data.fill_style, None, &tiny_vg.color_table, &affine, override_color);
                    draw_path(scene, &data.path, &data.line_style, Some(&data.line_width), &tiny_vg.color_table, &affine, override_color);
                },
                // This command only provides metadata for accessibility or text selection tools for the position and content
                // of text. A renderer can safely ignore this command since it must not have any effect on the resulting
                // graphic
                DrawCommand::TextHint(_data) => {}
            }
        }
    }
    }
}