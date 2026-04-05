//! Displays an TinyVg.

use craft_primitives::geometry::Rectangle;
use peniko::Color;
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_renderer::RenderList;

use craft_resource_manager::ResourceId;
use craft_resource_manager::resource_type::ResourceType;

use craft_primitives::geometry::{Affine, Point};

use crate::app::{PENDING_RESOURCES, TAFFY_TREE};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::apply_generic_leaf_layout;
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementInternals};
use crate::layout::TaffyTree;
use crate::layout::layout_context::{LayoutContext, TinyVgContext};
use crate::rgba;
use crate::text::text_context::TextContext;

/// Displays an TinyVg.
#[derive(Clone)]
pub struct TinyVg {
    pub inner: Rc<RefCell<TinyVgInner>>,
}

#[derive(Clone)]
pub struct TinyVgInner {
    is_tiny_vg_dirty: bool,
    resource_id: ResourceId,
    element_data: ElementData,
}

impl crate::elements::ElementData for TinyVgInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl Element for TinyVg {}

impl Drop for TinyVgInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for TinyVg {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }
}

impl ElementInternals for TinyVgInner {
    fn deep_clone(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.deep_clone_internal()
    }

    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        _text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        apply_generic_leaf_layout(
            self,
            taffy_tree,
            position,
            z_index,
            transform,
            clip_bounds,
            scale_factor,
        );
    }

    fn draw(&mut self, _renderer: &mut RenderList, _text_context: &mut TextContext, _scale_factor: f64) {
        if !self.is_visible() {
            return;
        }

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(_renderer, _scale_factor);

        let computed_box_transformed = self.get_computed_box_transformed();
        let content_rectangle = computed_box_transformed.content_rectangle();
        self.draw_borders(_renderer, _scale_factor);

        let mut color = None;
        if self.style().get_color() != rgba(0, 0, 0, 0) {
            color = Some(self.style().get_color());
        }
        _renderer.draw_tiny_vg(content_rectangle.scale(_scale_factor), self.resource_id.clone(), color);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl TinyVg {
    pub fn new(resource_id: ResourceId) -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<TinyVgInner>>| {
            RefCell::new(TinyVgInner {
                is_tiny_vg_dirty: false,
                resource_id: resource_id.clone(),
                element_data: ElementData::new(me.clone(), false),
            })
        });
        let layout_context = Some(LayoutContext::TinyVg(TinyVgContext::new(resource_id.clone())));
        inner.borrow_mut().element_data.create_layout_node(layout_context);
        inner.borrow_mut().style_mut().set_color(Color::TRANSPARENT);

        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            pending_resources.push_back((resource_id, ResourceType::TinyVg));
        });

        Self { inner }
    }

    pub fn set_resource_id(self, resource_id: ResourceId) -> Self {
        self.inner.borrow_mut().set_resource_id(resource_id);
        self
    }

    pub fn get_resource_id(&self) -> ResourceId {
        self.inner.borrow().get_resource_id().clone()
    }
}

impl TinyVgInner {
    pub fn set_resource_id(&mut self, resource_id: ResourceId) {
        self.is_tiny_vg_dirty = true;
        self.resource_id = resource_id.clone();

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let context = LayoutContext::TinyVg(TinyVgContext::new(resource_id));
            let node = self
                .element_data
                .layout
                .taffy_node_id
                .expect("Failed to get TinyVg node");
            taffy_tree.set_node_context(node, Some(context));
        });
    }

    pub fn get_resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}
