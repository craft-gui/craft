#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrblRectangle<T> where
    T: Clone {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> TrblRectangle<T>
where
    T: Clone + PartialEq,
{
    #[inline(always)]
    pub const fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn new_all(value: T) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn to_array(self) -> [T; 4]  {
        [self.top.clone(), self.right.clone(), self.bottom.clone(), self.left.clone()]
    }

    pub fn are_edges_uniform(&self) -> bool {
        self.top == self.right && self.right == self.bottom && self.bottom == self.left
    }
}

/*impl From<taffy::Rect<f32>> for TrblRectangle<f32> {
    fn from(rect: taffy::Rect<f32>) -> Self {
        TrblRectangle::new(rect.top, rect.right, rect.bottom, rect.left)
    }
}

impl From<taffy::Rect<f64>> for TrblRectangle<f64> {
    fn from(rect: taffy::Rect<f64>) -> Self {
        TrblRectangle::new(rect.top, rect.right, rect.bottom, rect.left)
    }
}
*/
