use crate::style::FontFamily;
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
        self.styles_mut().set_background(color);
        self
    }

    fn margin<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().set_margin(TrblRectangle::new(top.into(), right.into(), bottom.into(), left.into()));
        self
    }

    fn padding<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().set_padding(TrblRectangle::new(top.into(), right.into(), bottom.into(), left.into()));
        self
    }

    fn border_width<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().set_border_width(TrblRectangle::new(top.into(), right.into(), bottom.into(), left.into()));
        self
    }

    fn border_radius<U: IntoF32 + Copy>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().set_border_radius([
            (top.into_f32(), top.into_f32()),
            (right.into_f32(), right.into_f32()),
            (bottom.into_f32(), bottom.into_f32()),
            (left.into_f32(), left.into_f32()),
        ]);
        self
    }

    fn border_color(mut self, color: Color) -> Self {
        self.styles_mut().set_border_color(TrblRectangle::new_all(color));
        self
    }

    fn border_color_top(mut self, color: Color) -> Self {
        let mut border = self.styles_mut().border_color();
        border.top = color;
        self.styles_mut().set_border_color(border);
        self
    }

    fn border_color_right(mut self, color: Color) -> Self {
        let mut border = self.styles_mut().border_color();
        border.right = color;
        self.styles_mut().set_border_color(border);
        self
    }

    fn border_color_bottom(mut self, color: Color) -> Self {
        let mut border = self.styles_mut().border_color();
        border.bottom = color;
        self.styles_mut().set_border_color(border);
        self
    }

    fn border_color_left(mut self, color: Color) -> Self {
        let mut border = self.styles_mut().border_color();
        border.left = color;
        self.styles_mut().set_border_color(border);
        self
    }

    fn display(mut self, display: Display) -> Self {
        self.styles_mut().set_display(display);
        self
    }

    fn wrap(mut self, wrap: Wrap) -> Self {
        self.styles_mut().set_wrap(wrap);
        self
    }

    fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.styles_mut().set_justify_content(Some(justify_content));
        self
    }

    fn align_items(mut self, align_items: AlignItems) -> Self {
        self.styles_mut().set_align_items(Some(align_items));
        self
    }

    fn flex_direction(mut self, flex_direction: FlexDirection) -> Self {
        self.styles_mut().set_flex_direction(flex_direction);
        self
    }

    fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.styles_mut().set_flex_grow(flex_grow);
        self
    }

    fn flex_shrink(mut self, flex_shrink: f32) -> Self {
        self.styles_mut().set_flex_shrink(flex_shrink);
        self
    }

    fn flex_basis<U: Into<Unit>>(mut self, flex_basis: U) -> Self {
        self.styles_mut().set_flex_basis(flex_basis.into());
        self
    }

    fn width<U: Into<Unit>>(mut self, width: U) -> Self {
        self.styles_mut().set_width(width.into());
        self
    }

    fn height<U: Into<Unit>>(mut self, height: U) -> Self {
        self.styles_mut().set_height(height.into());
        self
    }

    fn max_width<U: Into<Unit>>(mut self, max_width: U) -> Self {
        self.styles_mut().set_max_width(max_width.into());
        self
    }

    fn max_height<U: Into<Unit>>(mut self, max_height: U) -> Self {
        self.styles_mut().set_max_height(max_height.into());
        self
    }

    fn min_width<U: Into<Unit>>(mut self, min_width: U) -> Self {
        self.styles_mut().set_min_width(min_width.into());
        self
    }

    fn min_height<U: Into<Unit>>(mut self, min_height: U) -> Self {
        self.styles_mut().set_min_height(min_height.into());
        self
    }

    fn overflow_x(mut self, overflow: Overflow) -> Self {
        let mut val = self.styles_mut().overflow();
        val[0] = overflow;
        self.styles_mut().set_overflow(val);
        self
    }

    fn overflow_y(mut self, overflow: Overflow) -> Self {
        let mut val = self.styles_mut().overflow();
        val[1] = overflow;
        self.styles_mut().set_overflow(val);
        self
    }

    fn color(mut self, color: Color) -> Self {
        self.styles_mut().set_color(color);
        self
    }

    fn font_family(mut self, font_family: &str) -> Self {
        self.styles_mut().set_font_family(FontFamily::new(font_family));
        self
    }

    fn selection_color(mut self, color: Color) -> Self {
        self.styles_mut().set_selection_color(color);
        self
    }

    fn cursor_color(mut self, color: Color) -> Self {
        self.styles_mut().set_cursor_color(Some(color));
        self
    }
    
    fn font_size<U: IntoF32 + Copy>(mut self, font_size: U) -> Self {
        self.styles_mut().set_font_size(font_size.into_f32());
        self
    }

    fn font_weight(mut self, font_weight: Weight) -> Self {
        self.styles_mut().set_font_weight(font_weight);
        self
    }

    fn underline(mut self, thickness: f32, color: Color, offset: Option<f32>) -> Self {
        self.styles_mut().set_underline(Some(Underline {
            thickness: Some(thickness),
            color,
            offset,
        }));
        self
    }

    fn font_style(mut self, font_style: FontStyle) -> Self {
        self.styles_mut().set_font_style(font_style);
        self
    }

    fn overflow(mut self, overflow: Overflow) -> Self {
        self.styles_mut().set_overflow([overflow, overflow]);
        self
    }

    fn position(mut self, position: Position) -> Self {
        self.styles_mut().set_position(position);
        self
    }

    fn inset<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().set_inset(TrblRectangle::new(top.into(), right.into(), bottom.into(), left.into()));
        self
    }

    fn scrollbar_width(mut self, scrollbar_width: f32) -> Self {
        self.styles_mut().set_scrollbar_width(scrollbar_width);
        self
    }

    fn box_sizing(mut self, box_sizing: taffy::BoxSizing) -> Self {
        self.styles_mut().set_box_sizing(box_sizing);
        self
    }

    fn gap<U: Into<Unit> + Clone>(mut self, gap: U) -> Self {
        self.styles_mut().set_gap([gap.clone().into(), gap.into()]);
        self
    }

    fn row_gap<U: Into<Unit>>(mut self, row_gap: U) -> Self {
        let mut val = self.styles_mut().gap();
        val[0] = row_gap.into();
        self.styles_mut().set_gap(val);
        self
    }

    fn column_gap<U: Into<Unit>>(mut self, column_gap: U) -> Self {
        let mut val = self.styles_mut().gap();
        val[1] = column_gap.into();
        self.styles_mut().set_gap(val);
        self
    }

    fn scrollbar_color(mut self, scroll_thumb_color: Color, scroll_track_color: Color) -> Self {
        let mut colors = self.styles_mut().scrollbar_color();
        colors.thumb_color = scroll_thumb_color;
        colors.track_color = scroll_track_color;
        self.styles_mut().set_scrollbar_color(colors);
        self
    }

    fn visible(mut self, visible: bool) -> Self {
        self.styles_mut().set_visible(visible);
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
