pub use peniko::kurbo::{CubicBez, ParamCurve, Point};

/// The motion of an animation modeled with a mathematical function.
#[derive(Default, Copy, Clone, Debug)]
pub enum TimingFunction {
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#linear
    #[default]
    Linear,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease
    Ease,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease-in
    EaseIn,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease-out
    EaseOut,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#ease-in-out
    EaseInOut,
    /// https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timing-function#cubic-beziernumber_01_number_number_01_number
    BezierCurve(FixedCubicBezier),
}

/// A cubic bézier curve where P0 and P3 are stuck at (0,0) and (1,1).
#[derive(Clone, Copy, Debug)]
pub struct FixedCubicBezier {
    pub(crate) cubic_bez: CubicBez,
}

impl FixedCubicBezier {
    /// Sets P1 and P2 of a fixed cubic bézier curve.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            cubic_bez: CubicBez::new(
                Point::new(0.0, 0.0),
                Point::new(x1 as f64, y1 as f64),
                Point::new(x2 as f64, y2 as f64),
                Point::new(1.0, 1.0),
            ),
        }
    }
}
