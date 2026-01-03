//! Displays an image.

use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use craft_primitives::geometry::Rectangle;
use craft_renderer::RenderList;
use craft_resource_manager::ResourceIdentifier;
use craft_resource_manager::resource_type::ResourceType;
use kurbo::{Affine, Point};

use crate::app::{ELEMENTS, PENDING_RESOURCES, TAFFY_TREE};
use crate::elements::Element;
use crate::elements::core::ElementInternals;
use crate::elements::element_data::ElementData;
use crate::layout::TaffyTree;
use crate::layout::layout_context::{ImageContext, LayoutContext};
use crate::text::text_context::TextContext;

/// Displays an image.
pub struct Image {
    is_image_dirty: bool,
    resource_identifier: ResourceIdentifier,
    element_data: ElementData,
}

impl Image {
    pub fn new(resource_identifier: ResourceIdentifier) -> Rc<RefCell<Self>> {
        let me = Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
            RefCell::new(Self {
                is_image_dirty: false,
                resource_identifier: resource_identifier.clone(),
                element_data: ElementData::new(me.clone(), false),
            })
        });

        let layout_context = Some(LayoutContext::Image(ImageContext::new(resource_identifier.clone())));
        me.borrow_mut().element_data.create_layout_node(layout_context);

        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            pending_resources.push_back((resource_identifier, ResourceType::Image));
        });

        ELEMENTS.with_borrow_mut(|elements| {
            elements.insert(me.borrow().deref());
        });

        me
    }

    pub fn image(&mut self, resource_identifier: ResourceIdentifier) -> &mut Self {
        self.is_image_dirty = true;
        self.resource_identifier = resource_identifier.clone();

        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let context = LayoutContext::Image(ImageContext::new(resource_identifier));
            let node = self
                .element_data
                .layout_item
                .taffy_node_id
                .expect("Failed to get Image node");
            taffy_tree.set_node_context(node, Some(context));
        });

        self
    }

    pub fn get_image(&self) -> &ResourceIdentifier {
        &self.resource_identifier
    }
}

impl crate::elements::core::ElementData for Image {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl Element for Image {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ElementInternals for Image {
    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        _pointer: Option<Point>,
        _text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        let layout = taffy_tree.layout(self.element_data.layout_item.taffy_node_id.unwrap());
        self.resolve_box(position, transform, layout, z_index);

        self.apply_borders(scale_factor);
        self.apply_clip(clip_bounds);
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _pointer: Option<Point>,
        scale_factor: f64,
    ) {
        if !self.is_visible() {
            return;
        }

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, scale_factor);

        let computed_box_transformed = self.computed_box_transformed();
        let content_rectangle = computed_box_transformed.content_rectangle();
        self.draw_borders(renderer, scale_factor);

        renderer.draw_image(content_rectangle.scale(scale_factor), self.resource_identifier.clone());
    }
}
