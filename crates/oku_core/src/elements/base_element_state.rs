use crate::elements::element_states::ElementState;

#[derive(Debug, Default, Clone)]
pub struct BaseElementState {
    #[allow(dead_code)]
    pub(crate) current_state: ElementState,
}