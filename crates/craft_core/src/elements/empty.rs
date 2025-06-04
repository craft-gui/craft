use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::geometry::{Point, Rectangle};
use crate::layout::layout_context::LayoutContext;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::renderer::RenderList;
use crate::text::text_context::TextContext;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;

#[derive(Clone, Default)]
pub struct Empty {
    pub element_data: ElementData,
}

impl Empty {
    #[allow(dead_code)]
    pub fn new() -> Empty {
        Empty {
            element_data: Default::default(),
        }
    }
}

impl Element for Empty {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Empty"
    }

    fn draw(
        &mut self,
        _renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _window: Option<Arc<Window>>,
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
        _text_context: &mut TextContext,
        _clip_bounds: Option<Rectangle>,
    ) {
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
