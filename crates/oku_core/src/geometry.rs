use crate::renderer::renderer::Rectangle;

#[derive(Clone, Debug, Default)]
pub struct Position {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Size {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

// Top Right Bottom Left Rectangle
#[derive(Clone, Debug, Default)]
pub struct TrblRectangle{
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl TrblRectangle {
    pub(crate) fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

pub type Border = TrblRectangle;
pub type Padding = TrblRectangle;
pub type Margin = TrblRectangle;

#[derive(Clone, Debug, Default)]
pub struct LayeredRectangle {
    pub margin: Margin,
    pub border: Border,
    pub padding: Padding,
    pub position: Position,
    pub size: Size,
}

impl LayeredRectangle {
    pub fn margin_rectangle_position(&self) -> Position {
        Position {
            x: self.position.x - self.margin.left,
            y: self.position.y - self.margin.top,
            z: 1.0,
        }
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

    pub fn border_rectangle_position(&self) -> Position {
        Position {
            x: self.position.x,
            y: self.position.y,
            z: 1.0,
        }
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

    pub fn padding_rectangle_position(&self) -> Position {
        let padding_x = self.position.x + self.border.left;
        let padding_y = self.position.y + self.border.top;
        Position {
            x: padding_x,
            y: padding_y,
            z: 1.0,
        }
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
        let content_width = self.size.width - self.padding.left - self.padding.right - self.border.left - self.border.right;
        let content_height = self.size.height - self.padding.top - self.padding.bottom - self.border.top - self.border.bottom;
        Size {
            width: content_width,
            height: content_height,
        }
    }
    
    pub fn content_rectangle_position(&self) -> Position {
        let content_x = self.position.x + self.border.left + self.padding.left;
        let content_y = self.position.y + self.border.top + self.padding.top;
        Position {
            x: content_x,
            y: content_y,
            z: 1.0,
        }
    }
    
    pub fn content_rectangle(&self) -> Rectangle {
        let content_position = self.content_rectangle_position();
        let content_size = self.content_rectangle_size();

        Rectangle {
            x: content_position.x,
            y: content_position.y,
            width: content_size.width,
            height: content_size.height,
        }
    }
}