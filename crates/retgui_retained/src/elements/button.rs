//! Stores one or more elements.

use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementInternals};
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;
use retgui_primitives::geometry::{Affine, Point, Rectangle};
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
        self.deep_clone_internal()
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(
            self,
            gummy_tree,
            position,
            z_index,
            transform,
            text_context,
            clip_bounds,
            scale_factor,
        );
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

    fn on_event(&mut self, _message: &EventKind, _text_context: &mut TextContext, _event: &mut Event) {}

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }
}

impl Button {
    pub fn new() -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<ButtonInner>>| {
            RefCell::new(ButtonInner {
                element_data: ElementData::new(me.clone(), true),
            })
        });
        inner.borrow_mut().element_data.create_layout_node(None);
        Self { inner }
    }
}
