use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use craft_retained::elements::{AsElement, ElementInternals};

use crate::elements::element::Element;
use crate::signals::Bindable;

#[derive(Clone)]
pub struct Text {
    pub inner: craft_retained::elements::Text,
}

impl Element for Text {}

impl AsElement for Text {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }
}

impl Text {
    pub fn new(text: &str) -> Self {
        Self {
            inner: craft_retained::elements::Text::new(text),
        }
    }

    pub fn text(&self, value: impl Bindable<String>) -> Self {
        let element = self.inner.clone();

        value.bind(move |string_val| {
            element.clone().text(string_val.as_str());
        });

        self.clone()
    }

    pub fn selectable(&self, value: impl Bindable<bool>) -> Self {
        let element = self.inner.clone();
        value.bind(move |value| {
            element.clone().selectable(value);
        });
        self.clone()
    }
}
