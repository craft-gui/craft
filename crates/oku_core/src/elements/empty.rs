use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::Element;
use crate::elements::layout_context::LayoutContext;
use crate::geometry::Point;
use crate::reactive::element_state_store::ElementStateStore;
use crate::RendererBox;
use parley::FontContext;
use std::any::Any;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Default, Debug)]
pub struct Empty {
    pub common_element_data: CommonElementData,
}

impl Empty {
    #[allow(dead_code)]
    pub fn new() -> Empty {
        Empty {
            common_element_data: Default::default(),
        }
    }
}

impl Element for Empty {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn name(&self) -> &'static str {
        "Empty"
    }

    fn draw(
        &mut self,
        _renderer: &mut RendererBox,
        _font_context: &mut FontContext,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &ElementStateStore,
        _pointer: Option<Point>,
    ) {
    }

    fn compute_layout(
        &mut self,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _font_context: &mut FontContext,
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
        _font_context: &mut FontContext,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
    ) {
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
