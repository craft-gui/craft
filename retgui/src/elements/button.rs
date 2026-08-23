//! Stores one or more elements.

use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use issho::{AccessEvent, IsshoError, Role};

use crate::app::queue_event;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, Element, ElementInternals};
use crate::events::{ClickEvent, ClickTrigger, EventKind};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;
use retgui_renderer::renderer::Renderer;
use retgui_resource_manager::ResourceManager;

#[derive(Clone)]
pub struct Button {
    pub inner: Rc<RefCell<ButtonInner>>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub struct ButtonInner {
    element_data: ElementData,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ButtonState {
    Default,
    Hovered,
    Pressed,
    Focused,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Button {}

impl Drop for ButtonInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Button {
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

impl crate::elements::ElementData for ButtonInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for ButtonInner {
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

    fn on_event(&mut self, event: &mut EventKind, _text_context: &mut TextContext) {
        if let EventKind::Click(_) = event {
            self.focus();
        }
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }

    fn on_access_event(&mut self, event: AccessEvent) -> Result<(), IsshoError> {
        if let AccessEvent::Invoke = event {
            let target = self
                .element_data
                .me
                .upgrade()
                .expect("button was detached while handling its invoke action");
            queue_event(EventKind::Click(ClickEvent::new(target, ClickTrigger::Accessibility)));
        }
        Ok(())
    }
}

impl Button {
    pub fn new() -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<ButtonInner>>| {
            RefCell::new(ButtonInner {
                element_data: ElementData::new(me.clone(), true),
            })
        });
        let mut inner_mut = inner.borrow_mut();
        inner_mut.element_data.create_layout_node(None);
        inner_mut.element_data.set_accessibility_role(Role::Button);
        drop(inner_mut);
        Self { inner }
    }
}
