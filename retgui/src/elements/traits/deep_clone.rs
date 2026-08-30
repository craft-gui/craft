use crate::app::{ELEMENTS, GUMMY_TREE, request_apply_layout, request_layout};
use crate::elements::ElementInternals;
use crate::elements::element_id::create_unique_element_id;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn clone_element<T, F>(source: &T, remap: F) -> Rc<RefCell<T>>
where
    T: ElementInternals + Clone + 'static,
    F: FnOnce(&Rc<RefCell<T>>, &mut crate::layout::GummyTree) -> Option<gummy::NodeId>,
{
    let new_typed_element = Rc::new(RefCell::new(source.clone()));
    let new_element: Rc<RefCell<dyn ElementInternals>> = new_typed_element.clone();

    let (access_tree, access_key, access_scale_factor, source_children, node_id) = {
        let mut new_data_binding = new_element.borrow_mut();
        let new_data = new_data_binding.element_data_mut();
        new_data.internal_id = create_unique_element_id();
        new_data.me = Rc::downgrade(&new_element);
        new_data.parent = None;
        if let Some(unfocused_outline_color) = new_data.unfocused_outline_color.take() {
            new_data.style.set_outline_color(unfocused_outline_color);
        }
        if let Some(unfocused_outline_width) = new_data.unfocused_outline_width.take() {
            new_data.style.set_outline_width(unfocused_outline_width);
        }
        let (access_tree, access_key) = {
            let tree = crate::accessibility::access_tree();
            let source_key = new_data.access_key.expect("source accessibility node was not created");
            let mut node = new_data
                .access_tree
                .get_node(source_key)
                .expect("source accessibility node was not created")
                .clone();
            node.set_context(new_data.me.clone());
            let key = tree.insert_node(node, None);
            new_data.access_tree = tree.clone();
            new_data.access_key = Some(key);
            new_data.access_root = Some(key);
            (tree, key)
        };

        // Clone the layout node
        let node_id = new_data.layout.gummy_node_id_mut();
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            *node_id = gummy_tree.clone_node(*node_id);
            gummy_tree.register_owner(*node_id, new_data.internal_id, new_data.me.clone());
        });
        request_apply_layout(*node_id);
        request_layout(*node_id);

        (
            access_tree,
            access_key,
            new_data.access_scale_factor,
            new_data.children.clone(),
            *node_id,
        )
    };

    let (new_id, new_me) = {
        let binding = new_element.borrow();
        let data = binding.element_data();
        (data.internal_id, data.me.clone())
    };
    ELEMENTS.with_borrow_mut(|elements| {
        elements.insert_id(new_id, new_me);
    });

    let child_layout_parent = GUMMY_TREE
        .with_borrow_mut(|gummy_tree| remap(&new_typed_element, gummy_tree))
        .unwrap_or(node_id);

    let mut new_children = Vec::with_capacity(source_children.len());
    for child in source_children {
        let new_child = child.borrow().deep_clone();
        new_child.inner.borrow_mut().element_data_mut().parent = Some(Rc::downgrade(&new_element));
        crate::accessibility::reparent_subtree(
            &mut *new_child.inner.borrow_mut(),
            &access_tree,
            access_key,
            access_key,
            access_scale_factor,
        );
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            gummy_tree.add_child(
                child_layout_parent,
                new_child.inner.borrow().element_data().layout.gummy_node_id.unwrap(),
            );
        });
        new_children.push(new_child);
    }
    new_element.borrow_mut().element_data_mut().children = new_children;

    // Keep the primary node scheduled even if an element hook added owned
    // nodes or rebuilt retained state after it was initially cloned.
    request_apply_layout(node_id);

    new_typed_element
}
