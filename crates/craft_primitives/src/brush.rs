use peniko::{Color};
use crate::gradient::Gradient;

#[derive(Clone, Debug, PartialEq)]
pub enum Brush {
    Color(Color),
    Gradient(Gradient)
}

impl Default for Brush {
    fn default() -> Self {
        Brush::Color(Color::BLACK)
    }
}