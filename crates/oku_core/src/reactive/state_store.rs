use crate::components::ComponentId;
use std::any::Any;
use std::collections::{HashMap, HashSet};

pub type StateStoreItem = dyn Any + Send;

#[derive(Default)]
pub struct StateStore {
    pub storage: HashMap<ComponentId, Box<StateStoreItem>>,
}

impl StateStore {
    pub(crate) fn remove_unused_state(&mut self, old_component_ids: &HashSet<ComponentId>, new_component_ids: &HashSet<ComponentId>) {
        // Get the old component ids that aren't in new_component_ids.
        old_component_ids.difference(new_component_ids).for_each(|component_id| {
            self.storage.remove(component_id);
        });
    }
}