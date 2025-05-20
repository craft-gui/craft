use crate::geometry::{ElementBox, Rectangle, Size};
use crate::layout::layout_context::LayoutContext;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Default)]
pub struct LayoutItem {
    /// The taffy node id after this element is laid out.
    /// This may be None if this is a non-visual element like Font.
    pub taffy_node_id: Option<NodeId>,

    pub content_size: Size<f32>,
    // The computed values after transforms are applied.
    pub computed_box_transformed: ElementBox,
    // The computed values without any transforms applied to them.
    pub computed_box: ElementBox,
    pub computed_scrollbar_size: Size<f32>,
    pub scrollbar_size: Size<f32>,
    pub computed_scroll_track: Rectangle,
    pub computed_scroll_thumb: Rectangle,
    pub(crate) max_scroll_y: f32,
    
    pub layout_order: u32,
    pub clip_bounds: Option<Rectangle>,
    
    //  ---
    pub child_nodes: Vec<NodeId>,
}

impl LayoutItem {
    pub fn new() {
        
    }
    
    pub fn push_child(&mut self, child: &Option<NodeId>) {
        if let Some(taffy_node_id) = child.as_ref() {
            self.child_nodes.push(*taffy_node_id);   
        }
    }
    pub fn build_tree(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, style: taffy::Style) -> Option<NodeId> {
        self.taffy_node_id = Some(taffy_tree.new_with_children(style, &self.child_nodes).unwrap());
        self.taffy_node_id.clone()
    }
    pub fn build_tree_with_context(&mut self,
                                   taffy_tree: &mut TaffyTree<LayoutContext>,
                                   style: taffy::Style,
                                   layout_context: LayoutContext) -> Option<NodeId> {
        self.taffy_node_id = Some(taffy_tree.new_leaf_with_context(style, layout_context).unwrap());
        self.taffy_node_id.clone()
    }
}