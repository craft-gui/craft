use std::cell::RefCell;
use std::rc::Rc;

use craft_retained::elements::{AsElement, ElementInternals, WindowInternal};
use craft_retained::{RendererType, WinitWindow};
use craft_retained::winit::event_loop::ActiveEventLoop;
use crate::elements::element::Element;

#[derive(Clone)]
pub struct Window {
    pub inner: craft_retained::elements::Window,
}

impl Element for Window {}

impl AsElement for Window {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.inner.clone()
    }
}

impl Window {
    pub fn new(title: &str) -> Self{
        Self {
            inner: craft_retained::elements::Window::new(title),
        }
    }

    pub fn new_advanced<F>(window_fn: F, renderer_type: RendererType) -> Self
    where
        F: FnMut(&ActiveEventLoop) -> WinitWindow + 'static,
    {
        Self {
            inner: craft_retained::elements::Window::new_advanced(window_fn, renderer_type),
        }
    }
}