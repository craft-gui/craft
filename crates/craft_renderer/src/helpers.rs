use vello_common::paint::PaintType;
use crate::Brush;

#[cfg(any(feature = "vello_cpu_renderer", feature = "vello_hybrid_renderer", feature = "vello_hybrid_renderer_webgl"))]
pub(crate) fn brush_to_paint(brush: &Brush) -> PaintType {
    match brush {
        Brush::Color(color) => PaintType::Solid(*color),
        Brush::Gradient(gradient) => PaintType::Gradient(gradient.clone()),
    }
}

pub const fn rgba_to_encoded_u32(r: u32, g: u32, b: u32, a: u32) -> u32 {
    b | (g << 8) | (r << 16) | (a << 24)
}
