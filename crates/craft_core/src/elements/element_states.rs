#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Hash)]
pub enum ElementState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Disabled,
    Focused,
}
