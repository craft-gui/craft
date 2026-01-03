use peniko::color::AlphaColor;
use peniko::kurbo::{BezPath, SvgArc};
use peniko::{Color, Gradient, kurbo};
use tinyvg_rs::color_table::ColorTable;
use tinyvg_rs::commands::{Path, PathCommand, Point, Style};

use crate::renderer::Brush;

#[allow(clippy::wrong_self_convention)]
/// Convert the TinyVG point to a kurbo color.
pub(crate) fn to_kurbo_point(point: Point) -> kurbo::Point {
    kurbo::Point::new(point.x.0, point.y.0)
}

#[allow(clippy::wrong_self_convention)]
/// Convert the TinyVG color to a peniko color.
pub(crate) fn to_peniko_color(color: tinyvg_rs::color_table::RgbaF32) -> Color {
    Color::from(AlphaColor::new([color.0, color.1, color.2, color.3]))
}

/// Assemble a kurbo bez path from a TinyVG path.
pub(crate) fn assemble_path(
    path: &Path,
    fill_style: &Style,
    color_table: &ColorTable,
    override_color: &Option<Color>,
) -> (BezPath, Brush) {
    let brush = get_brush(fill_style, color_table, override_color);
    let mut bezier_path = BezPath::new();

    for segment in &path.segments {
        let mut current = segment.start;
        bezier_path.move_to(to_kurbo_point(current));

        for path_command in &segment.path_commands {
            match path_command {
                PathCommand::Line(point, _line_width) => {
                    bezier_path.line_to(to_kurbo_point(*point));
                    current = current.move_to(point);
                }
                PathCommand::HorizontalLine(horizontal, _line_width) => {
                    let horizontal_end_point = Point {
                        x: *horizontal,
                        y: current.y,
                    };
                    bezier_path.line_to(to_kurbo_point(horizontal_end_point));
                    current = current.move_to(&horizontal_end_point);
                }
                PathCommand::VerticalLine(vertical, _line_width) => {
                    let vertical_end_point = Point {
                        x: current.x,
                        y: *vertical,
                    };
                    bezier_path.line_to(to_kurbo_point(vertical_end_point));
                    current = current.move_to(&vertical_end_point);
                }
                PathCommand::CubicBezier(cubic_bezier, _line_width) => {
                    let end = cubic_bezier.point_1;
                    bezier_path.curve_to(
                        (cubic_bezier.control_point_0.x.0, cubic_bezier.control_point_0.y.0),
                        (cubic_bezier.control_point_1.x.0, cubic_bezier.control_point_1.y.0),
                        (end.x.0, end.y.0),
                    );
                    current = current.move_to(&end);
                }
                PathCommand::ArcCircle(arc_circle, _line_width) => {
                    let arc_start = to_kurbo_point(current);
                    let arc_end = to_kurbo_point(arc_circle.target);

                    let arc = SvgArc {
                        from: arc_start,
                        to: arc_end,
                        radii: kurbo::Vec2::new(arc_circle.radius.0, arc_circle.radius.0),
                        x_rotation: 0.0,
                        large_arc: arc_circle.large_arc,
                        sweep: arc_circle.sweep,
                    };

                    let arc = kurbo::Arc::from_svg_arc(&arc);
                    if let Some(arc) = arc {
                        for el in arc.append_iter(0.1) {
                            bezier_path.push(el);
                        }
                    }

                    current = current.move_to(&arc_circle.target);
                }
                PathCommand::ArcEllipse(arc_ellipse, _line_width) => {
                    let arc_start = to_kurbo_point(current);
                    let arc_end = to_kurbo_point(arc_ellipse.target);

                    let arc = SvgArc {
                        from: arc_start,
                        to: arc_end,
                        radii: kurbo::Vec2::new(arc_ellipse.radius_x.0, arc_ellipse.radius_y.0),
                        x_rotation: 0.0,
                        large_arc: arc_ellipse.large_arc,
                        sweep: arc_ellipse.sweep,
                    };

                    let arc = kurbo::Arc::from_svg_arc(&arc);
                    if let Some(arc) = arc {
                        for el in arc.append_iter(0.1) {
                            bezier_path.push(el);
                        }
                    }
                    current = current.move_to(&arc_ellipse.target);
                }
                PathCommand::ClosePath => {
                    bezier_path.close_path();
                }
                PathCommand::QuadraticBezier(quadratic_bezier, _line_width) => {
                    let end = quadratic_bezier.point_1;
                    bezier_path.quad_to(
                        (
                            to_kurbo_point(quadratic_bezier.control_point).x,
                            to_kurbo_point(quadratic_bezier.control_point).y,
                        ),
                        (to_kurbo_point(end).x, to_kurbo_point(end).y),
                    );

                    current = current.move_to(&end);
                }
            }
        }
    }

    (bezier_path, brush)
}

/// Convert the TinyVG style to a Brush.
pub(crate) fn get_brush(fill_style: &Style, color_table: &ColorTable, override_color: &Option<Color>) -> Brush {
    if let Some(override_color) = override_color {
        return Brush::Color(*override_color);
    }

    match fill_style {
        Style::FlatColor(flat_colored) => {
            let color = color_table[flat_colored.color_index as usize];
            Brush::Color(to_peniko_color(color))
        }
        Style::LinearGradient(linear_gradient) => {
            let color_0 = color_table[linear_gradient.color_index_0 as usize];
            let color_1 = color_table[linear_gradient.color_index_1 as usize];

            let start = to_kurbo_point(linear_gradient.point_0);
            let end = to_kurbo_point(linear_gradient.point_1);

            let linear =
                Gradient::new_linear(start, end).with_stops([to_peniko_color(color_0), to_peniko_color(color_1)]);
            Brush::Gradient(linear)
        }
        Style::RadialGradient(radial_gradient) => {
            let color_0 = color_table[radial_gradient.color_index_0 as usize];
            let color_1 = color_table[radial_gradient.color_index_1 as usize];

            let center = to_kurbo_point(radial_gradient.point_0);
            let edge = to_kurbo_point(radial_gradient.point_1);
            let radius = center.distance(edge);

            let radial = Gradient::new_radial(center, radius as f32)
                .with_stops([to_peniko_color(color_0), to_peniko_color(color_1)]);

            Brush::Gradient(radial)
        }
    }
}
