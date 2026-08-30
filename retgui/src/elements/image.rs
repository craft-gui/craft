//! Displays an image.

use std::sync::Arc;

use retgui_resource_manager::{ResourceId, ResourceManager};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::resource_type::ResourceType;

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::apply_generic_leaf_layout;
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementNode, Elements};
use crate::layout::GummyTree;
use crate::layout::layout_context::{ImageContext, LayoutContext};
use crate::text::text_context::TextContext;

/// Displays an image.
#[derive(Clone, Copy)]
pub struct Image {
    pub(crate) inner: DynElement,
}

#[derive(Clone)]
pub(crate) struct ImageNode {
    is_image_dirty: bool,
    resource_id: ResourceId,
    element_data: ElementData,
}

impl crate::elements::ElementNodeData for ImageNode {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl Element for Image {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl ElementNode for ImageNode {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, elements, |_, _| None))
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_leaf_layout(self, gummy_tree, z_index, scale_factor);
    }

    fn draw(
        &self,
        _elements: &Elements,
        _renderer: &mut dyn Renderer,
        _resource_manager: Arc<ResourceManager>,
        _scale_factor: f64,
        _text_context: &mut TextContext,
    ) {
        if !self.is_visible() {
            return;
        }

        self.maybe_start_overlay(_renderer);

        self.add_hit_testable(_renderer, true, _scale_factor);

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(_renderer, _scale_factor);

        let content_rectangle = self.element_data.layout.local_box().content_rectangle();

        _renderer.draw_image(content_rectangle.scale(_scale_factor), self.resource_id.clone());

        self.maybe_end_overlay(_renderer);
    }
}

impl Image {
    pub fn new(elements: &mut Elements, resource_id: ResourceId) -> Self {
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(ImageNode {
                is_image_dirty: false,
                resource_id: resource_id.clone(),
                element_data: ElementData::new(me, false, access_tree),
            })
        });
        let layout_context = Some(LayoutContext::Image(ImageContext::new(resource_id.clone())));
        elements.create_layout_node(inner, layout_context);

        elements.pending_resources.push_back((resource_id, ResourceType::Image));

        Self { inner }
    }

    pub fn dummy(elements: &mut Elements) -> Self {
        let inner = elements.insert_with(|me, access_tree| {
            Box::new(ImageNode {
                is_image_dirty: false,
                resource_id: ResourceId::DUMMY,
                element_data: ElementData::new(me, false, access_tree),
            })
        });
        let layout_context = Some(LayoutContext::Image(ImageContext::new(ResourceId::DUMMY)));
        elements.create_layout_node(inner, layout_context);

        Self { inner }
    }

    pub fn resource_id(self, elements: &mut Elements, resource_id: ResourceId) -> Self {
        elements.try_dispatch_mut(self.inner, |image, elements| {
            (image as &mut dyn std::any::Any)
                .downcast_mut::<ImageNode>()
                .unwrap()
                .set_image(elements, resource_id)
        });
        self
    }

    pub fn get_resource_id(&self, elements: &Elements) -> ResourceId {
        elements
            .try_get_as::<ImageNode>(self.inner)
            .map_or(ResourceId::DUMMY, |image| image.get_resource_id().clone())
    }
}

impl ImageNode {
    pub fn set_image(&mut self, elements: &mut Elements, resource_id: ResourceId) {
        self.is_image_dirty = true;
        self.resource_id = resource_id.clone();

        elements
            .pending_resources
            .push_back((self.resource_id.clone(), ResourceType::Image));

        elements.with_gummy_tree(|gummy_tree, _| {
            let context = LayoutContext::Image(ImageContext::new(resource_id));
            let node = self
                .element_data
                .layout
                .gummy_node_id
                .expect("Failed to get Image node");
            gummy_tree.set_node_context(node, Some(context));
        });
        self.request_window_redraw();
    }

    pub fn get_resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}
