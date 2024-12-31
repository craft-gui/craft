use taffy::Position;
use crate::renderer::color::Color;
use crate::style::{AlignItems, Display, FlexDirection, FontStyle, JustifyContent, Overflow, Style, Unit, Weight, Wrap};

pub trait ElementStyles
where
    Self: Sized,
{
    fn styles_mut(&mut self) -> &mut Style;

    fn background(mut self, color: Color) -> Self {
        self.styles_mut().background = color;
        self
    }

    fn margin<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().margin = [top.into(), right.into(), bottom.into(), left.into()];
        self
    }

    fn padding<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().padding = [top.into(), right.into(), bottom.into(), left.into()];
        self
    }

    fn border_width<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().border_width = [top.into(), right.into(), bottom.into(), left.into()];
        self
    }

    fn border_radius<U: Into<f32> + Copy>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().border_radius = [
            (top.into(), top.into()),
            (right.into(), right.into()),
            (bottom.into(), bottom.into()),
            (left.into(), left.into())
        ];
        self
    }

    fn border_color(mut self, color: Color) -> Self {
        self.styles_mut().border_color = [color, color, color, color];
        self
    }
    
    fn border_color_top(mut self, color: Color) -> Self {
        self.styles_mut().border_color[0] = color;
        self
    }
    
    fn border_color_right(mut self, color: Color) -> Self {
        self.styles_mut().border_color[1] = color;
        self
    }
    
    fn border_color_bottom(mut self, color: Color) -> Self {
        self.styles_mut().border_color[2] = color;
        self
    }
    
    fn border_color_left(mut self, color: Color) -> Self {
        self.styles_mut().border_color[3] = color;
        self
    }

    fn display(mut self, display: Display) -> Self {
        self.styles_mut().display = display;
        self
    }

    fn wrap(mut self, wrap: Wrap) -> Self {
        self.styles_mut().wrap = wrap;
        self
    }

    fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.styles_mut().justify_content = Some(justify_content);
        self
    }

    fn align_items(mut self, align_items: AlignItems) -> Self {
        self.styles_mut().align_items = Some(align_items);
        self
    }

    fn flex_direction(mut self, flex_direction: FlexDirection) -> Self {
        self.styles_mut().flex_direction = flex_direction;
        self
    }

    fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.styles_mut().flex_grow = flex_grow;
        self
    }

    fn flex_shrink(mut self, flex_shrink: f32) -> Self {
        self.styles_mut().flex_shrink = flex_shrink;
        self
    }

    fn flex_basis<U: Into<Unit>>(mut self, flex_basis: U) -> Self {
        self.styles_mut().flex_basis = flex_basis.into();
        self
    }

    fn width<U: Into<Unit>>(mut self, width: U) -> Self {
        self.styles_mut().width = width.into();
        self
    }

    fn height<U: Into<Unit>>(mut self, height: U) -> Self {
        self.styles_mut().height = height.into();
        self
    }

    fn max_width<U: Into<Unit>>(mut self, max_width: U) -> Self {
        self.styles_mut().max_width = max_width.into();
        self
    }

    fn max_height<U: Into<Unit>>(mut self, max_height: U) -> Self {
        self.styles_mut().max_height = max_height.into();
        self
    }

    fn min_width<U: Into<Unit>>(mut self, min_width: U) -> Self {
        self.styles_mut().min_width = min_width.into();
        self
    }

    fn min_height<U: Into<Unit>>(mut self, min_height: U) -> Self {
        self.styles_mut().min_height = min_height.into();
        self
    }

    fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.styles_mut().overflow[0] = overflow;
        self
    }

    fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.styles_mut().overflow[1] = overflow;
        self
    }

    fn color(mut self, color: Color) -> Self {
        self.styles_mut().color = color;
        self
    }
    
    fn font_family(mut self, font_family: &str) -> Self {
        self.styles_mut().set_font_family(font_family);
        self
    }

    fn font_size(mut self, font_size: f32) -> Self {
        self.styles_mut().font_size = font_size;
        self
    }

    fn font_weight(mut self, font_weight: Weight) -> Self {
        self.styles_mut().font_weight = font_weight;
        self
    }

    fn font_style(mut self, font_style: FontStyle) -> Self {
        self.styles_mut().font_style = font_style;
        self
    }

    fn overflow(mut self, overflow: Overflow) -> Self {
        self.styles_mut().overflow = [overflow, overflow];
        self
    }

    fn position(mut self, position: Position) -> Self {
        self.styles_mut().position = position;
        self
    }

    fn inset<U: Into<Unit>>(mut self, top: U, right: U, bottom: U, left: U) -> Self {
        self.styles_mut().inset = [top.into(), right.into(), bottom.into(), left.into()];
        self
    }

    fn scrollbar_width(mut self, scrollbar_width: f32) -> Self {
        self.styles_mut().scrollbar_width = scrollbar_width;
        self
    }

    fn box_sizing(mut self, box_sizing: taffy::BoxSizing) -> Self {
        self.styles_mut().box_sizing = box_sizing;
        self
    }

    fn gap<U: Into<Unit> + Clone>(mut self, gap: U) -> Self {
        self.styles_mut().gap = [gap.clone().into(), gap.into()];
        self
    }

    fn row_gap<U: Into<Unit>>(mut self, row_gap: U) -> Self {
        self.styles_mut().gap[0] = row_gap.into();
        self
    }

    fn column_gap<U: Into<Unit>>(mut self, column_gap: U) -> Self {
        self.styles_mut().gap[1] = column_gap.into();
        self
    }
}







impl From<&str> for Unit {
    fn from(s: &str) -> Self {
        let s = s.trim();
        if s.eq_ignore_ascii_case("auto") {
            return Unit::Auto;
        }
        if s.ends_with("px") {
            match s[..s.len() - 2].trim().parse::<f32>() {
                Ok(value) => Unit::Px(value),
                Err(_) => Unit::Auto,
            }
        } else if s.ends_with('%') {
            match s[..s.len() - 1].trim().parse::<f32>() {
                Ok(value) => Unit::Percentage(value),
                Err(_) => Unit::Auto,
            }
        } else {
            panic!("Invalid unit: {}", s);
        }
    }
}