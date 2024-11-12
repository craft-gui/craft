use std::any::Any;
use std::collections::HashMap;
use crate::components::ComponentId;

pub type StateStoreItem = dyn Any + Send;

#[derive(Default)]
pub struct StateStore {
    pub storage: HashMap<ComponentId, Box<StateStoreItem>>
}