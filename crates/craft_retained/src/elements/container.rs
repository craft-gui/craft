//! Stores one or more elements.

use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_primitives::geometry::{Affine, Point, Rectangle};
use craft_renderer::RenderList;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementInternals, resolve_clip_for_scrollable, scrollable};
use crate::events::{Event, EventKind};
use crate::layout::TaffyTree;
use crate::style::Overflow;
use crate::text::text_context::TextContext;

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
        self.deep_clone_internal()
    }

    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(
            self,
            taffy_tree,
            position,
            z_index,
            transform,
            text_context,
            clip_bounds,
            scale_factor,
        );
    }

    fn draw(&mut self, renderer: &mut RenderList, text_context: &mut TextContext, scale_factor: f64) {
        draw_generic_container(self, renderer, text_context, scale_factor);
    }

    fn on_event(
        &mut self,
        message: &EventKind,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        scrollable::handle_scroll_logic(self, message, event);
    }

    fn apply_clip(&mut self, clip_bounds: Option<Rectangle>) {
        let overflow = self.style().get_overflow();
        if overflow[0] == Overflow::Scroll || overflow[1] == Overflow::Scroll {
            resolve_clip_for_scrollable(self, clip_bounds);
        } else {
            self.element_data.layout.resolve_clip(clip_bounds);
        }
    }

    fn push(&mut self, child: Rc<RefCell<dyn ElementInternals>>) {
        push_child_to_element(self, child);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
