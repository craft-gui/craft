use kurbo::Affine;
use crate::geometry::{Border, Margin, Padding, Point, Rectangle, Size};

/// An element's box roughly analogous to CSS's box-model.
#[derive(Clone, Copy, Debug, Default)]
pub struct ElementBox {
    pub margin: Margin,
    pub border: Border,
    pub padding: Padding,
    pub position: Point,
    pub size: Size<f32>,
}

impl ElementBox {
    pub fn transform(&self, transform: Affine) -> Self {
        let mut transformed_box = *self;
        transformed_box.position = transform * self.position;
        transformed_box
    }

    pub fn margin_rectangle_position(&self) -> Point {
        Point::new(self.position.x - self.margin.left as f64, self.position.y - self.margin.top as f64)
    }

    pub fn margin_rectangle_size(&self) -> Size<f32> {
        let margin_width = self.size.width + self.margin.left + self.margin.right;
        let margin_height = self.size.height + self.margin.top + self.margin.bottom;
        Size {
            width: margin_width,
            height: margin_height,
        }
    }

    pub fn margin_rectangle(&self) -> Rectangle {
        let margin_position = self.margin_rectangle_position();
        let margin_size = self.margin_rectangle_size();

        Rectangle {
            x: margin_position.x as f32,
            y: margin_position.y as f32,
            width: margin_size.width,
            height: margin_size.height,
        }
    }

    pub fn border_rectangle_size(&self) -> Size<f32> {
        Size {
            width: self.size.width,
            height: self.size.height,
        }
    }

    pub fn border_rectangle_position(&self) -> Point {
        Point::new(self.position.x, self.position.y)
    }

    pub fn border_rectangle(&self) -> Rectangle {
        let border_position = self.border_rectangle_position();
        let border_size = self.border_rectangle_size();

        Rectangle {
            x: border_position.x as f32,
            y: border_position.y as f32,
            width: border_size.width,
            height: border_size.height,
        }
    }

    pub fn padding_rectangle_size(&self) -> Size<f32> {
        let padding_width = self.size.width - self.border.left - self.border.right;
        let padding_height = self.size.height - self.border.top - self.border.bottom;
        Size {
            width: padding_width,
            height: padding_height,
        }
    }

    pub fn padding_rectangle_position(&self) -> Point {
        let padding_x = self.position.x + self.border.left as f64;
        let padding_y = self.position.y + self.border.top as f64;
        Point::new(padding_x, padding_y)
    }

    pub fn padding_rectangle(&self) -> Rectangle {
        let padding_position = self.padding_rectangle_position();
        let padding_size = self.padding_rectangle_size();

        Rectangle {
            x: padding_position.x as f32,
            y: padding_position.y as f32,
            width: padding_size.width,
            height: padding_size.height,
        }
    }

    pub fn content_rectangle_size(&self) -> Size<f32> {
        let content_width =
            self.size.width - self.padding.left - self.padding.right - self.border.left - self.border.right;
        let content_height =
            self.size.height - self.padding.top - self.padding.bottom - self.border.top - self.border.bottom;
        Size::new(content_width, content_height)
    }

    pub fn content_rectangle_position(&self) -> Point {
        let content_x = self.position.x as f32 + self.border.left + self.padding.left;
        let content_y = self.position.y as f32 + self.border.top + self.padding.top;
        Point::new(content_x as f64, content_y as f64)
    }

    pub fn content_rectangle(&self) -> Rectangle {
        let content_position = self.content_rectangle_position();
        let content_size = self.content_rectangle_size();

        Rectangle::new(content_position.x as f32, content_position.y as f32, content_size.width, content_size.height)
    }
}
