//! Stores one or more elements.

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, Element, ElementInternals, scrollable};
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;

use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

#[derive(Clone)]
pub struct Container {
    pub inner: Rc<RefCell<ContainerInner>>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub struct ContainerInner {
    element_data: ElementData,
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Container {}

impl Drop for ContainerInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Container {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }
}

impl crate::elements::ElementData for ContainerInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for ContainerInner {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        clone_element::<Self, _>(self, |_, _| None)
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(self, gummy_tree, z_index, scale_factor);
    }

    fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(&mut self, message: &EventKind, _text_context: &mut TextContext, event: &mut Event) {
        scrollable::handle_scroll_logic(self, message, event);
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }
}

impl Container {
    pub fn new() -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<ContainerInner>>| {
            RefCell::new(ContainerInner {
                element_data: ElementData::new(me.clone(), true),
            })
        });
        inner.borrow_mut().element_data.create_layout_node(None);
        Self { inner }
    }
}
