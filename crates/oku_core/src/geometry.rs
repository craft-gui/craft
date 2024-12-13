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