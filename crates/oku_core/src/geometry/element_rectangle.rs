use crate::geometry::{Border, Margin, Padding, Point, Rectangle, Size};

#[derive(Clone, Copy, Debug, Default)]
pub struct ElementRectangle {
    pub margin: Margin,
    pub border: Border,
    pub padding: Padding,
    pub position: Point,
    pub size: Size,
}

impl ElementRectangle {
    pub fn transform(&self, transform: glam::Mat4) -> Self {
        let mut transformed_box = *self;
        let transformed_xy = transform.mul_vec4(glam::vec4(self.position.x, self.position.y, 1.0, 1.0));
        transformed_box.position = Point::new(transformed_xy.x, transformed_xy.y);
        transformed_box
    }

    pub fn margin_rectangle_position(&self) -> Point {
        Point::new(self.position.x - self.margin.left, self.position.y - self.margin.top)
    }

    pub fn margin_rectangle_size(&self) -> Size {
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
            x: margin_position.x,
            y: margin_position.y,
            width: margin_size.width,
            height: margin_size.height,
        }
    }

    pub fn border_rectangle_size(&self) -> Size {
        Size {
            width: self.size.width,
            height: self.size.height,
        }
    }

    pub fn border_rectangle_position(&self) -> Point {
        Point::new(self.position.x, self.position.y)
    }

    pub fn border_rectangle(&self) -> Rectangle {
        let border_position = self.padding_rectangle_position();
        let border_size = self.padding_rectangle_size();

        Rectangle {
            x: border_position.x,
            y: border_position.y,
            width: border_size.width,
            height: border_size.height,
        }
    }

    pub fn padding_rectangle_size(&self) -> Size {
        let padding_width = self.size.width - self.border.left - self.border.right;
        let padding_height = self.size.height - self.border.top - self.border.bottom;
        Size {
            width: padding_width,
            height: padding_height,
        }
    }

    pub fn padding_rectangle_position(&self) -> Point {
        let padding_x = self.position.x + self.border.left;
        let padding_y = self.position.y + self.border.top;
        Point::new(padding_x, padding_y)
    }

    pub fn padding_rectangle(&self) -> Rectangle {
        let padding_position = self.padding_rectangle_position();
        let padding_size = self.padding_rectangle_size();

        Rectangle {
            x: padding_position.x,
            y: padding_position.y,
            width: padding_size.width,
            height: padding_size.height,
        }
    }

    pub fn content_rectangle_size(&self) -> Size {
        let content_width =
            self.size.width - self.padding.left - self.padding.right - self.border.left - self.border.right;
        let content_height =
            self.size.height - self.padding.top - self.padding.bottom - self.border.top - self.border.bottom;
        Size::new(content_width, content_height)
    }

    pub fn content_rectangle_position(&self) -> Point {
        let content_x = self.position.x + self.border.left + self.padding.left;
        let content_y = self.position.y + self.border.top + self.padding.top;
        Point::new(content_x, content_y)
    }

    pub fn content_rectangle(&self) -> Rectangle {
        let content_position = self.content_rectangle_position();
        let content_size = self.content_rectangle_size();

        Rectangle::new(content_position.x, content_position.y, content_size.width, content_size.height)
    }
}
