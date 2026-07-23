use smallvec::SmallVec;

use kurbo::Point;
use peniko::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub kind: GradientKind,
    pub color_stops: SmallVec<[ColorStop; 4]>,
    pub extend: Extend,
    pub hue_direction: HueDirection,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CoordinateSpace {
    Relative,
    Absolute
}

impl Gradient {
    pub fn new_linear(start: Point, end: Point) -> Self {
        Gradient {
            kind: GradientKind::Linear(
                LinearGradientData {
                    start,
                    end,
                }
            ),
            color_stops: Default::default(),
            extend: Default::default(),
            hue_direction: Default::default(),
        }
    }

    #[must_use]
    pub fn color_stops(mut self, stops: &[ColorStop]) -> Self {
        self.color_stops = SmallVec::from_slice(stops);
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub enum Extend {
    #[default]
    Pad = 0,
    Repeat = 1,
    Reflect = 2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorStop {
    /// Normalized (0.0 - 1.0)
    pub offset: f32,
    pub color: Color
}

impl ColorStop {
    pub fn new(offset: f32, color: Color) -> Self {
        Self {
            offset,
            color,
        }
    }
}

/// The hue direction for interpolation
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum HueDirection {
    #[default]
    Shorter = 0,
    Longer = 1,
    Increasing = 2,
    Decreasing = 3,
}

/// Note: Coordinates are relative to the widget.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradientData {
    pub start: Point,
    pub end: Point,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadialGradientData {
    /// Center of start circle.
    pub start_center: Point,
    /// Radius of start circle.
    pub start_radius: f32,
    /// Center of end circle.
    pub end_center: Point,
    /// Radius of end circle.
    pub end_radius: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SweepGradientData {
    pub center: Point,
    pub start_angle: f32,
    pub end_angle: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GradientKind {
    Linear(LinearGradientData),
    Radial(RadialGradientData),
    Sweep(SweepGradientData)
}