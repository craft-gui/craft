use crate::layout::layout_context::{measure_content, LayoutContext};
use crate::text::text_context::TextContext;
use craft_resource_manager::ResourceManager;
use std::sync::Arc;
use taffy::{Layout, NodeId, PrintTree, Size, Style};

pub struct TaffyTree {
    inner: taffy::TaffyTree<LayoutContext>,
    /// True if at least one node is dirty.
    dirty: bool,
}

impl TaffyTree {
    pub(crate) fn new() -> Self {
        Self {
            inner: taffy::TaffyTree::<LayoutContext>::new(),
            dirty: true,
        }
    }

    pub fn new_leaf(&mut self, layout: Style) -> NodeId {
        self.inner.new_leaf(layout).unwrap()
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.inner.add_child(parent, child).unwrap();
    }

    pub fn mark_dirty(&mut self, node: NodeId) {
        self.inner.mark_dirty(node).unwrap();
    }

    pub fn children(&self, parent: NodeId) -> Vec<NodeId> {
        self.inner.children(parent).unwrap()
    }

    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) {
        self.inner.set_children(parent, children).unwrap();
    }

    pub fn compute_layout(
        &mut self,
        node_id: NodeId,
        available_space: Size<taffy::AvailableSpace>,
        text_context: &mut TextContext,
        resource_manager: Arc<ResourceManager>,
    ) {
        self.inner
            .compute_layout_with_measure(
                node_id,
                available_space,
                |known_dimensions, available_space, _node_id, node_context, style| {
                    measure_content(
                        known_dimensions,
                        available_space,
                        node_context,
                        text_context,
                        resource_manager.clone(),
                        style,
                    )
                },
            )
            .unwrap();
    }

    // Remove the entire layout subtree.

    pub fn remove_subtree(&mut self, node: NodeId) {
        // Can we avoid this allocation?
        let children = self.inner.children(node).unwrap();

        for child in children {
            self.remove_subtree(child);
        }

        self.inner.remove(node).map(|_| ()).unwrap();
    }

    #[inline]
    pub fn set_style(&mut self, node: NodeId, style: Style) {
        self.inner.set_style(node, style).unwrap();
    }

    /// Creates and adds a new unattached leaf node to the tree, and returns the [`NodeId`] of the new node
    ///
    /// Creates and adds a new leaf node with a supplied context
    pub fn new_leaf_with_context(&mut self, style: Style, context: LayoutContext) -> NodeId {
        self.inner.new_leaf_with_context(style, context).unwrap()
    }

    /// Sets the context data associated with the node
    #[inline]
    pub fn set_node_context(&mut self, node: NodeId, measure: Option<LayoutContext>) {
        self.inner.set_node_context(node, measure).unwrap();
    }

    /// Return this node layout relative to its parent
    #[inline]
    pub fn layout(&self, node: NodeId) -> &Layout {
        self.inner.layout(node).unwrap()
    }

    #[inline(always)]
    pub fn get_has_new_layout(&self, node_id: NodeId) -> bool {
        self.inner.get_has_new_layout(node_id)
    }

    /// Marks the layout of this node as seen
    #[inline]
    pub fn mark_seen(&mut self, node: NodeId) {
        self.inner.mark_seen(node);
    }
}
