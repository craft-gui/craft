/// Stores elements in a spatially indexed tree.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Weak;
use kurbo::{Point, Rect, RoundedRect};
use understory_box_tree::{LocalNode, QueryFilter, Tree};
use understory_index::backends::FlatVec;
use crate::elements::Element;

#[derive(Default)]
pub struct SpatialTree {
    tree: Tree<FlatVec<f64>>,
    map: HashMap<understory_box_tree::NodeId, Weak<RefCell<dyn Element>>>,
    cache: HashMap<understory_box_tree::NodeId, Rect>,
    updates: usize,
}

impl SpatialTree {

    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Insert a node into the tree.
    pub fn insert(&mut self, element: &mut dyn Element) {
        let spatial_node = self.tree.insert(None, LocalNode::default());
        element.element_data_mut().layout_item.spatial_node_id = Some(spatial_node);
        self.map.insert(spatial_node, element.element_data().me.clone().unwrap());
    }

    pub fn get(&mut self, element: &dyn Element) -> Option<Weak<RefCell<dyn Element>>> {
        self.map.get(&element.element_data().layout_item.spatial_node_id.unwrap()).cloned()
    }

    /// Pushes a child element.
    ///
    /// If the child has no spatial node, no changes will be made.
    ///
    /// Panics:
    /// If the parent has no spatial node, this function wil panic.
    pub fn push_child(&mut self, parent: &dyn Element, child: &dyn Element) {
        let parent_spatial_id = parent.element_data().layout_item.spatial_node_id.unwrap();
        let child_spatial_id = child.element_data().layout_item.spatial_node_id;
        if let Some(child_spatial_id) = child_spatial_id {
            self.updates += 1;
            self.tree.reparent(child_spatial_id, Some(parent_spatial_id));
        }
    }

    pub fn update_bounds(&mut self, element: &dyn Element) {
        let spatial_id = element.element_data().layout_item.spatial_node_id.unwrap();
        let bounds = element.element_data().layout_item.computed_box_transformed.padding_rectangle().to_kurbo();
        if let Some(cache_bounds) = self.cache.get(&spatial_id) {
            if *cache_bounds != bounds {
                self.tree.set_local_bounds(spatial_id, bounds);
                self.cache.insert(spatial_id, bounds);
                self.updates += 1;
            }
        } else {
            self.tree.set_local_bounds(spatial_id, bounds);
            self.cache.insert(spatial_id, bounds);
            self.updates += 1;
        }
    }

    /// Removes a node.
    ///
    /// Panics:
    /// If the element has no spatial node.
    pub fn remove(&mut self, element: &dyn Element) {
        let spatial_id = element.element_data().layout_item.spatial_node_id.unwrap();
        self.tree.remove(spatial_id);
        self.map.remove(&spatial_id);
        self.updates += 1;
    }

    /// Hit tests a point and returns the top-most element.
    pub fn hit_test_point(&self, point: Point) -> Option<Weak<RefCell<dyn Element>>> {
        let node = self.tree.hit_test_point(point, QueryFilter::new())?.node;
        self.map.get(&node).cloned()
    }

    /// Ensure all changes are reflected in the tree.
    pub fn commit(&mut self) {
        if self.updates == 0 {
            return;
        }
        self.updates = 0;
        self.tree.commit();
    }

}