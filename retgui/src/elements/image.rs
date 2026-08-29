//! Displays an image.

use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use retgui_resource_manager::{ResourceId, ResourceManager};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::resource_type::ResourceType;

use crate::app::{GUMMY_TREE, PENDING_RESOURCES};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::apply_generic_leaf_layout;
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, Element, ElementInternals};
use crate::layout::GummyTree;
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
    resource_id: ResourceId,
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
    type Inner = ImageInner;

    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementInternals>> {
        self.inner.clone()
    }

    fn borrow(&self) -> Ref<'_, dyn ElementInternals> {
        self.inner.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, dyn ElementInternals> {
        self.inner.borrow_mut()
    }

    fn with<R>(&self, callback: impl FnOnce(&Self::Inner) -> R) -> R {
        callback(&self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut Self::Inner) -> R) -> R {
        callback(&mut self.inner.borrow_mut())
    }
}

impl ElementInternals for ImageInner {
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
        apply_generic_leaf_layout(self, gummy_tree, z_index, scale_factor);
    }

    fn draw(
        &mut self,
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
    pub fn new(resource_id: ResourceId) -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<ImageInner>>| {
            RefCell::new(ImageInner {
                is_image_dirty: false,
                resource_id: resource_id.clone(),
                element_data: ElementData::new(me.clone(), false),
            })
        });
        let layout_context = Some(LayoutContext::Image(ImageContext::new(resource_id.clone())));
        inner.borrow_mut().element_data.create_layout_node(layout_context);

        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            pending_resources.push_back((resource_id, ResourceType::Image));
        });

        Self { inner }
    }

    pub fn dummy() -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<ImageInner>>| {
            RefCell::new(ImageInner {
                is_image_dirty: false,
                resource_id: ResourceId::DUMMY,
                element_data: ElementData::new(me.clone(), false),
            })
        });
        let layout_context = Some(LayoutContext::Image(ImageContext::new(ResourceId::DUMMY)));
        inner.borrow_mut().element_data.create_layout_node(layout_context);

        Self { inner }
    }

    pub fn resource_id(self, resource_id: ResourceId) -> Self {
        self.inner.borrow_mut().set_image(resource_id);
        self
    }

    pub fn get_resource_id(&self) -> ResourceId {
        self.inner.borrow().get_resource_id().clone()
    }
}

impl ImageInner {
    pub fn set_image(&mut self, resource_id: ResourceId) {
        self.is_image_dirty = true;
        self.resource_id = resource_id.clone();

        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            pending_resources.push_back((self.resource_id.clone(), ResourceType::Image));
        });

        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
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
