#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Disabled,
    Focused,
}
