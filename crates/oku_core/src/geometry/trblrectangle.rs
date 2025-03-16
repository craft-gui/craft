#[derive(Copy, Clone, Debug, Default)]
pub struct TrblRectangle {
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

impl From<taffy::Rect<f32>> for TrblRectangle {
    fn from(rect: taffy::Rect<f32>) -> Self {
        TrblRectangle::new(rect.top, rect.right, rect.bottom, rect.left)
    }
}
