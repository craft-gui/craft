#[derive(Copy, Clone, Debug, Default)]
pub struct TrblRectangle<T> where T: Copy {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> TrblRectangle<T> where T: Copy {
    pub(crate) fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self { 
            top,
            right,
            bottom,
            left 
        }
    }
    pub(crate) fn new_all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value
        }
    }
    
    pub(crate) fn to_array(self) -> [T; 4] {
        [self.top, self.right, self.bottom, self.left]
    }
}


impl From<taffy::Rect<f32>> for TrblRectangle<f32> {
    fn from(rect: taffy::Rect<f32>) -> Self {
        TrblRectangle::new(rect.top, rect.right, rect.bottom, rect.left)
    }
}
