// https://www.w3.org/TR/css-backgrounds-3/
// https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/box-shadow
use craft_primitives::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxShadow {
    pub inset: bool,
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur_radius: f64,
    pub spread_radius: f64,
    pub color: Color,
}

impl BoxShadow {
    pub fn new(inset: bool, offset_x: f64, offset_y: f64, blur_radius: f64, spread_radius: f64, color: Color) -> Self {
        Self {
            inset,
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            color,
        }
    }
}
