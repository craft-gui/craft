use crate::components::{ComponentId, FocusAction};
use crate::elements::base_element_state::BaseElementState;
use std::any::Any;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ElementStateStoreItem {
    pub base: BaseElementState,
    pub data: Box<dyn Any + Send>,
}

#[derive(Default)]
pub struct ElementStateStore {
    pub storage: HashMap<ComponentId, ElementStateStoreItem>,
}

impl ElementStateStore {
    pub(crate) fn update_element_focus(&mut self, focus: FocusAction) {
        match focus {
            FocusAction::None => {}
            FocusAction::Set(id) => {
                for (element_id, value) in self.storage.iter_mut() {
                    value.base.focused = *element_id == id;
                }
            }
            FocusAction::Unset => {
                for value in self.storage.values_mut() {
                    value.base.focused = false;
                }
            }
        }
    }

    pub(crate) fn remove_unused_state(
        &mut self,
        old_element_ids: &HashSet<ComponentId>,
        new_element_ids: &HashSet<ComponentId>,
    ) {
        // Get the old element ids that aren't in new_element_ids.
        old_element_ids.difference(new_element_ids).for_each(|element_id| {
            self.storage.remove(element_id);
        });
    }
}
