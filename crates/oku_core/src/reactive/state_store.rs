use crate::components::ComponentId;
use std::any::Any;
use std::collections::HashMap;

pub type StateStoreItem = dyn Any + Send;

#[derive(Default)]
pub struct StateStore {
    pub storage: HashMap<ComponentId, Box<StateStoreItem>>,
}
