use crate::old_elements::element::{Element};
use crate::events::{CraftMessage, Event};
use crate::generate_component_methods;
use craft_primitives::geometry::{Point, Rectangle, Size};
use crate::layout::layout_context::LayoutContext;
use craft_renderer::renderer::RenderList;
use crate::style::Style;
use crate::text::text_context::TextContext;
use std::any::Any;
use std::sync::Arc;
use kurbo::Affine;
use smallvec::SmallVec;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;
use smol_str::SmolStr;
use crate::old_elements::ElementBoxed;

/// An element for storing related elements.
#[derive(Clone, Default)]
pub struct Container {
    children: SmallVec<[ElementBoxed; 4]>,
}


impl Element for Container {
    fn children(&self) -> &SmallVec<[ElementBoxed; 4]> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut SmallVec<[ElementBoxed; 4]> {
        &mut self.children
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        pointer: Option<Point>,
        window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {

    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        scale_factor: f64,
    ) -> Option<NodeId> {
        None
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {

    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: &CraftMessage,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
        target: Option<&dyn Element>,
        _current_target: Option<&dyn Element>,
    ) {

    }

    fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
    }
}
