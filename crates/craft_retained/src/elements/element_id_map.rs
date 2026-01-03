use std::cell::RefCell;
use std::rc::Weak;

use rustc_hash::FxHashMap;

use crate::elements::Element;

/// Maps element ids to Weak<Refcell<dyn Element>>

#[derive(Default, Clone)]
pub struct ElementIdMap {
    map: FxHashMap<u64, Weak<RefCell<dyn Element>>>,
}

impl ElementIdMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, element: &dyn Element) -> Option<Weak<RefCell<dyn Element>>> {
        let element_data = element.element_data();
        self.map.insert(element_data.internal_id, element_data.me.clone())
    }

    pub fn insert_id(&mut self, id: u64, element: Weak<RefCell<dyn Element>>) -> Option<Weak<RefCell<dyn Element>>> {
        self.map.insert(id, element)
    }

    pub fn remove(&mut self, element: &dyn Element) -> Option<Weak<RefCell<dyn Element>>> {
        self.map.remove(&element.element_data().internal_id)
    }

    pub fn remove_id(&mut self, id: u64) -> Option<Weak<RefCell<dyn Element>>> {
        self.map.remove(&id)
    }

    pub fn get(&self, id: u64) -> Option<&Weak<RefCell<dyn Element>>> {
        self.map.get(&id)
    }
}
