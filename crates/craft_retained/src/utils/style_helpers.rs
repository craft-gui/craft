use crate::Color;
use crate::style::Unit;

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba8(r, g, b, a)
}

pub fn px<U>(val: U) -> Unit
where
    U: Into<f64>,
{
    Unit::Px(val.into() as f32)
}

pub fn pct<U>(val: U) -> Unit
where
    U: Into<f64>,
{
    Unit::Percentage(val.into() as f32)
}

pub const fn auto() -> Unit {
    Unit::Auto
}
