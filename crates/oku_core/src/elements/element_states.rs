#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Disabled,
    Focused,
}