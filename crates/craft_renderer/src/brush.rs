use peniko::{BrushRef, Color, Gradient};

#[derive(Clone, Debug)]
pub enum Brush {
    Color(Color),
    Gradient(Gradient),
}

impl<'a> From<&'a Brush> for BrushRef<'a> {
    fn from(brush: &'a Brush) -> Self {
        match brush {
            Brush::Color(color) => Self::Solid(*color),
            Brush::Gradient(gradient) => Self::Gradient(gradient),
        }
    }
}
