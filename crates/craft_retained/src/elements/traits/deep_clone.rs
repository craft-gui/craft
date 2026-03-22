use crate::app::{TAFFY_TREE, request_apply_layout, request_layout};
use crate::elements::ElementInternals;
use crate::elements::element_id::create_unique_element_id;
use std::cell::RefCell;
use std::rc::Rc;

pub trait DeepClone {
    fn deep_clone_internal(&self) -> Rc<RefCell<dyn ElementInternals>>;
}

impl<T> DeepClone for T
where
    T: ElementInternals + Clone + 'static,
{
    fn deep_clone_internal(&self) -> Rc<RefCell<dyn ElementInternals>> {
        let new_element = Rc::new(RefCell::new(self.clone()));
        let new_element: Rc<RefCell<dyn ElementInternals>> = new_element;

        {
            let mut new_data_binding = new_element.borrow_mut();
            let new_data = new_data_binding.element_data_mut();
            new_data.internal_id = create_unique_element_id();
            new_data.me = Rc::downgrade(&new_element);
            new_data.parent = None;

            // Clone the layout node
            let node_id = new_data.layout.taffy_node_id_mut();
            TAFFY_TREE.with_borrow_mut(|taffy_tree| *node_id = taffy_tree.clone_node(*node_id));
            request_apply_layout(*node_id);
            request_layout(*node_id);

            let node_id_copy = *node_id;
            let mut new_children = Vec::new();
            for child in &new_data.children {
                let new_child = child.borrow().deep_clone();
                new_children.push(new_child.clone());

                let new_child_copy = new_child.clone();
                TAFFY_TREE.with_borrow_mut(move |taffy_tree| {
                    taffy_tree.add_child(
                        node_id_copy,
                        new_child_copy.borrow().element_data().layout.taffy_node_id.unwrap(),
                    );
                });
            }
            new_data.children = new_children;
        }

        new_element
    }
}
