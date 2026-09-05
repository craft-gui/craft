use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gummy::{Layout, NodeId, Size, Style};

use retgui_resource_manager::ResourceManager;

use crate::elements::{DynElement, ElementNodes, Elements};
use crate::layout::layout_context::{LayoutContext, measure_content};
use crate::text::text_context::TextContext;

type LayoutOwnerRegistration = (u64, DynElement);
pub(crate) type PendingLayoutOwner = (u64, DynElement, u32);

pub struct GummyTree {
    inner: gummy::GummyTree<LayoutContext>,
    seen_layouts: HashMap<NodeId, Layout>,
    /// True if at least one node is dirty.
    is_layout_dirty: bool,
    /// True if the layout should be re-applied.
    is_apply_layout_dirty: HashSet<NodeId>,
    layout_owners: HashMap<NodeId, LayoutOwnerRegistration>,
    layout_orders: HashMap<NodeId, u32>,
}

impl GummyTree {
    pub(crate) fn new() -> Self {
        Self {
            inner: gummy::GummyTree::<LayoutContext>::new(),
            seen_layouts: HashMap::new(),
            is_layout_dirty: true,
            is_apply_layout_dirty: HashSet::new(),
            layout_owners: HashMap::new(),
            layout_orders: HashMap::new(),
        }
    }

    pub fn clone_node(&mut self, src: NodeId) -> NodeId {
        let style = self.inner.style(src).unwrap();
        let context = self.inner.get_node_context(src);

        match context {
            None => self.inner.new_leaf(style.clone()).unwrap(),
            Some(ctx) => self.inner.new_leaf_with_context(style.clone(), ctx.clone()).unwrap(),
        }
    }

    pub fn new_leaf(&mut self, layout: Style) -> NodeId {
        self.inner.new_leaf(layout).unwrap()
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.inner.add_child(parent, child).unwrap();
        self.request_layout();
    }

    pub fn add_child_at_index(&mut self, parent: NodeId, child: NodeId, index: usize) {
        self.inner.insert_child_at_index(parent, index, child).unwrap();
        self.request_layout();
    }

    pub fn mark_dirty(&mut self, node: NodeId) {
        self.inner.mark_dirty(node).unwrap();
        self.request_layout();
        self.request_apply_layout(node);
    }

    pub fn mark_node_and_leaves_dirty(&mut self, node: NodeId) {
        self.inner.mark_dirty(node).ok();
        self.mark_leaves_dirty(node);
        self.request_layout();
    }

    fn mark_leaves_dirty(&mut self, parent: NodeId) {
        let children = self.inner.children(parent).unwrap_or_default();

        if children.is_empty() {
            self.inner.mark_dirty(parent).ok();
            self.request_apply_layout(parent);
        } else {
            for child in children {
                self.mark_leaves_dirty(child);
            }
        }
    }

    pub fn children(&self, parent: NodeId) -> Vec<NodeId> {
        self.inner.children(parent).unwrap()
    }

    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) {
        self.inner.set_children(parent, children).unwrap();
        self.request_layout();
    }

    pub fn compute_layout(
        &mut self,
        node_id: NodeId,
        available_space: Size<gummy::AvailableSpace>,
        elements: &mut Elements,
        text_context: &mut TextContext,
        resource_manager: Arc<ResourceManager>,
    ) {
        let (_, nodes) = elements.disjoint_borrow_layout_and_elements();
        self.compute_layout_with_nodes(node_id, available_space, nodes, text_context, resource_manager);
    }

    pub(crate) fn compute_layout_with_nodes(
        &mut self,
        node_id: NodeId,
        available_space: Size<gummy::AvailableSpace>,
        nodes: &mut ElementNodes,
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
                        nodes,
                        text_context,
                        resource_manager.clone(),
                        style,
                    )
                },
            )
            .unwrap();
        self.is_layout_dirty = false;
    }

    /// Remove a specific `node` and its ancestors from the tree and drop it
    pub fn remove_subtree(&mut self, node: NodeId) {
        // Can we avoid this allocation?
        let children = self.inner.children(node).unwrap();

        for child in children {
            self.remove_subtree(child);
        }
        self.remove_node(node);
        self.request_layout();
    }

    /// Removes the `node`.
    ///
    /// The `node` is not removed from the tree entirely, it is simply no longer attached to its previous parent.
    pub fn unparent_node(&mut self, node: NodeId) {
        if let Some(parent) = self.inner.parent(node) {
            self.inner.remove_child(parent, node).unwrap();
            self.request_layout();
        }
    }

    /// Remove a specific node from the tree and drop it
    pub fn remove_node(&mut self, node: NodeId) {
        let owner_id = self.layout_owners.get(&node).map(|(owner_id, _)| *owner_id);
        let owned_children = owner_id.map_or_else(Vec::new, |owner_id| {
            self.inner
                .children(node)
                .unwrap_or_default()
                .into_iter()
                .filter(|child| self.layout_owners.get(child).map(|(id, _)| *id) == Some(owner_id))
                .collect::<Vec<_>>()
        });
        for owned in owned_children {
            self.inner.remove(owned).unwrap();
            self.forget_node(owned);
        }

        self.inner.remove(node).unwrap();
        self.forget_node(node);
        self.request_layout();
    }

    fn forget_node(&mut self, node: NodeId) {
        self.seen_layouts.remove(&node);
        self.layout_owners.remove(&node);
        self.layout_orders.remove(&node);
        self.is_apply_layout_dirty.remove(&node);
    }

    #[inline]
    pub fn set_style(&mut self, node: NodeId, style: Style) {
        self.inner.set_style(node, style).unwrap();
        self.request_layout();
        self.request_apply_layout(node);
    }

    /// Creates and adds a new unattached leaf node to the tree, and returns the [`NodeId`] of the new node
    ///
    /// Creates and adds a new leaf node with a supplied context
    pub(crate) fn new_leaf_with_context(&mut self, style: Style, context: LayoutContext) -> NodeId {
        self.inner.new_leaf_with_context(style, context).unwrap()
    }

    /// Sets the context data associated with the node
    #[inline]
    pub(crate) fn set_node_context(&mut self, node: NodeId, measure: Option<LayoutContext>) {
        self.inner.set_node_context(node, measure).unwrap();
        self.request_layout();
        self.request_apply_layout(node);
    }

    /// Return this node layout relative to its parent
    #[inline]
    pub fn get_layout(&self, node: NodeId) -> &Layout {
        self.inner.layout(node).unwrap()
    }

    #[inline(always)]
    pub fn has_new_layout(&self, node_id: NodeId) -> bool {
        self.seen_layouts.get(&node_id) != Some(self.get_layout(node_id))
    }

    /// Marks the layout of this node as seen
    #[inline]
    pub fn mark_seen(&mut self, node: NodeId) {
        let layout = *self.get_layout(node);
        self.seen_layouts.insert(node, layout);
    }

    pub(crate) fn register_owner(&mut self, node: NodeId, owner_id: u64, owner: DynElement) {
        self.layout_owners.insert(node, (owner_id, owner));
    }

    /// Returns only owners whose own Gummy result changed or whose local
    /// presentation state explicitly requested an apply.
    ///
    /// A Gummy recompute requires comparing the resulting tree because Gummy
    /// does not expose its changed-node list. Apply-only invalidations use the
    /// exact pending-node set and never walk the retained subtree.
    pub(crate) fn take_layout_owners(&mut self, root: NodeId, layout_was_recomputed: bool) -> Vec<PendingLayoutOwner> {
        fn collect(
            tree: &mut GummyTree,
            node: NodeId,
            order: &mut u32,
            seen_owners: &mut HashSet<u64>,
            owners: &mut Vec<PendingLayoutOwner>,
        ) {
            let node_order = *order;
            *order += 1;
            tree.layout_orders.insert(node, node_order);
            let needs_apply = tree.has_new_layout(node) || tree.is_apply_layout_dirty.contains(&node);
            if needs_apply
                && let Some((owner_id, owner)) = tree.layout_owners.get(&node)
                && seen_owners.insert(*owner_id)
            {
                owners.push((*owner_id, *owner, node_order));
            }

            for child in tree.inner.children(node).unwrap_or_default() {
                collect(tree, child, order, seen_owners, owners);
            }
        }

        let mut owners = Vec::new();
        if layout_was_recomputed {
            let mut order = 0;
            collect(self, root, &mut order, &mut HashSet::new(), &mut owners);
        } else {
            let mut pending = self
                .is_apply_layout_dirty
                .iter()
                .copied()
                .filter(|node| self.root_of(*node) == root)
                .collect::<Vec<_>>();
            pending.sort_by_key(|node| self.layout_orders.get(node).copied().unwrap_or(u32::MAX));

            let mut seen_owners = HashSet::new();
            for node in pending {
                if let Some((owner_id, owner)) = self.layout_owners.get(&node)
                    && seen_owners.insert(*owner_id)
                {
                    owners.push((*owner_id, *owner, self.layout_orders.get(&node).copied().unwrap_or(0)));
                }
            }
        }

        let pending = std::mem::take(&mut self.is_apply_layout_dirty);
        self.is_apply_layout_dirty = pending.into_iter().filter(|node| self.root_of(*node) != root).collect();
        owners
    }

    #[inline(always)]
    pub fn request_layout(&mut self) {
        self.is_layout_dirty = true;
    }

    #[inline(always)]
    pub fn request_apply_layout(&mut self, node: NodeId) {
        self.is_apply_layout_dirty.insert(node);
    }

    /*#[inline(always)]
    pub fn is_layout_dirty(&self) -> bool {
        self.is_layout_dirty
    }*/

    #[inline(always)]
    pub fn is_layout_dirty(&self, root: NodeId) -> bool {
        self.inner.dirty(root).unwrap()
    }

    #[inline(always)]
    pub fn is_apply_layout_dirty(&self, root: &NodeId) -> bool {
        self.is_apply_layout_dirty
            .iter()
            .any(|node| self.root_of(*node) == *root)
    }

    pub fn apply_layout(&mut self, root: NodeId) {
        self.is_apply_layout_dirty.retain(|node| *node != root);
    }

    /// Get an node's root.
    pub fn root_of(&self, mut node: NodeId) -> NodeId {
        while let Some(parent) = self.inner.parent(node) {
            node = parent;
        }
        node
    }
}
