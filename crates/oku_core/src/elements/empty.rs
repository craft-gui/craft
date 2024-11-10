use crate::components::component::{ComponentId, GenericUserState};
use crate::elements::element::{CommonElementData, Element};
use crate::elements::layout_context::LayoutContext;
use crate::RendererBox;
use cosmic_text::FontSystem;
use std::any::Any;
use std::collections::HashMap;
use taffy::{NodeId, TaffyTree};
use crate::engine::events::OkuEvent;

#[derive(Clone, Default, Debug)]
pub struct Empty {
    pub common_element_data: CommonElementData,
}

impl Empty {
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
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
    ) {
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem, element_state: &mut HashMap<ComponentId, Box<GenericUserState>>) -> NodeId {
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        for child in self.common_element_data.children.iter_mut() {
            let child_node = child.compute_layout(taffy_tree, font_system, element_state);
            child_nodes.push(child_node);
        }

        let style: taffy::Style = self.common_element_data.style.into();

        taffy_tree.new_with_children(style, &vec![]).unwrap()
    }

    fn finalize_layout(
        &mut self,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _x: f32,
        _y: f32,
        _font_system: &mut FontSystem,
        _element_state: &mut HashMap<ComponentId, Box<GenericUserState>>,
    ) {
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(&self, event: OkuEvent, element_state: &mut HashMap<ComponentId, Box<GenericUserState>>, font_system: &mut FontSystem) {
        
    }
}
