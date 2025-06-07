use crate::geometry::TrblRectangle;
use crate::renderer::color::Color;
use crate::style::{AlignItems, Display, FlexDirection, FontStyle, JustifyContent, Overflow, Style, Underline, Unit, Weight, Wrap};
use taffy::Position;

pub trait ElementStyles
where
    Self: Sized,
{
    fn styles_mut(&mut self) -> &mut Style;

    fn background(mut self, color: Color) -> Self {
        *self.styles_mut().background_mut() = color;
        self
    }

    fn margin<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        *self.styles_mut().margin_mut() = TrblRectangle::new(top.into(), right.into(), bottom.into(), left.into());
        self
    }

    fn padding<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        *self.styles_mut().padding_mut() = TrblRectangle::new(top.into(), right.into(), bottom.into(), left.into());
        self
    }

    fn border_width<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        *self.styles_mut().border_width_mut() =
            TrblRectangle::new(top.into(), right.into(), bottom.into(), left.into());
        self
    }

    fn border_radius<U: IntoF32 + Copy>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        *self.styles_mut().border_radius_mut() = [
            (top.into_f32(), top.into_f32()),
            (right.into_f32(), right.into_f32()),
            (bottom.into_f32(), bottom.into_f32()),
            (left.into_f32(), left.into_f32()),
        ];
        self
    }

    fn border_color(mut self, color: Color) -> Self {
        *self.styles_mut().border_color_mut() = TrblRectangle::new_all(color);
        self
    }

    fn border_color_top(mut self, color: Color) -> Self {
        self.styles_mut().border_color_mut().top = color;
        self
    }

    fn border_color_right(mut self, color: Color) -> Self {
        self.styles_mut().border_color_mut().right = color;
        self
    }

    fn border_color_bottom(mut self, color: Color) -> Self {
        self.styles_mut().border_color_mut().bottom = color;
        self
    }

    fn border_color_left(mut self, color: Color) -> Self {
        self.styles_mut().border_color_mut().left = color;
        self
    }

    fn display(mut self, display: Display) -> Self {
        *self.styles_mut().display_mut() = display;
        self
    }

    fn wrap(mut self, wrap: Wrap) -> Self {
        *self.styles_mut().wrap_mut() = wrap;
        self
    }

    fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        *self.styles_mut().justify_content_mut() = Some(justify_content);
        self
    }

    fn align_items(mut self, align_items: AlignItems) -> Self {
        *self.styles_mut().align_items_mut() = Some(align_items);
        self
    }

    fn flex_direction(mut self, flex_direction: FlexDirection) -> Self {
        *self.styles_mut().flex_direction_mut() = flex_direction;
        self
    }

    fn flex_grow(mut self, flex_grow: f32) -> Self {
        *self.styles_mut().flex_grow_mut() = flex_grow;
        self
    }

    fn flex_shrink(mut self, flex_shrink: f32) -> Self {
        *self.styles_mut().flex_shrink_mut() = flex_shrink;
        self
    }

    fn flex_basis<U: Into<Unit>>(mut self, flex_basis: U) -> Self {
        *self.styles_mut().flex_basis_mut() = flex_basis.into();
        self
    }

    fn width<U: Into<Unit>>(mut self, width: U) -> Self {
        *self.styles_mut().width_mut() = width.into();
        self
    }

    fn height<U: Into<Unit>>(mut self, height: U) -> Self {
        *self.styles_mut().height_mut() = height.into();
        self
    }

    fn max_width<U: Into<Unit>>(mut self, max_width: U) -> Self {
        *self.styles_mut().max_width_mut() = max_width.into();
        self
    }

    fn max_height<U: Into<Unit>>(mut self, max_height: U) -> Self {
        *self.styles_mut().max_height_mut() = max_height.into();
        self
    }

    fn min_width<U: Into<Unit>>(mut self, min_width: U) -> Self {
        *self.styles_mut().min_width_mut() = min_width.into();
        self
    }

    fn min_height<U: Into<Unit>>(mut self, min_height: U) -> Self {
        *self.styles_mut().min_height_mut() = min_height.into();
        self
    }

    fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.styles_mut().overflow_mut()[0] = overflow;
        self
    }

    fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.styles_mut().overflow_mut()[1] = overflow;
        self
    }

    fn color(mut self, color: Color) -> Self {
        *self.styles_mut().color_mut() = color;
        self
    }

    fn font_family(mut self, font_family: &str) -> Self {
        self.styles_mut().set_font_family(font_family);
        self
    }

    fn font_size<U: IntoF32 + Copy>(mut self, font_size: U) -> Self {
        *self.styles_mut().font_size_mut() = font_size.into_f32();
        self
    }

    fn font_weight(mut self, font_weight: Weight) -> Self {
        *self.styles_mut().font_weight_mut() = font_weight;
        self
    }
    
    fn underline(mut self, thickness: f32, color: Color, offset: Option<f32>) -> Self {
        *self.styles_mut().underline_mut() = Some(
            Underline {
                thickness: Some(thickness),
                color,
                offset
            }
        );
        self
    }

    fn font_style(mut self, font_style: FontStyle) -> Self {
        *self.styles_mut().font_style_mut() = font_style;
        self
    }

    fn overflow(mut self, overflow: Overflow) -> Self {
        *self.styles_mut().overflow_mut() = [overflow, overflow];
        self
    }

    fn position(mut self, position: Position) -> Self {
        *self.styles_mut().position_mut() = position;
        self
    }

    fn inset<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        *self.styles_mut().inset_mut() = TrblRectangle::new(top.into(), right.into(), bottom.into(), left.into());
        self
    }

    fn scrollbar_width(mut self, scrollbar_width: f32) -> Self {
        *self.styles_mut().scrollbar_width_mut() = scrollbar_width;
        self
    }

    fn box_sizing(mut self, box_sizing: taffy::BoxSizing) -> Self {
        *self.styles_mut().box_sizing_mut() = box_sizing;
        self
    }

    fn gap<U: Into<Unit> + Clone>(mut self, gap: U) -> Self {
        *self.styles_mut().gap_mut() = [gap.clone().into(), gap.into()];
        self
    }

    fn row_gap<U: Into<Unit>>(mut self, row_gap: U) -> Self {
        self.styles_mut().gap_mut()[0] = row_gap.into();
        self
    }

    fn column_gap<U: Into<Unit>>(mut self, column_gap: U) -> Self {
        self.styles_mut().gap_mut()[1] = column_gap.into();
        self
    }

    fn scrollbar_color(mut self, scroll_thumb_color: Color, scroll_track_color: Color) -> Self {
        let colors = self.styles_mut().scrollbar_color_mut();
        colors.thumb_color = scroll_thumb_color;
        colors.track_color = scroll_track_color;
        self
    }

    fn visible(mut self, visible: bool) -> Self {
        let visible_mut = self.styles_mut().visible_mut();
        *visible_mut = visible;
        self
    }
}

impl From<&str> for Unit {
    fn from(s: &str) -> Self {
        let s = s.trim();
        if s.eq_ignore_ascii_case("auto") {
            return Unit::Auto;
        }
        if let Some(stripped) = s.strip_suffix("px") {
            match stripped.trim().parse::<f32>() {
                Ok(value) => Unit::Px(value),
                Err(_) => Unit::Auto,
            }
        } else if let Some(stripped) = s.strip_suffix('%') {
            match stripped.trim().parse::<f32>() {
                Ok(value) => Unit::Percentage(value),
                Err(_) => Unit::Auto,
            }
        } else {
            panic!("Invalid unit: {}", s);
        }
    }
}

impl From<i32> for Unit {
    fn from(value: i32) -> Self {
        Unit::Px(value as f32)
    }
}

pub trait IntoF32 {
    fn into_f32(self) -> f32;
}

impl IntoF32 for f32 {
    fn into_f32(self) -> f32 {
        self
    }
}

impl IntoF32 for i32 {
    fn into_f32(self) -> f32 {
        self as f32
    }
}
