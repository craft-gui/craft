use crate::geometry::corner::Corner;
use crate::renderer::color::Color;

use peniko::kurbo::{BezPath, PathEl, Point, Shape, Vec2};

use crate::geometry::cornerside::CornerSide;
use crate::geometry::side::Side;
use crate::geometry::{Rectangle, TrblRectangle};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

pub struct BorderSpec {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    top_left_radius_x: f64,
    top_left_radius_y: f64,
    top_right_radius_x: f64,
    top_right_radius_y: f64,
    bottom_right_radius_x: f64,
    bottom_right_radius_y: f64,
    bottom_left_radius_x: f64,
    bottom_left_radius_y: f64,

    top_width: f64,
    right_width: f64,
    bottom_width: f64,
    left_width: f64,

    top_color: Color,
    right_color: Color,
    bottom_color: Color,
    left_color: Color,
}

impl BorderSpec {
    pub fn new(rect: Rectangle, widths: [f32; 4], radii: [(f32, f32); 4], colors: TrblRectangle<Color>) -> Self {
        Self {
            x1: rect.x as f64,
            y1: rect.y as f64,
            x2: (rect.x + rect.width) as f64,
            y2: (rect.y + rect.height) as f64,
            top_left_radius_x: radii[0].0 as f64,
            top_left_radius_y: radii[0].1 as f64,
            top_right_radius_x: radii[1].0 as f64,
            top_right_radius_y: radii[1].1 as f64,
            bottom_right_radius_x: radii[2].0 as f64,
            bottom_right_radius_y: radii[2].1 as f64,
            bottom_left_radius_x: radii[3].0 as f64,
            bottom_left_radius_y: radii[3].1 as f64,
            top_width: widths[0] as f64,
            right_width: widths[1] as f64,
            bottom_width: widths[2] as f64,
            left_width: widths[3] as f64,
            top_color: colors.top,
            right_color: colors.right,
            bottom_color: colors.bottom,
            left_color: colors.left,
        }
    }

    pub fn compute_border_spec(&self) -> ComputedBorderSpec {
        ComputedBorderSpec::new(self)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ComputedBorderSpec {
    sides: [SideData; 4],
    computed_corner: [ComputedCorner; 4],
}

#[derive(Copy, Clone, Debug)]
pub struct SideData {
    width: f64,
    pub(crate) color: Color,
}

impl Default for SideData {
    fn default() -> Self {
        Self {
            width: 0.0,
            color: Color::BLACK,
        }
    }
}

impl SideData {
    fn new(width: f64, color: Color) -> Self {
        Self { width, color }
    }
}

#[derive(Clone, Debug, Default)]
struct ComputedCorner {
    inner_transition_point: Point,
    outer_transition_point: Point,
    is_inner_sharp: bool,
    is_outer_sharp: bool,

    inner_sides: [ComputedCornerSide; 2],
    outer_sides: [ComputedCornerSide; 2],

    border_point: Point,
    corner_point: Point,
}

#[derive(Debug, Default, Clone)]
struct ComputedCornerSide {
    arc: BezPath,
}

impl ComputedCornerSide {
    fn new() -> Self {
        Self {
            arc: BezPath::new(),
        }
    }
}

/// Calculate the angle of intersection between the quarter-ellipse and a line from the border point to the corner point.
fn intersect_angle(top_left_radius_x: f64, top_left_radius_y: f64, top_width: f64, right_width: f64) -> f64 {
    // 1. Set up the equations and solve for the intersection point using the quadratic formula.
    // 2. To get the angle, we use the parametric equation of the ellipse, solving for the angle.

    // https://www.desmos.com/calculator/l9yg7tsvfz

    let r_x = top_left_radius_x;
    let r_y = top_left_radius_y;

    // If one of the vertical borders has a width of 0, the intersection line will be vertical.
    // Return 0 degrees, which will be a rectangular corner.
    if right_width == 0.0 {
        return 0.0;
    }

    let b_t = top_width;
    let b_l = right_width;

    let q_a = (1.0) / (r_x * r_x) + (b_t * b_t) / (b_l * b_l * r_y * r_y);

    let q_b = (-2.0 * r_x) / (r_x * r_x) + (-2.0 * b_t * r_y) / (b_l * r_y * r_y);

    let discriminant = q_b * q_b - 4.0 * q_a;

    if discriminant < 0.0 || q_a == 0.0 {
        panic!("No intersection");
    }

    // q_a is > 0, so x_2 Should always be smaller than x_1 giving us the closest intersection point.
    //let x_1 = (-q_b + discriminant.sqrt()) / (2.0 * q_a);
    let x_2 = (-q_b - discriminant.sqrt()) / (2.0 * q_a);

    let x = x_2;
    let intersection_angle = f64::acos((x - r_x) / r_x);

    // Since the ellipse was modeled in the first quadrant, the angel will be relative to PI.
    PI - intersection_angle
}

fn extend_path_with_arc(path: &mut BezPath, arc_path: &BezPath) {
    let start = match arc_path.elements().first() {
        Some(PathEl::MoveTo(point)) => point,
        _ => panic!("Expected MoveTo"),
    };
    path.line_to(*start);
    for el in arc_path.elements().iter().skip(1) {
        path.push(*el);
    }
}

#[derive(Default, Clone)]
struct CornerData {
    outer_radius_x: f64,
    outer_radius_y: f64,
    inner_radius_x: f64,
    inner_radius_y: f64,
    inner_is_sharp: bool,
    outer_is_sharp: bool,

    border_point: Point,
    border_radius_point: Point,
    corner_point: Point,
}

impl CornerData {
    fn new(border_spec: &BorderSpec, radius_scale: f64, corner: Corner) -> Self {
        let radius_x = match corner {
            Corner::TopLeft => border_spec.top_left_radius_x,
            Corner::TopRight => border_spec.top_right_radius_x,
            Corner::BottomRight => border_spec.bottom_right_radius_x,
            Corner::BottomLeft => border_spec.bottom_left_radius_x,
        } * radius_scale;

        let radius_y = match corner {
            Corner::TopLeft => border_spec.top_left_radius_y,
            Corner::TopRight => border_spec.top_right_radius_y,
            Corner::BottomRight => border_spec.bottom_right_radius_y,
            Corner::BottomLeft => border_spec.bottom_left_radius_y,
        } * radius_scale;

        let width_x = match corner {
            Corner::TopLeft => border_spec.left_width,
            Corner::TopRight => border_spec.right_width,
            Corner::BottomRight => border_spec.right_width,
            Corner::BottomLeft => border_spec.left_width,
        };

        let width_y = match corner {
            Corner::TopLeft => border_spec.top_width,
            Corner::TopRight => border_spec.top_width,
            Corner::BottomRight => border_spec.bottom_width,
            Corner::BottomLeft => border_spec.bottom_width,
        };

        let mut inner_radius_x = (radius_x - width_x).max(0.0);
        let mut inner_radius_y = (radius_y - width_y).max(0.0);

        let inner_is_sharp = is_inner_radius_sharp(width_x, radius_x, width_y, radius_y);
        let outer_is_sharp = is_outer_radius_sharp(radius_x, radius_y);

        if inner_is_sharp {
            inner_radius_x = 0.0;
            inner_radius_y = 0.0;
        }

        let corner_point = match corner {
            Corner::TopLeft => Point::new(border_spec.x1, border_spec.y1),
            Corner::TopRight => Point::new(border_spec.x2, border_spec.y1),
            Corner::BottomRight => Point::new(border_spec.x2, border_spec.y2),
            Corner::BottomLeft => Point::new(border_spec.x1, border_spec.y2),
        };

        let border_point = match corner {
            Corner::TopLeft => {
                Point::new(border_spec.x1 + border_spec.left_width, border_spec.y1 + border_spec.top_width)
            }
            Corner::TopRight => {
                Point::new(border_spec.x2 - border_spec.right_width, border_spec.y1 + border_spec.top_width)
            }
            Corner::BottomRight => {
                Point::new(border_spec.x2 - border_spec.right_width, border_spec.y2 - border_spec.bottom_width)
            }
            Corner::BottomLeft => {
                Point::new(border_spec.x1 + border_spec.left_width, border_spec.y2 - border_spec.bottom_width)
            }
        };

        let border_radius_point = match corner {
            Corner::TopLeft => Point::new(border_spec.x1 + radius_x, border_spec.y1 + radius_y),
            Corner::TopRight => Point::new(border_spec.x2 - radius_x, border_spec.y1 + radius_y),
            Corner::BottomRight => Point::new(border_spec.x2 - radius_x, border_spec.y2 - radius_y),
            Corner::BottomLeft => Point::new(border_spec.x1 + radius_x, border_spec.y2 - radius_y),
        };

        Self {
            outer_radius_x: radius_x,
            outer_radius_y: radius_y,
            inner_radius_x,
            inner_radius_y,
            inner_is_sharp,
            outer_is_sharp,
            border_point,
            border_radius_point,
            corner_point,
        }
    }
}

impl ComputedBorderSpec {
    fn new(border_spec: &BorderSpec) -> Self {
        let box_width = (border_spec.x2 - border_spec.x1).max(0.0);
        let box_height = (border_spec.y2 - border_spec.y1).max(0.0);

        let f_top_x = box_width / (border_spec.top_left_radius_x + border_spec.top_right_radius_x);
        let f_bottom_x = box_width / (border_spec.bottom_left_radius_x + border_spec.bottom_right_radius_x);

        let f_left_y = box_height / (border_spec.bottom_left_radius_y + border_spec.top_left_radius_y);
        let f_right_y = box_height / (border_spec.top_right_radius_y + border_spec.bottom_right_radius_y);

        let f = f64::min(f64::min(f_top_x, f_left_y), f64::min(f_bottom_x, f_right_y));

        let f = if f < 1.0 { f } else { 1.0 };

        //////////////////////////

        let sides = [
            SideData::new(border_spec.top_width, border_spec.top_color),
            SideData::new(border_spec.right_width, border_spec.right_color),
            SideData::new(border_spec.bottom_width, border_spec.bottom_color),
            SideData::new(border_spec.left_width, border_spec.left_color),
        ];

        let corners = [
            CornerData::new(border_spec, f, Corner::TopLeft),
            CornerData::new(border_spec, f, Corner::TopRight),
            CornerData::new(border_spec, f, Corner::BottomRight),
            CornerData::new(border_spec, f, Corner::BottomLeft),
        ];

        let computed_corner = [
            Self::create_corner(Corner::TopLeft, &corners, &sides),
            Self::create_corner(Corner::TopRight, &corners, &sides),
            Self::create_corner(Corner::BottomRight, &corners, &sides),
            Self::create_corner(Corner::BottomLeft, &corners, &sides),
        ];

        ComputedBorderSpec {
            sides,
            computed_corner,
        }
    }

    fn create_corner(corner: Corner, corners: &[CornerData; 4], sides: &[SideData; 4]) -> ComputedCorner {
        // The (start_angle, end_angle) for the inner curves.
        // The reverse of these angle pairs is used for the outer curves.
        const INNER_ANGLES: [(f64, f64); 4] =
            [(PI, FRAC_PI_2), (FRAC_PI_2, 0.0), (TAU, 3.0 * FRAC_PI_2), (3.0 * FRAC_PI_2, PI)];

        let corner_data = &corners[corner as usize];
        let primary_side = &sides[corner.get_primary_side() as usize];
        let secondary_side = &sides[corner.get_secondary_side() as usize];

        let inner_sweep_angle = intersect_angle(
            corner_data.inner_radius_x,
            corner_data.inner_radius_y,
            primary_side.width,
            secondary_side.width,
        );

        let outer_sweep_angle: f64 = intersect_angle(
            corner_data.outer_radius_x,
            corner_data.outer_radius_y,
            primary_side.width,
            secondary_side.width,
        );

        let inner_intersect_angle = corner.get_relative_angle(inner_sweep_angle);

        let inner_angle = INNER_ANGLES[corner as usize];

        let inner_start_angle = inner_angle.0;
        let inner_end_angle = inner_angle.1;

        let outer_angle = (inner_angle.1, inner_angle.0);
        let outer_start_angle = outer_angle.0;
        let outer_end_angle = outer_angle.1;

        let inner_first_start_angle = inner_start_angle;
        let inner_first_end_angle = inner_intersect_angle;
        let inner_first_sweep_angle = inner_first_end_angle - inner_first_start_angle;

        let inner_second_start_angle = inner_intersect_angle;
        let inner_second_end_angle = inner_end_angle;
        let inner_second_sweep_angle = inner_second_end_angle - inner_second_start_angle;

        let outer_intersect_angle = corner.get_relative_angle(outer_sweep_angle);

        let outer_first_start_angle = outer_start_angle;
        let outer_first_end_angle = outer_intersect_angle;
        let outer_first_sweep_angle = outer_first_end_angle - outer_first_start_angle;

        let outer_second_start_angle = outer_intersect_angle;
        let outer_second_end_angle = outer_end_angle;
        let outer_second_sweep_angle = outer_second_end_angle - outer_second_start_angle;

        let mut inner_sides = [ComputedCornerSide::new(), ComputedCornerSide::new()];
        let mut outer_sides = [ComputedCornerSide::new(), ComputedCornerSide::new()];

        let inner_transition_point = corner_data.border_radius_point
            + Vec2::new(
                corner_data.inner_radius_x * f64::cos(inner_intersect_angle),
                -corner_data.inner_radius_y * f64::sin(inner_intersect_angle),
            );

        let outer_transition_point = corner_data.border_radius_point
            + Vec2::new(
                corner_data.outer_radius_x * f64::cos(outer_intersect_angle),
                -corner_data.outer_radius_y * f64::sin(outer_intersect_angle),
            );

        let inner_arc_first = peniko::kurbo::Arc::new(
            corner_data.border_radius_point,
            Vec2::new(corner_data.inner_radius_x, corner_data.inner_radius_y),
            to_clockwise_angle(inner_first_start_angle),
            -inner_first_sweep_angle,
            0.0,
        )
        .to_path(0.1);

        let inner_arc_second = peniko::kurbo::Arc::new(
            corner_data.border_radius_point,
            Vec2::new(corner_data.inner_radius_x, corner_data.inner_radius_y),
            to_clockwise_angle(inner_second_start_angle),
            -inner_second_sweep_angle,
            0.0,
        )
        .to_path(0.1);

        let outer_arc_first = peniko::kurbo::Arc::new(
            corner_data.border_radius_point,
            Vec2::new(corner_data.outer_radius_x, corner_data.outer_radius_y),
            to_clockwise_angle(outer_first_start_angle),
            -outer_first_sweep_angle,
            0.0,
        )
        .to_path(0.1);

        let outer_arc_second = peniko::kurbo::Arc::new(
            corner_data.border_radius_point,
            Vec2::new(corner_data.outer_radius_x, corner_data.outer_radius_y),
            to_clockwise_angle(outer_second_start_angle),
            -outer_second_sweep_angle,
            0.0,
        )
        .to_path(0.1);

        inner_sides[corner.get_inner_start_side() as usize].arc = inner_arc_first;
        inner_sides[corner.get_inner_start_side().next() as usize].arc = inner_arc_second;

        outer_sides[corner.get_outer_start_side() as usize].arc = outer_arc_first;
        outer_sides[corner.get_outer_start_side().next() as usize].arc = outer_arc_second;

        ComputedCorner {
            inner_transition_point,
            outer_transition_point,
            is_inner_sharp: corner_data.inner_is_sharp,
            is_outer_sharp: corner_data.outer_is_sharp,
            inner_sides,
            outer_sides,
            border_point: corner_data.border_point,
            corner_point: corner_data.corner_point,
        }
    }

    fn get_computed_corner(&self, corner: Corner) -> &ComputedCorner {
        &self.computed_corner[corner as usize]
    }

    pub(crate) fn get_side(&self, side: Side) -> &SideData {
        &self.sides[side as usize]
    }

    pub(crate) fn build_side_path(&self, side: Side) -> BezPath {
        let start_corner = side.as_corner();
        let end_corner = side.next_clockwise().as_corner();

        let start_corner = self.get_computed_corner(start_corner);
        let end_corner = self.get_computed_corner(end_corner);

        const CORNER_SIDES: [(CornerSide, CornerSide); 4] = [
            (CornerSide::Top, CornerSide::Top),
            (CornerSide::Bottom, CornerSide::Top),
            (CornerSide::Bottom, CornerSide::Bottom),
            (CornerSide::Top, CornerSide::Bottom),
        ];

        // Figure out which corner-side arcs to use
        let (start_side, end_side) = CORNER_SIDES[side as usize];

        let mut path = BezPath::new();

        //
        // 1) Start corner, inner arc or move
        //
        if !start_corner.is_inner_sharp {
            // Arc
            path.extend(start_corner.inner_sides[start_side as usize].arc.clone());
        } else {
            // Sharp → just move there
            path.move_to(start_corner.border_point);
        }

        //
        // 2) End corner, inner arc or line
        //
        if !end_corner.is_inner_sharp {
            extend_path_with_arc(&mut path, &end_corner.inner_sides[end_side as usize].arc);
        } else {
            path.line_to(end_corner.border_point);
        }

        //
        // 3) End corner, outer arc or line to corner
        //
        if !end_corner.is_outer_sharp {
            path.line_to(end_corner.outer_transition_point);
            extend_path_with_arc(&mut path, &end_corner.outer_sides[end_side as usize].arc);
        } else {
            path.line_to(end_corner.corner_point);
        }

        //
        // 4) Start corner, outer arc or line to corner
        //
        if !start_corner.is_outer_sharp {
            extend_path_with_arc(&mut path, &start_corner.outer_sides[start_side as usize].arc);
            path.line_to(start_corner.inner_transition_point);
        } else {
            path.line_to(start_corner.corner_point);
        }

        //
        // 5) Close it up
        //
        path.close_path();

        path
    }

    pub(crate) fn build_background_path(&self) -> BezPath {
        let mut background_path = BezPath::new();

        let top_left = self.get_computed_corner(Corner::TopLeft);
        let top_right = self.get_computed_corner(Corner::TopRight);
        let bottom_right = self.get_computed_corner(Corner::BottomRight);
        let bottom_left = self.get_computed_corner(Corner::BottomLeft);

        if !top_left.is_outer_sharp {
            background_path.extend(top_left.outer_sides[CornerSide::Top as usize].arc.clone());
            extend_path_with_arc(&mut background_path, &top_left.outer_sides[CornerSide::Bottom as usize].arc);
        } else {
            background_path.move_to(top_left.corner_point);
        }

        if !bottom_left.is_outer_sharp {
            extend_path_with_arc(&mut background_path, &bottom_left.outer_sides[CornerSide::Top as usize].arc);
            extend_path_with_arc(&mut background_path, &bottom_left.outer_sides[CornerSide::Bottom as usize].arc);
        } else {
            background_path.line_to(bottom_left.corner_point);
        }

        if !bottom_right.is_outer_sharp {
            extend_path_with_arc(&mut background_path, &bottom_right.outer_sides[CornerSide::Bottom as usize].arc);
            extend_path_with_arc(&mut background_path, &bottom_right.outer_sides[CornerSide::Top as usize].arc);
        } else {
            background_path.line_to(bottom_right.corner_point);
        }

        if !top_right.is_outer_sharp {
            extend_path_with_arc(&mut background_path, &top_right.outer_sides[CornerSide::Bottom as usize].arc);
            extend_path_with_arc(&mut background_path, &top_right.outer_sides[CornerSide::Top as usize].arc);
        } else {
            background_path.line_to(top_right.corner_point);
        }
        background_path.close_path();

        background_path
    }
}

const fn to_clockwise_angle(angle: f64) -> f64 {
    TAU - angle
}

fn is_outer_radius_sharp(radius_x: f64, radius_y: f64) -> bool {
    radius_x == 0.0 || radius_y == 0.0
}

fn is_inner_radius_sharp(width_x: f64, radius_x: f64, width_y: f64, radius_y: f64) -> bool {
    width_x >= radius_x || width_y >= radius_y
}
