#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorBrush {
    pub color: crate::Color,
}

impl ColorBrush {
    pub fn new(color: peniko::Color) -> Self {
        Self { color }
    }
}

impl Default for ColorBrush {
    fn default() -> Self {
        Self {
            color: peniko::Color::BLACK,
        }
    }
}
