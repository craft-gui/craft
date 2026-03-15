use crate::elements::ElementInternals;
use std::cell::RefCell;
use std::rc::Rc;

pub trait DeepClone {
    fn deep_clone_internal(&self) -> Rc<RefCell<dyn ElementInternals>>;
}

impl<T> DeepClone for T where T: ElementInternals + Clone + 'static {
    fn deep_clone_internal(&self) -> Rc<RefCell<dyn ElementInternals>> {
        let mut new_element = Rc::new(RefCell::new(self.clone()));
        let new_element: Rc<RefCell<dyn ElementInternals>> = new_element;

        {
            let mut new_data_binding = new_element.borrow_mut();
            let new_data = new_data_binding.element_data_mut();
            new_data.me = Rc::downgrade(&new_element);
            new_data.parent = None;



            // TODO: Clone the layout nodes?
            // if let Some(node_id) = new_data.layout.taffy_node_id_mut() {
            //
            //     TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            //     });
            //
            //     new_data.layout.taffy_node_id_mut();
            //     request_apply_layout(node_id);
            //     request_layout(node_id);
            // }

            let mut new_children = Vec::new();
            for child in &new_data.children {
                let new_child = child.borrow().deep_clone();
                new_children.push(new_child);
            }
        }

        new_element
    }
}