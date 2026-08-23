use retgui_primitives::Color;
use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::TrblRectangle;
use retgui_primitives::gradient::{Gradient, GradientKind, LinearGradientData, RadialGradientData, SweepGradientData};

use crate::style::{BoxShadow, FontWeight, ScrollbarColor, StyleVariant, Underline, Unit};

pub(super) fn interpolate_style_variant(start: &StyleVariant, end: &StyleVariant, t: f64) -> StyleVariant {
    use StyleVariant::*;

    match (start, end) {
        (Margin(start), Margin(end)) => Margin(interpolate(start, end, t, lerp_unit_rect)),
        (Padding(start), Padding(end)) => Padding(interpolate(start, end, t, lerp_unit_rect)),
        (Gap(start), Gap(end)) => Gap(interpolate(start, end, t, lerp_unit_pair)),
        (Inset(start), Inset(end)) => Inset(interpolate(start, end, t, lerp_unit_rect)),
        (Width(start), Width(end)) => Width(interpolate(start, end, t, lerp_unit)),
        (MinWidth(start), MinWidth(end)) => MinWidth(interpolate(start, end, t, lerp_unit)),
        (MaxWidth(start), MaxWidth(end)) => MaxWidth(interpolate(start, end, t, lerp_unit)),
        (Height(start), Height(end)) => Height(interpolate(start, end, t, lerp_unit)),
        (MinHeight(start), MinHeight(end)) => MinHeight(interpolate(start, end, t, lerp_unit)),
        (MaxHeight(start), MaxHeight(end)) => MaxHeight(interpolate(start, end, t, lerp_unit)),
        (FlexGrow(start), FlexGrow(end)) => FlexGrow(interpolate(start, end, t, lerp_f32_value)),
        (FlexShrink(start), FlexShrink(end)) => FlexShrink(interpolate(start, end, t, lerp_f32_value)),
        (FlexBasis(start), FlexBasis(end)) => FlexBasis(interpolate(start, end, t, lerp_unit)),
        (Order(start), Order(end)) => Order(interpolate(start, end, t, lerp_i32)),
        (BackgroundBrush(start), BackgroundBrush(end)) => BackgroundBrush(interpolate(start, end, t, lerp_brush)),
        (TextBrush(start), TextBrush(end)) => TextBrush(interpolate(start, end, t, lerp_brush)),
        (LineHeight(start), LineHeight(end)) => LineHeight(interpolate(start, end, t, lerp_f32_value)),
        (FontSize(start), FontSize(end)) => FontSize(interpolate(start, end, t, lerp_f32_value)),
        (FontWeight(start), FontWeight(end)) => FontWeight(interpolate(start, end, t, lerp_font_weight)),
        (Underline(start), Underline(end)) => Underline(interpolate(start, end, t, lerp_underline)),
        (BorderColor(start), BorderColor(end)) => BorderColor(interpolate(start, end, t, lerp_color_rect)),
        (BorderWidth(start), BorderWidth(end)) => BorderWidth(interpolate(start, end, t, lerp_unit_rect)),
        (BorderRadius(start), BorderRadius(end)) => BorderRadius(interpolate(start, end, t, lerp_radii)),
        (OutlineColor(start), OutlineColor(end)) => OutlineColor(interpolate(start, end, t, lerp_color_rect)),
        (OutlineWidth(start), OutlineWidth(end)) => OutlineWidth(interpolate(start, end, t, lerp_unit_rect)),
        (ScrollbarBrush(start), ScrollbarBrush(end)) => {
            ScrollbarBrush(interpolate(start, end, t, lerp_scrollbar_color))
        }
        (ScrollbarThumbMargin(start), ScrollbarThumbMargin(end)) => {
            ScrollbarThumbMargin(interpolate(start, end, t, lerp_f32_rect))
        }
        (ScrollbarThumbRadius(start), ScrollbarThumbRadius(end)) => {
            ScrollbarThumbRadius(interpolate(start, end, t, lerp_radii))
        }
        (ScrollbarWidth(start), ScrollbarWidth(end)) => ScrollbarWidth(interpolate(start, end, t, lerp_f32_value)),
        (SelectionBrush(start), SelectionBrush(end)) => SelectionBrush(interpolate(start, end, t, lerp_brush)),
        (CursorBrush(start), CursorBrush(end)) => CursorBrush(interpolate(start, end, t, lerp_optional_brush)),
        (BoxShadows(start), BoxShadows(end)) => {
            BoxShadows(lerp_box_shadows(start, end, t).unwrap_or_else(|| discrete(start, end, t)))
        }

        (BoxSizing(_), BoxSizing(_))
        | (Position(_), Position(_))
        | (Display(_), Display(_))
        | (Wrap(_), Wrap(_))
        | (AlignItems(_), AlignItems(_))
        | (AlignSelf(_), AlignSelf(_))
        | (AlignContent(_), AlignContent(_))
        | (JustifyContent(_), JustifyContent(_))
        | (FlexDirection(_), FlexDirection(_))
        | (FontFamily(_), FontFamily(_))
        | (FontStyle(_), FontStyle(_))
        | (TextAlign(_), TextAlign(_))
        | (Overflow(_), Overflow(_))
        | (Overlay(_), Overlay(_))
        | (Visible(_), Visible(_)) => discrete(start, end, t),

        _ => unreachable!("cannot interpolate different style variants"),
    }
}

fn interpolate<T: Clone>(start: &T, end: &T, t: f64, interpolator: impl FnOnce(&T, &T, f64) -> Option<T>) -> T {
    interpolator(start, end, t).unwrap_or_else(|| discrete(start, end, t))
}

#[inline(always)]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline(always)]
fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn discrete<T: Clone>(start: &T, end: &T, t: f64) -> T {
    if t < 0.5 { start.clone() } else { end.clone() }
}

fn lerp_f32_value(start: &f32, end: &f32, t: f64) -> Option<f32> {
    Some(lerp(*start, *end, t as f32))
}

fn lerp_i32(start: &i32, end: &i32, t: f64) -> Option<i32> {
    Some(lerp(*start as f32, *end as f32, t as f32).round() as i32)
}

fn lerp_font_weight(start: &FontWeight, end: &FontWeight, t: f64) -> Option<FontWeight> {
    Some(FontWeight(lerp(start.0 as f32, end.0 as f32, t as f32).round() as u16))
}

fn lerp_unit(start: &Unit, end: &Unit, t: f64) -> Option<Unit> {
    match (*start, *end) {
        (Unit::Px(start), Unit::Px(end)) => Some(Unit::Px(lerp(start, end, t as f32))),
        (Unit::Percentage(start), Unit::Percentage(end)) => Some(Unit::Percentage(lerp(start, end, t as f32))),
        (Unit::Auto, Unit::Auto) => Some(Unit::Auto),
        _ => None,
    }
}

fn lerp_unit_pair(start: &[Unit; 2], end: &[Unit; 2], t: f64) -> Option<[Unit; 2]> {
    Some([lerp_unit(&start[0], &end[0], t)?, lerp_unit(&start[1], &end[1], t)?])
}

fn lerp_unit_rect(start: &TrblRectangle<Unit>, end: &TrblRectangle<Unit>, t: f64) -> Option<TrblRectangle<Unit>> {
    Some(TrblRectangle::new(
        lerp_unit(&start.top, &end.top, t)?,
        lerp_unit(&start.right, &end.right, t)?,
        lerp_unit(&start.bottom, &end.bottom, t)?,
        lerp_unit(&start.left, &end.left, t)?,
    ))
}

fn lerp_color_rect(start: &TrblRectangle<Color>, end: &TrblRectangle<Color>, t: f64) -> Option<TrblRectangle<Color>> {
    let t = t as f32;
    Some(TrblRectangle::new(
        start.top.lerp_rect(end.top, t),
        start.right.lerp_rect(end.right, t),
        start.bottom.lerp_rect(end.bottom, t),
        start.left.lerp_rect(end.left, t),
    ))
}

fn lerp_f32_rect(start: &TrblRectangle<f32>, end: &TrblRectangle<f32>, t: f64) -> Option<TrblRectangle<f32>> {
    let t = t as f32;
    Some(TrblRectangle::new(
        lerp(start.top, end.top, t),
        lerp(start.right, end.right, t),
        lerp(start.bottom, end.bottom, t),
        lerp(start.left, end.left, t),
    ))
}

fn lerp_radii(start: &[(f32, f32); 4], end: &[(f32, f32); 4], t: f64) -> Option<[(f32, f32); 4]> {
    let t = t as f32;
    Some(std::array::from_fn(|index| {
        (
            lerp(start[index].0, end[index].0, t),
            lerp(start[index].1, end[index].1, t),
        )
    }))
}

fn lerp_brush(start: &Brush, end: &Brush, t: f64) -> Option<Brush> {
    match (start, end) {
        (Brush::Color(start), Brush::Color(end)) => Some(Brush::Color(start.lerp_rect(*end, t as f32))),
        (Brush::Gradient(start), Brush::Gradient(end)) => lerp_gradient(start, end, t).map(Brush::Gradient),
        _ => None,
    }
}

fn lerp_optional_brush(start: &Option<Brush>, end: &Option<Brush>, t: f64) -> Option<Option<Brush>> {
    match (start, end) {
        (Some(start), Some(end)) => lerp_brush(start, end, t).map(Some),
        (None, None) => Some(None),
        _ => None,
    }
}

fn lerp_optional_f32(start: Option<f32>, end: Option<f32>, t: f64) -> Option<Option<f32>> {
    match (start, end) {
        (Some(start), Some(end)) => Some(Some(lerp(start, end, t as f32))),
        (None, None) => Some(None),
        _ => None,
    }
}

fn lerp_underline(start: &Option<Underline>, end: &Option<Underline>, t: f64) -> Option<Option<Underline>> {
    match (start, end) {
        (Some(start), Some(end)) => Some(Some(Underline {
            thickness: lerp_optional_f32(start.thickness, end.thickness, t)?,
            brush: lerp_brush(&start.brush, &end.brush, t)?,
            offset: lerp_optional_f32(start.offset, end.offset, t)?,
        })),
        (None, None) => Some(None),
        _ => None,
    }
}

fn lerp_scrollbar_color(start: &ScrollbarColor, end: &ScrollbarColor, t: f64) -> Option<ScrollbarColor> {
    Some(ScrollbarColor {
        thumb_color: lerp_brush(&start.thumb_color, &end.thumb_color, t)?,
        track_color: lerp_brush(&start.track_color, &end.track_color, t)?,
    })
}

fn lerp_box_shadows(start: &[BoxShadow], end: &[BoxShadow], t: f64) -> Option<Vec<BoxShadow>> {
    if start.len() != end.len() {
        return None;
    }

    start
        .iter()
        .zip(end)
        .map(|(start, end)| {
            (start.inset == end.inset).then(|| BoxShadow {
                inset: start.inset,
                offset_x: lerp_f64(start.offset_x, end.offset_x, t),
                offset_y: lerp_f64(start.offset_y, end.offset_y, t),
                blur_radius: lerp_f64(start.blur_radius, end.blur_radius, t),
                spread_radius: lerp_f64(start.spread_radius, end.spread_radius, t),
                color: start.color.lerp_rect(end.color, t as f32),
            })
        })
        .collect()
}

fn lerp_gradient(start: &Gradient, end: &Gradient, t: f64) -> Option<Gradient> {
    if start.color_stops.len() != end.color_stops.len()
        || start.extend != end.extend
        || start.hue_direction != end.hue_direction
    {
        return None;
    }

    let kind = match (&start.kind, &end.kind) {
        (GradientKind::Linear(start), GradientKind::Linear(end)) => GradientKind::Linear(LinearGradientData {
            start: start.start.lerp(end.start, t),
            end: start.end.lerp(end.end, t),
        }),
        (GradientKind::Radial(start), GradientKind::Radial(end)) => GradientKind::Radial(RadialGradientData {
            start_center: start.start_center.lerp(end.start_center, t),
            start_radius: lerp(start.start_radius, end.start_radius, t as f32),
            end_center: start.end_center.lerp(end.end_center, t),
            end_radius: lerp(start.end_radius, end.end_radius, t as f32),
        }),
        (GradientKind::Sweep(start), GradientKind::Sweep(end)) => GradientKind::Sweep(SweepGradientData {
            center: start.center.lerp(end.center, t),
            start_angle: lerp(start.start_angle, end.start_angle, t as f32),
            end_angle: lerp(start.end_angle, end.end_angle, t as f32),
        }),
        _ => return None,
    };

    let mut gradient = start.clone();
    gradient.kind = kind;
    for (stop, end) in gradient.color_stops.iter_mut().zip(&end.color_stops) {
        stop.offset = lerp(stop.offset, end.offset, t as f32);
        stop.color = stop.color.lerp_rect(end.color, t as f32);
    }
    Some(gradient)
}
