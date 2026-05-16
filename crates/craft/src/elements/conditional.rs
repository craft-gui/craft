use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::elements::{AsElement, DynElement, Element as RetainedElement, ElementInternals};

use crate::elements::Element;
use crate::signals::Signal;

impl Element for Conditional {}

impl AsElement for Conditional {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.inner.clone()
    }
}

#[derive(Clone)]
pub struct Conditional {
    pub inner: craft_retained::elements::Container,
    pub signal: Signal<bool>,
    pub hidden_child: Rc<RefCell<Option<DynElement>>>,
}

impl Conditional {
    // TODO use active and disabled as params
    pub fn new(signal: Signal<bool>) -> Self {
        let inner = craft_retained::elements::Container::new();

        let inner_clone = inner.clone();

        let signal_clone = signal.clone();
        let hidden_child: Rc<RefCell<Option<DynElement>>> = Rc::new(RefCell::new(None));
        let hidden_child_clone = hidden_child.clone();
        let runner = Rc::new(move || {
            let val = signal_clone.get();
            let container = inner_clone.clone();
            if val {
                if let Some(child) = hidden_child_clone.borrow_mut().take() {
                    container.push(child);
                }
            } else {
                if let Ok(first_child) = container.get_first_child() {
                    container.remove_child(first_child.clone()).unwrap();
                    hidden_child_clone.replace(Some(first_child));
                }
            }
        });
        runner();

        signal.subscribe(runner);

        Self {
            inner,
            signal,
            hidden_child,
        }
    }
}
