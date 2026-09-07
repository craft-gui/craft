//! A type-erased handle to an element stored in [`App`](crate::App).

use slotmap::DefaultKey;

use crate::elements::Element;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DynElement {
    key: DefaultKey,
    store_id: u64,
}

impl Element for DynElement {
    fn as_dyn_element(&self) -> DynElement {
        *self
    }
}

impl DynElement {
    pub(crate) const fn new(element: DynElement) -> Self {
        element
    }

    pub(crate) const fn from_key(key: DefaultKey, store_id: u64) -> Self {
        Self { key, store_id }
    }

    pub(crate) const fn key(self) -> DefaultKey {
        self.key
    }

    pub(crate) const fn store_id(self) -> u64 {
        self.store_id
    }
}
