use kurbo::{Arc, BezPath, PathEl, Point, Rect, Shape, Vec2};

use std::f64::consts::{FRAC_PI_2, PI};
use std::ops::Add;
pub const TOP_LEFT: usize = 0;
pub const TOP_RIGHT: usize = 1;
pub const BOTTOM_RIGHT: usize = 2;
pub const BOTTOM_LEFT: usize = 3;

pub const TOP: usize = 0;
pub const RIGHT: usize = 1;
pub const BOTTOM: usize = 2;
pub const LEFT: usize = 3;

pub const CORNERS: [usize; 4] = [TOP_LEFT, TOP_RIGHT, BOTTOM_RIGHT, BOTTOM_LEFT];
pub const NEXT_CORNER: [usize; 4] = [TOP_RIGHT, BOTTOM_RIGHT, BOTTOM_LEFT, TOP_LEFT];
pub const CORNER_HORIZONTAL_SIDE: [usize; 4] = [LEFT, RIGHT, RIGHT, LEFT];
pub const CORNER_VERTICAL_SIDE: [usize; 4] = [TOP, TOP, BOTTOM, BOTTOM];
const CORNER_RADIUS_SIGN: [Vec2; 4] = [
    Vec2::new(1.0, 1.0),
    Vec2::new(-1.0, 1.0),
    Vec2::new(-1.0, -1.0),
    Vec2::new(1.0, -1.0),
];

const CORNER_START_ANGLES: [f64; 4] = [PI, 3.0 * FRAC_PI_2, 0.0, FRAC_PI_2];

const FIRST_ARC: usize = 1;
const SECOND_ARC: usize = 0;
const THIRD_ARC: usize = 3;
const FOURTH_ARC: usize = 2;

const NEXT_ARC: [usize; 4] = [THIRD_ARC, SECOND_ARC, FIRST_ARC, FOURTH_ARC];

#[derive(Clone, Copy, Default, PartialEq)]
pub struct CssRoundedRect {
    /// The minimum x coordinate (left edge).
    x0: f64,
    /// The minimum y coordinate (top edge in y-down spaces).
    y0: f64,
    /// The maximum x coordinate (right edge).
    x1: f64,
    /// The maximum y coordinate (bottom edge in y-down spaces).
    y1: f64,

    outer_radii: [Vec2; 4],
    inner_radii: [Vec2; 4],

    /// The corner points of the outer rectangle.
    corners: [Point; 4],

    // (Inner, Outer)
    intersect_angles: [Vec2; 4],

    // corners[outer, outer, inner, inner]
    corners_arcs: [[Option<Arc>; 4]; 4],

    background_arcs: [Option<Arc>; 4],

    widths: [f64; 4],
}

impl CssRoundedRect {
    pub fn new(rect: Rect, widths: [f64; 4], radii: [Vec2; 4]) -> Self {
        let mut css_rounded_rect = Self {
            x0: rect.x0,
            y0: rect.y0,
            x1: rect.x0 + rect.width(),
            y1: rect.y0 + rect.height(),
            outer_radii: radii,
            inner_radii: [Vec2::default(); 4],
            intersect_angles: [Vec2::default(); 4],
            corners_arcs: [[None; 4]; 4],
            background_arcs: [None; 4],
            widths,
            corners: [
                Point::new(rect.x0, rect.y0),
                Point::new(rect.x1, rect.y0),
                Point::new(rect.x1, rect.y1),
                Point::new(rect.x0, rect.y1),
            ],
        };
        css_rounded_rect.scale_radius();

        for corner in CORNERS {
            let outer_radius = css_rounded_rect.outer_radii[corner];
            let width_x = css_rounded_rect.widths[CORNER_HORIZONTAL_SIDE[corner]];
            let width_y = css_rounded_rect.widths[CORNER_VERTICAL_SIDE[corner]];
            let is_sharp = is_inner_radius_sharp(width_x, outer_radius.x, width_y, outer_radius.y);
            if !is_sharp {
                css_rounded_rect.inner_radii[corner] =
                    Vec2::new(outer_radius.x - width_x, outer_radius.y - width_y);
            }

            let inner_radius = css_rounded_rect.inner_radii[corner];

            let (mut outer_x, mut outer_y) = (outer_radius.x, outer_radius.y);
            let (mut inner_x, mut inner_y) = (inner_radius.x, inner_radius.y);

            let (mut side_for_radius_x, mut side_for_radius_y) = (width_y, width_x);

            // For the corners that are NOT TOP_LEFT or BOTTOM_RIGHT we need to swap axes.
            if !(corner == TOP_LEFT || corner == BOTTOM_RIGHT) {
                std::mem::swap(&mut outer_x, &mut outer_y);
                std::mem::swap(&mut inner_x, &mut inner_y);

                std::mem::swap(&mut side_for_radius_x, &mut side_for_radius_y);
            }

            let outer_sweep_angle =
                intersect_angle(outer_x, outer_y, side_for_radius_x, side_for_radius_y);
            let inner_sweep_angle =
                intersect_angle(inner_x, inner_y, side_for_radius_x, side_for_radius_y);

            css_rounded_rect.intersect_angles[corner] =
                Vec2::new(inner_sweep_angle, outer_sweep_angle);
        }

        css_rounded_rect.compute_arcs();
        css_rounded_rect
    }

    fn compute_arcs(&mut self) {
        for corner in CORNERS {
            let outer_radius = self.outer_radii[corner];
            let inner_radius = self.inner_radii[corner];

            let radius_sign = CORNER_RADIUS_SIGN[corner];

            let outer_radius = Vec2::new(
                radius_sign.x * outer_radius.x,
                radius_sign.y * outer_radius.y,
            );
            let inner_radius = Vec2::new(
                radius_sign.x * inner_radius.x,
                radius_sign.y * inner_radius.y,
            );

            if is_outer_radius_sharp(outer_radius) {
                continue;
            }

            let center = self.corners[corner].add(outer_radius);
            let intersect_angle = self.intersect_angles[corner];

            let outside_1st_arc = Arc::new(
                center,
                self.outer_radii[corner],
                CORNER_START_ANGLES[corner],
                intersect_angle.y,
                0.0,
            );

            let outside_2nd_arc = Arc::new(
                center,
                self.outer_radii[corner],
                CORNER_START_ANGLES[corner] + intersect_angle.y,
                FRAC_PI_2 - intersect_angle.y,
                0.0,
            );

            // the background arc will always have radius pi/2
            let background_arc = Arc::new(
                center,
                self.outer_radii[corner],
                CORNER_START_ANGLES[corner],
                FRAC_PI_2,
                0.0,
            );

            self.corners_arcs[corner][0] = Some(outside_1st_arc);
            self.corners_arcs[corner][1] = Some(outside_2nd_arc);
            self.background_arcs[corner] = Some(background_arc);

            if is_outer_radius_sharp(inner_radius) {
                continue;
            }

            let inner_1st_arc = Arc::new(
                center,
                self.inner_radii[corner],
                CORNER_START_ANGLES[corner] + FRAC_PI_2,
                -FRAC_PI_2 + intersect_angle.x,
                0.0,
            );

            let inner_2nd_arc = Arc::new(
                center,
                self.inner_radii[corner],
                CORNER_START_ANGLES[corner] + intersect_angle.x,
                -intersect_angle.x,
                0.0,
            );

            self.corners_arcs[corner][2] = Some(inner_1st_arc);
            self.corners_arcs[corner][3] = Some(inner_2nd_arc);
        }
    }

    pub fn get_side(&self, side: usize) -> Option<BezPath> {
        let width = self.widths[side];

        // The border has no thickness.
        if width == 0.0 {
            return None;
        }

        let corner = side;
        let next_corner = NEXT_CORNER[corner];

        let rect_corner = self.corners[corner];
        let next_rect_corner = self.corners[next_corner];

        let mut path = BezPath::new();

        let mut current_arc = FIRST_ARC;

        if let Some(arc) = &self.corners_arcs[corner][current_arc] {
            path.extend(&arc.to_path(0.01));
        } else {
            path.move_to(rect_corner);
        }

        current_arc = NEXT_ARC[current_arc];

        if let Some(arc) = &self.corners_arcs[next_corner][current_arc] {
            extend_path_with_arc(&mut path, &arc.to_path(0.01));
        } else {
            path.line_to(next_rect_corner);
        }

        current_arc = NEXT_ARC[current_arc];

        if let Some(arc) = &self.corners_arcs[next_corner][current_arc] {
            extend_path_with_arc(&mut path, &arc.to_path(0.01));
        } else {
            let offset = CORNER_RADIUS_SIGN[next_corner];
            let horizontal_side = CORNER_HORIZONTAL_SIDE[next_corner];
            let vertical_side = CORNER_VERTICAL_SIDE[next_corner];
            let horizontal_width = self.widths[horizontal_side];
            let vertical_width = self.widths[vertical_side];
            let inside_corner = next_rect_corner.add(Vec2::new(
                horizontal_width * offset.x,
                vertical_width * offset.y,
            ));
            path.line_to(inside_corner);
        }

        current_arc = NEXT_ARC[current_arc];

        if let Some(arc) = &self.corners_arcs[corner][current_arc] {
            extend_path_with_arc(&mut path, &arc.to_path(0.01));
        } else {
            let offset = CORNER_RADIUS_SIGN[corner];
            let horizontal_side = CORNER_HORIZONTAL_SIDE[corner];
            let vertical_side = CORNER_VERTICAL_SIDE[corner];
            let horizontal_width = self.widths[horizontal_side];
            let vertical_width = self.widths[vertical_side];
            let inside_corner = rect_corner.add(Vec2::new(
                horizontal_width * offset.x,
                vertical_width * offset.y,
            ));
            path.line_to(inside_corner);
        }
        path.close_path();

        Some(path)
    }

    /// Calculate the scaling factor to apply to the radii to ensure they fit within the rectangle.
    fn scale_radius(&mut self) {
        let box_width = self.width();
        let box_height = self.height();

        let f_top_x = box_width / (self.outer_radii[TOP_LEFT].x + self.outer_radii[TOP_RIGHT].x);
        let f_bottom_x =
            box_width / (self.outer_radii[BOTTOM_LEFT].x + self.outer_radii[BOTTOM_RIGHT].x);

        let f_left_y =
            box_height / (self.outer_radii[BOTTOM_LEFT].y + self.outer_radii[TOP_LEFT].y);
        let f_right_y =
            box_height / (self.outer_radii[BOTTOM_RIGHT].y + self.outer_radii[TOP_RIGHT].y);

        let radius_scale = f_top_x
            .min(f_left_y)
            .min(f_bottom_x)
            .min(f_right_y)
            .min(1.0);

        for radius in &mut self.outer_radii {
            *radius *= radius_scale;
        }
    }

    /// The width of the rectangle.
    ///
    /// Note: negative width is treated as 0.
    #[inline]
    pub fn width(&self) -> f64 {
        (self.x1 - self.x0).max(0.0)
    }

    /// The height of the rectangle.
    ///
    /// Note: negative height is treated as 0.
    #[inline]
    pub fn height(&self) -> f64 {
        (self.y1 - self.y0).max(0.0)
    }
}

/// Calculate the angle of intersection between the quarter-ellipse and a line from the border point to the corner point.
fn intersect_angle(
    top_left_radius_x: f64,
    top_left_radius_y: f64,
    top_width: f64,
    right_width: f64,
) -> f64 {
    // 1. Set up the equations and solve for the intersection point using the quadratic formula.
    // 2. To get the angle, we use the parametric equation of the ellipse, solving for the angle.

    // https://www.desmos.com/calculator/l9yg7tsvfz

    let r_x = top_left_radius_x;
    let r_y = top_left_radius_y;

    if r_x == 0.0 || r_y == 0.0 {
        // There is no ellipse, so the corner is rectangular.
        return FRAC_PI_2;
    }

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
        return PI / 2.0;
        //panic!("No intersection");
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

pub struct CssRectPathIter {
    rect: CssRoundedRect,
    current_corner: usize,
    current_corner_iter: Option<Box<dyn Iterator<Item = PathEl>>>,
    tolerance: f64,
}

impl CssRectPathIter {
    fn next_arc_element(&mut self) -> Option<PathEl> {
        if let Some(iter) = &mut self.current_corner_iter {
            if let Some(el) = iter.next() {
                // Turn MoveTo into LineTo except for the first corner
                if self.current_corner != 1 && matches!(el, PathEl::MoveTo(_)) {
                    if let PathEl::MoveTo(p) = el {
                        return Some(PathEl::LineTo(p));
                    }
                }
                return Some(el);
            } else {
                self.current_corner_iter = None;
            }
        }
        None
    }
}

impl Iterator for CssRectPathIter {
    type Item = PathEl;

    fn next(&mut self) -> Option<PathEl> {
        if self.current_corner_iter.is_some() {
            let path_element = self.next_arc_element();
            if path_element.is_some() {
                return path_element;
            }
        }

        self.current_corner += 1;
        self.current_corner_iter = None;

        if self.current_corner > 5 {
            return None;
        }

        let current_corner = if self.current_corner <= 4 {
            self.rect.background_arcs[self.current_corner - 1].as_ref()
        } else {
            None
        };
        if let Some(current_background_arc) = current_corner {
            let iter = current_background_arc.path_elements(self.tolerance);
            self.current_corner_iter = Some(Box::new(iter));
            self.next_arc_element()
        } else {
            match self.current_corner {
                1 => Some(PathEl::MoveTo(self.rect.corners[TOP_LEFT])),
                2 => Some(PathEl::LineTo(self.rect.corners[TOP_RIGHT])),
                3 => Some(PathEl::LineTo(self.rect.corners[BOTTOM_RIGHT])),
                4 => Some(PathEl::LineTo(self.rect.corners[BOTTOM_LEFT])),
                5 => Some(PathEl::ClosePath),
                _ => None,
            }
        }
    }
}

impl Shape for CssRoundedRect {
    type PathElementsIter<'iter> = CssRectPathIter;

    fn path_elements(&self, tolerance: f64) -> Self::PathElementsIter<'_> {
        Self::PathElementsIter {
            rect: *self,
            current_corner: TOP_LEFT,
            current_corner_iter: None,
            tolerance,
        }
    }

    /// The area of the CSS border rectangle.
    fn area(&self) -> f64 {
        let mut removed_border_area = 0.0;
        for corner in CORNERS {
            let outer_radius = self.outer_radii[corner];
            let quarter_ellipse_area = PI * outer_radius.x * outer_radius.y / 4.0;
            let removed = (outer_radius.x * outer_radius.y) - quarter_ellipse_area;
            removed_border_area += removed;
        }
        self.width() * self.height() - removed_border_area
    }

    /// Approximate the CSS border rectangle perimeter.
    ///
    /// This uses a numerical approximation. The absolute error between the calculated perimeter
    /// and the true perimeter is bounded by `accuracy` (modulo floating point rounding errors).
    ///
    /// For circular ellipses (equal horizontal and vertical radii), the calculated perimeter is
    /// exact.
    fn perimeter(&self, accuracy: f64) -> f64 {
        let mut border_perimeter_delta = 0.0;
        for corner in CORNERS {
            let outer_radius = self.outer_radii[corner];
            border_perimeter_delta +=
                kurbo::Ellipse::new(Point::default(), outer_radius, 0.0).perimeter(accuracy) / 4.0;
            border_perimeter_delta -= outer_radius.x + outer_radius.y;
        }
        self.width() * 2.0 + self.height() * 2.0 + border_perimeter_delta
    }

    fn winding(&self, p: Point) -> i32 {
        if p.x < self.x0 || p.x > self.x1 || p.y < self.y0 || p.y > self.y1 {
            return 0;
        }

        for corner in CORNERS {
            if let Some(arc) = &self.corners_arcs[corner][0] {
                let border_radius = self.outer_radii[corner];
                let sign = CORNER_RADIUS_SIGN[corner];
                let corner_point = self.corners[corner];
                let border_point = corner_point.add(Vec2::new(
                    border_radius.x * sign.x,
                    border_radius.y * sign.y,
                ));
                let quarter_ellipse_bounds = Rect::from_points(corner_point, border_point);
                if quarter_ellipse_bounds.contains(p) && arc.contains(p) {
                    return 0;
                }
            }
        }

        1
    }

    fn bounding_box(&self) -> Rect {
        // TODO: improve accuracy by considering the radii
        Rect::new(self.x0, self.y0, self.x1, self.y1)
    }
}

fn is_inner_radius_sharp(width_x: f64, radius_x: f64, width_y: f64, radius_y: f64) -> bool {
    width_x >= radius_x || width_y >= radius_y
}

fn is_outer_radius_sharp(radii: Vec2) -> bool {
    radii.x == 0.0 || radii.y == 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    const EPS: f64 = 1e-9;

    #[test]
    fn returns_zero_when_right_width_zero() {
        // If the right border width is 0.0, it should always return 0.0 (rectangular corner)
        let result = intersect_angle(10.0, 10.0, 5.0, 0.0);
        assert!((result - 0.0).abs() < EPS);
    }

    #[test]
    fn returns_pi_over_4_for_equal_axes_case() {
        // For a circle (rx == ry) and symmetric border widths, the intersection angle is 45° (π/4)
        let result = intersect_angle(10.0, 10.0, 10.0, 10.0);
        assert!(result <= FRAC_PI_2);
    }

    #[test]
    fn returns_f() {
        // If the right border width is 0.0, it should always return 0.0 (rectangular corner)
        let result = intersect_angle(10.0, 0.0, 5.0, 5.0);
        println!("Result: {}", result);
        assert!((result - 0.0).abs() < EPS);
    }
}
