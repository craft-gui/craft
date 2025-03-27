use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::elements::element_data::ElementData;
use crate::elements::element::Element;
use crate::elements::layout_context::LayoutContext;
use crate::geometry::Point;
use crate::reactive::element_state_store::ElementStateStore;
use crate::resource_manager::ResourceIdentifier;
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::FontSystem;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;

#[derive(Clone, Debug)]
pub struct Font {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub element_data: ElementData,
}

impl Font {
    pub fn new(resource_identifier: ResourceIdentifier) -> Self {
        Self {
            resource_identifier,
            element_data: Default::default(),
        }
    }

    pub fn name() -> &'static str {
        "Font"
    }
}

impl Element for Font {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Font"
    }

    fn draw(
        &mut self,
        _renderer: &mut RendererBox,
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<dyn Window>>
    ) {
    }

    fn compute_layout(
        &mut self,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        _scale_factor: f64,
    ) -> Option<NodeId> {
        None
    }

    fn finalize_layout(
        &mut self,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _position: Point,
        _z_index: &mut u32,
        _transform: glam::Mat4,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _font_system: &mut FontSystem,
    ) {
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Font {
    generate_component_methods_no_children!();
}
