//! Displays an image.

use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_primitives::geometry::Rectangle;

use craft_renderer::RenderList;

use craft_resource_manager::ResourceIdentifier;
use craft_resource_manager::resource_type::ResourceType;

use craft_primitives::geometry::{Affine, Point};

use crate::app::{PENDING_RESOURCES, TAFFY_TREE};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::apply_generic_leaf_layout;
use crate::elements::traits::DeepClone;
use crate::elements::{AsElement, Element, ElementInternals};
use crate::layout::TaffyTree;
use crate::layout::layout_context::{ImageContext, LayoutContext};
use crate::text::text_context::TextContext;

/// Displays an image.
#[derive(Clone)]
pub struct Image {
    pub inner: Rc<RefCell<ImageInner>>,
}

#[derive(Clone)]
pub struct ImageInner {
    is_image_dirty: bool,
    resource_identifier: ResourceIdentifier,
    element_data: ElementData,
}

impl crate::elements::ElementData for ImageInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl Element for Image {}

impl Drop for ImageInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Image {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }
}

impl ElementInternals for ImageInner {
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

        _renderer.draw_image(content_rectangle.scale(_scale_factor), self.resource_identifier.clone());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Image {
    pub fn new(resource_identifier: ResourceIdentifier) -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<ImageInner>>| {
            RefCell::new(ImageInner {
                is_image_dirty: false,
                resource_identifier: resource_identifier.clone(),
                element_data: ElementData::new(me.clone(), false),
            })
        });
        let layout_context = Some(LayoutContext::Image(ImageContext::new(resource_identifier.clone())));
        inner.borrow_mut().element_data.create_layout_node(layout_context);

        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            pending_resources.push_back((resource_identifier, ResourceType::Image));
        });

        Self { inner }
    }

    pub fn image(self, resource_identifier: ResourceIdentifier) -> Self {
        self.inner.borrow_mut().image(resource_identifier);
        self
    }

    pub fn get_image(&self) -> ResourceIdentifier {
        self.inner.borrow().get_image().clone()
    }
}

impl ImageInner {
    pub fn image(&mut self, resource_identifier: ResourceIdentifier) {
        self.is_image_dirty = true;
        self.resource_identifier = resource_identifier.clone();

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let context = LayoutContext::Image(ImageContext::new(resource_identifier));
            let node = self
                .element_data
                .layout
                .taffy_node_id
                .expect("Failed to get Image node");
            taffy_tree.set_node_context(node, Some(context));
        });
    }

    pub fn get_image(&self) -> &ResourceIdentifier {
        &self.resource_identifier
    }
}
