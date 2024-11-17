use taffy::Position;
use crate::engine::renderer::color::Color;
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

    fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.styles_mut().margin = [top, right, bottom, left];
        self
    }

    fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.styles_mut().padding = [top, right, bottom, left];
        self
    }

    fn border(mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.styles_mut().border = [top, right, bottom, left];
        self
    }

    fn border_color(mut self, border_color: Color) -> Self {
        self.styles_mut().border_color = border_color;
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

    fn flex_basis(mut self, flex_basis: Unit) -> Self {
        self.styles_mut().flex_basis = flex_basis;
        self
    }

    fn width(mut self, width: Unit) -> Self {
        self.styles_mut().width = width;
        self
    }

    fn height(mut self, height: Unit) -> Self {
        self.styles_mut().height = height;
        self
    }

    fn max_width(mut self, max_width: Unit) -> Self {
        self.styles_mut().max_width = max_width;
        self
    }

    fn max_height(mut self, max_height: Unit) -> Self {
        self.styles_mut().max_height = max_height;
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
    
    fn inset(mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.styles_mut().inset = [top, right, bottom, left];
        self
    }
}
