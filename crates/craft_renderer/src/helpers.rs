#[cfg(any(
    feature = "vello_cpu_renderer",
    feature = "vello_hybrid_renderer",
    feature = "vello_hybrid_renderer_webgl"
))]
use crate::Brush;
use craft_primitives::geometry::{Point, Rectangle};
use craft_primitives::gradient::{Extend, GradientKind, HueDirection};
use peniko::InterpolationAlphaSpace;
use peniko::color::ColorSpaceTag;
#[cfg(any(
    feature = "vello_cpu_renderer",
    feature = "vello_hybrid_renderer",
    feature = "vello_hybrid_renderer_webgl"
))]
use vello_common::paint::PaintType;

#[cfg(any(
    feature = "vello_cpu_renderer",
    feature = "vello_hybrid_renderer",
    feature = "vello_hybrid_renderer_webgl"
))]
pub(crate) fn brush_to_paint(rect: Rectangle, brush: &Brush) -> PaintType {
    match brush {
        Brush::Color(color) => PaintType::Solid(*color),
        Brush::Gradient(gradient) => {
            let kind: peniko::GradientKind = match &gradient.kind {
                GradientKind::Linear(linear) => {
                    let start_x = rect.left() as f64 + (linear.start.x * rect.width as f64);
                    let start_y = rect.top() as f64 + (linear.start.y * rect.height as f64);

                    let end_x = rect.left() as f64 + (linear.end.x * rect.width as f64);
                    let end_y = rect.top() as f64 + (linear.end.y * rect.height as f64);

                    peniko::GradientKind::Linear(peniko::LinearGradientPosition {
                        start: Point::new(start_x, start_y),
                        end: Point::new(end_x, end_y),
                    })
                }
                GradientKind::Radial(radial) => {
                    let start_center_x = rect.left() as f64 + (radial.start_center.x * rect.width as f64);
                    let start_center_y = rect.top() as f64 + (radial.start_center.y * rect.height as f64);

                    let end_center_x = rect.left() as f64 + (radial.end_center.x * rect.width as f64);
                    let end_center_y = rect.top() as f64 + (radial.end_center.y * rect.height as f64);

                    let radius_scale = rect.width.max(rect.height);

                    peniko::GradientKind::Radial(peniko::RadialGradientPosition {
                        start_center: Point::new(start_center_x, start_center_y),
                        start_radius: radial.start_radius * radius_scale,
                        end_center: Point::new(end_center_x, end_center_y),
                        end_radius: radial.end_radius * radius_scale,
                    })
                }
                GradientKind::Sweep(sweep) => {
                    let center_x = rect.left() as f64 + (sweep.center.x * rect.width as f64);
                    let center_y = rect.top() as f64 + (sweep.center.y * rect.height as f64);

                    peniko::GradientKind::Sweep(peniko::SweepGradientPosition {
                        center: Point::new(center_x, center_y),
                        start_angle: sweep.start_angle,
                        end_angle: sweep.end_angle,
                    })
                }
            };

            let extend: peniko::Extend = match &gradient.extend {
                Extend::Pad => peniko::Extend::Pad,
                Extend::Repeat => peniko::Extend::Repeat,
                Extend::Reflect => peniko::Extend::Reflect,
            };

            let hue_direction: peniko::color::HueDirection = match &gradient.hue_direction {
                HueDirection::Shorter => peniko::color::HueDirection::Shorter,
                HueDirection::Longer => peniko::color::HueDirection::Longer,
                HueDirection::Increasing => peniko::color::HueDirection::Increasing,
                HueDirection::Decreasing => peniko::color::HueDirection::Decreasing,
            };

            let stops: Vec<peniko::ColorStop> = gradient
                .color_stops
                .iter()
                .map(|c| peniko::ColorStop {
                    offset: c.offset,
                    color: c.color.into(),
                })
                .collect();

            PaintType::Gradient(peniko::Gradient {
                kind,
                extend,
                interpolation_cs: ColorSpaceTag::Srgb,
                hue_direction,
                interpolation_alpha_space: InterpolationAlphaSpace::Premultiplied,
                stops: peniko::ColorStops(stops.into()),
            })
        }
    }
}

#[cfg(feature = "vello_cpu_renderer")]
pub const fn rgba_to_encoded_u32(r: u32, g: u32, b: u32, a: u32) -> u32 {
    b | (g << 8) | (r << 16) | (a << 24)
}
