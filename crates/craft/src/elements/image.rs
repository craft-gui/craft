use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::ResourceId;
use craft_retained::elements::{AsElement, ElementInternals};

use crate::elements::Element;
use crate::signals::Bindable;

#[derive(Clone)]
pub struct Image {
    pub inner: craft_retained::elements::Image,
}

impl AsElement for Image {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.inner.clone()
    }
}

impl Element for Image {}

impl Image {
    pub fn new(resource_id: impl Bindable<ResourceId>) -> Self {
        let inner = craft_retained::elements::Image::dummy();
        let inner_clone = inner.clone();
        resource_id.bind(move |resource_id| {
            inner_clone.clone().resource_id(resource_id);
        });
        Self { inner }
    }

    pub fn resource_id(self, resource_id: impl Bindable<ResourceId>) -> Self {
        let element = self.clone();
        resource_id.bind(move |resource_id| {
            element.clone().inner.resource_id(resource_id);
        });
        self
    }

    pub fn get_resource_id(&self) -> ResourceId {
        self.inner.get_resource_id()
    }
}
