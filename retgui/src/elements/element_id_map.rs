use std::cell::RefCell;
use std::rc::Weak;

use rustc_hash::FxHashMap;

use crate::elements::ElementInternals;

/// Maps element ids to Weak<Refcell<dyn Element>>

#[derive(Default, Clone)]
pub(crate) struct ElementIdMap {
    map: FxHashMap<u64, Weak<RefCell<dyn ElementInternals>>>,
}

impl ElementIdMap {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert(&mut self, element: &dyn ElementInternals) -> Option<Weak<RefCell<dyn ElementInternals>>> {
        let element_data = element.element_data();
        self.map.insert(element_data.internal_id, element_data.me.clone())
    }

    pub(crate) fn insert_id(
        &mut self,
        id: u64,
        element: Weak<RefCell<dyn ElementInternals>>,
    ) -> Option<Weak<RefCell<dyn ElementInternals>>> {
        self.map.insert(id, element)
    }

    pub(crate) fn remove_id(&mut self, id: u64) -> Option<Weak<RefCell<dyn ElementInternals>>> {
        self.map.remove(&id)
    }

    pub(crate) fn get(&self, id: u64) -> Option<&Weak<RefCell<dyn ElementInternals>>> {
        self.map.get(&id)
    }
}
