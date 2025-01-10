use crate::components::component::ComponentSpecification;
use crate::elements::element::{Element};
use crate::elements::layout_context::{ImageContext, LayoutContext};
use crate::renderer::color::Color;
use crate::resource_manager::ResourceIdentifier;
use crate::reactive::state_store::StateStore;
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit, Weight};
use crate::{generate_component_methods_no_children, RendererBox};
use crate::components::props::Props;
use cosmic_text::FontSystem;
use std::any::Any;
use taffy::{NodeId, TaffyTree};
use crate::elements::common_element_data::CommonElementData;
use crate::geometry::{Border, ElementRectangle, Margin, Padding, Point, Size};

#[derive(Clone, Debug)]
pub struct Font {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub common_element_data: CommonElementData,
}

impl Font {
    pub fn new(resource_identifier: ResourceIdentifier) -> Self {
        Self {
            resource_identifier,
            common_element_data: Default::default(),
        }
    }

    pub fn name() -> &'static str {
        "Font"
    }
}

impl Element for Font {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
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
        _element_state: &StateStore,
        _pointer: Option<Point>,
    ) {
    }

    fn compute_layout(
        &mut self,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _font_system: &mut FontSystem,
        _element_state: &mut StateStore,
        _scale_factor: f64,
    ) -> Option<NodeId> {
        None
    }

    fn finalize_layout(
        &mut self,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _x: f32,
        _y: f32,
        _z_index: &mut u32,
        _transform: glam::Mat4,
        _font_system: &mut FontSystem,
        _element_state: &mut StateStore,
        _pointer: Option<Point>,
    ) {
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Font {
    generate_component_methods_no_children!();
}
