use crate::components::ComponentId;
use std::any::Any;
use std::collections::HashMap;
use crate::elements::base_element_state::BaseElementState;

#[derive(Debug)]
pub struct ElementStateStoreItem {
    pub base: BaseElementState,
    pub data: Box<dyn Any + Send>   
}

#[derive(Default)]
pub struct ElementStateStore {
    pub storage: HashMap<ComponentId, ElementStateStoreItem>,
}
