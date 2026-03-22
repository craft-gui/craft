use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::app::TAFFY_TREE;
use crate::elements::ElementInternals;

/// A helper to push children.
pub fn push_child_to_element(parent: &mut dyn ElementInternals, child: Rc<RefCell<dyn ElementInternals>>) {
    let element_data = parent.element_data_mut();
    let me: Weak<RefCell<dyn ElementInternals>> = element_data.me.clone();
    let me_window = element_data.window.clone();
    child.borrow_mut().element_data_mut().parent = Some(me);
    child.borrow_mut().element_data_mut().window = me_window;
    child.borrow_mut().propagate_window_down();
    element_data.children.push(child.clone());

    // Add the children's taffy node.
    TAFFY_TREE.with_borrow_mut(|taffy_tree| {
        let parent_id = element_data.layout.taffy_node_id.unwrap();
        let child_id = child.borrow().element_data().layout.taffy_node_id;
        if let Some(child_id) = child_id {
            taffy_tree.add_child(parent_id, child_id);
        }
    });
}
