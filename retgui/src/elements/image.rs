//! Displays an image.

use std::collections::VecDeque;
use std::sync::Arc;

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::resource_type::ResourceType;
use retgui_resource_manager::{ResourceId, ResourceManager};

use crate::App;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::apply_generic_leaf_layout;
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementIds, ElementInternals, ElementStates, RetGuiAccessTree, RetainedElements};
use crate::layout::GummyTree;
use crate::layout::layout_context::{ImageContext, LayoutContext};
use crate::text::text_context::TextContext;

/// Displays an image.
#[derive(Clone, Copy)]
pub struct Image {
    pub(crate) inner: DynElement,
}

#[derive(Clone)]
pub(crate) struct ImageElement {
    is_image_dirty: bool,
    resource_id: ResourceId,
    element_data: ElementData,
}

impl crate::elements::HasElementData for ImageElement {
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

impl ElementInternals for ImageElement {
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        DynElement::new(clone_element::<Self, _>(
            self,
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            |_, _| None,
        ))
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
        _elements: &RetainedElements,
        _states: &ElementStates,
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
    pub fn new(app: &mut App, resource_id: ResourceId) -> Self {
        Self {
            inner: ImageElement::insert(
                &mut app.elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
                &mut app.pending_resources,
                resource_id,
            ),
        }
    }

    pub fn dummy(app: &mut App) -> Self {
        Self {
            inner: ImageElement::insert_unloaded(
                &mut app.elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
                ResourceId::DUMMY,
            ),
        }
    }

    pub fn set_resource_id(&self, app: &mut App, resource_id: ResourceId) {
        if let Some(image) = app.elements.try_get_as_mut::<ImageElement>(self.inner) {
            image.set_image(&mut app.gummy_tree, &mut app.pending_resources, resource_id);
        }
    }

    pub fn resource_id(&self, app: &App) -> ResourceId {
        app.try_get_as::<ImageElement>(self.inner)
            .map_or(ResourceId::DUMMY, |image| image.get_resource_id().clone())
    }
}

impl ImageElement {
    pub(crate) fn insert(
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
        resource_id: ResourceId,
    ) -> DynElement {
        let inner = Self::insert_unloaded(elements, gummy_tree, access_tree, by_internal_id, resource_id.clone());
        pending_resources.push_back((resource_id, ResourceType::Image));
        inner
    }

    fn insert_unloaded(
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        resource_id: ResourceId,
    ) -> DynElement {
        let inner = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
            Box::new(ImageElement {
                is_image_dirty: false,
                resource_id: resource_id.clone(),
                element_data: ElementData::new(me, false, access_tree),
            })
        });
        let element = elements.get_as_mut::<ImageElement>(inner);
        element
            .element_data
            .create_layout_node(gummy_tree, Some(LayoutContext::Image(ImageContext::new(resource_id))));
        inner
    }

    pub fn set_image(
        &mut self,
        gummy_tree: &mut GummyTree,
        pending_resources: &mut VecDeque<(ResourceId, ResourceType)>,
        resource_id: ResourceId,
    ) {
        self.is_image_dirty = true;
        self.resource_id = resource_id.clone();

        pending_resources.push_back((self.resource_id.clone(), ResourceType::Image));

        let context = LayoutContext::Image(ImageContext::new(resource_id));
        let node = self
            .element_data
            .layout
            .gummy_node_id
            .expect("Failed to get Image layout node");
        gummy_tree.set_node_context(node, Some(context));
        self.request_window_redraw();
    }

    pub fn get_resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}
