#[derive(Clone, Copy)]
#[repr(usize)]
pub enum CornerSide {
    Top = 0,
    Bottom = 1,
}

impl CornerSide {

    pub(crate) fn next(self) -> Self {
        unsafe {
            std::mem::transmute((self as usize + 1) & 1)
        }
    }

}