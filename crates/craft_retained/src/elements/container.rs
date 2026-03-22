//! Stores one or more elements.

use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_primitives::geometry::Rectangle;
use craft_renderer::RenderList;

use kurbo::{Affine, Point};

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::push_child_to_element;
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementInternals, resolve_clip_for_scrollable, scrollable};
use crate::events::{Event, EventKind};
use crate::layout::TaffyTree;
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
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout.taffy_node_id.unwrap();
        let layout = taffy_tree.get_layout(node);
        let has_new_layout = taffy_tree.has_new_layout(node);

        let dirty = has_new_layout
            || transform != self.element_data.layout.get_transform()
            || position != self.element_data.layout.position;
        self.element_data.layout.has_new_layout = has_new_layout;

        if dirty {
            self.resolve_box(position, transform, layout, z_index);
            self.apply_borders(scale_factor);
            // For scroll changes from taffy;
            self.element_data.apply_scroll(layout);
            self.apply_clip(clip_bounds);
            self.element_data.layout.scroll_state.mark_old();
        }

        // For manual scroll updates.
        if !dirty && self.element_data.layout.scroll_state.is_new() {
            self.element_data.apply_scroll(layout);
            self.element_data.layout.scroll_state.mark_old();
        }

        if has_new_layout {
            taffy_tree.mark_seen(node);
        }

        let scroll_y = self.element_data.scroll().scroll_y() as f64;
        let child_transform = Affine::translate((0.0, -scroll_y));

        self.apply_layout_children(
            taffy_tree,
            z_index,
            transform * child_transform,
            pointer,
            text_context,
            scale_factor,
            false,
        )
    }

    fn draw(&mut self, renderer: &mut RenderList, text_context: &mut TextContext, scale_factor: f64) {
        if !self.is_visible() {
            return;
        }
        self.add_hit_testable(renderer, true, scale_factor);
        self.draw_borders(renderer, scale_factor);
        self.maybe_start_layer(renderer, scale_factor);
        self.draw_children(renderer, text_context, scale_factor);
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer, scale_factor);
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
        resolve_clip_for_scrollable(self, clip_bounds);
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
