//! Stores one or more elements.

use crate::elements::element_data::ElementData;
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use crate::elements::internal_helpers::add_generic_accesskit_data;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::DeepClone;
use crate::elements::{resolve_clip_for_scrollable, scrollable, AsElement, Element, ElementData as ElementDataTrait, ElementInternals};
use crate::events::{Event, EventKind};
use crate::layout::TaffyTree;
use crate::style::Overflow;
use crate::text::text_context::TextContext;
#[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
use accesskit::{Role, TreeUpdate};
use craft_primitives::geometry::{Affine, Point, Rectangle};
use craft_renderer::renderer::Renderer;
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;
use craft_resource_manager::ResourceManager;

#[derive(Clone)]
pub struct RadioGroup {
    pub inner: Rc<RefCell<RadioGroupInner>>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub struct RadioGroupInner {
    element_data: ElementData,
    label: String,
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self::new("Radio Group")
    }
}

impl Element for RadioGroup {}

impl Drop for RadioGroupInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for RadioGroup {
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

impl crate::elements::ElementData for RadioGroupInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for RadioGroupInner {
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

    fn draw(&mut self, renderer: &mut dyn Renderer, resource_manager: Arc<ResourceManager>, scale_factor: f64, text_context: &mut TextContext) {
        draw_generic_container(self, renderer, resource_manager, text_context, scale_factor);
    }

    #[cfg(all(feature = "accesskit", not(target_arch = "wasm32")))]
    fn compute_accessibility_tree(&mut self, tree: &mut TreeUpdate, parent_index: Option<usize>, scale_factor: f64) {
        let current_node_id = accesskit::NodeId(self.element_data().internal_id);

        let mut current_node = accesskit::Node::new(Role::RadioGroup);
        current_node.set_label(self.label.clone());

        add_generic_accesskit_data(
            &mut self.element_data,
            current_node,
            current_node_id,
            tree,
            parent_index,
            scale_factor,
        );
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
            self.element_data.layout.apply_clip(clip_bounds);
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

impl RadioGroup {
    pub fn new(label: &str) -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<RadioGroupInner>>| {
            RefCell::new(RadioGroupInner {
                element_data: ElementData::new(me.clone(), true),
                label: label.to_string(),
            })
        });
        inner.borrow_mut().element_data.create_layout_node(None);
        Self { inner }
    }
}
