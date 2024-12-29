use crate::geometry::corner::Corner;

#[repr(usize)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Side {
    Top = 0,
    Right = 1,
    Bottom = 2,
    Left = 3,
}

impl Side {
    pub(crate) fn next_clockwise(self) -> Self {
        // top -> right
        // right -> bottom
        // bottom -> left
        // left -> top
        unsafe {
            std::mem::transmute(((self as usize) + 1) & 3)
        }
    }

    pub(crate) fn as_corner(self) -> Corner {
        // top -> top_left
        // right -> top_right
        // bottom -> bottom_right
        // left -> bottom_left
        unsafe { std::mem::transmute(self) }
    }
}
