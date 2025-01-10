use crate::elements::element_states::ElementState;

#[derive(Debug, Default, Clone)]
pub struct BaseElementState {
    pub(crate) current_state: ElementState,
}