use issho::IsshoError;
use std::cell::RefCell;
use std::rc::Weak;
use std::sync::Arc;
use winit::window::Window;

use crate::elements::ElementInternals;

pub type RetGuiAccessTree = issho::AccessTree<Arc<Window>, Weak<RefCell<dyn ElementInternals>>>;

thread_local! {
    static ACCESS_TREE: RetGuiAccessTree = {
        let tree = RetGuiAccessTree::new();
        tree.set_framework_name("RetGui");
        tree.set_native_platform();
        tree.set_on_access_event(|tree, node_id, event| -> Result<(), IsshoError> {
            let element = {
                let Some(node) = tree.get_node(node_id) else {
                    return Err(IsshoError::MissingAccessNode(node_id));
                };
                let Some(context) = node.context() else {
                    return Err(IsshoError::MissingAccessNode(node_id));
                };
                context.upgrade().unwrap()
            };

            element.borrow_mut().on_access_event(event)
        });
        tree
    };
}

pub(crate) fn access_tree() -> RetGuiAccessTree {
    ACCESS_TREE.with(Clone::clone)
}

pub(crate) fn set_subtree_context(
    element: &mut dyn ElementInternals,
    tree: &RetGuiAccessTree,
    root: issho::AccessKey,
    scale_factor: f64,
) {
    {
        let data = element.element_data_mut();
        data.access_tree = tree.clone();
        data.access_root = Some(root);
        data.access_scale_factor = scale_factor;
    }

    for child in element.element_data().children.clone() {
        set_subtree_context(&mut *child.borrow_mut(), tree, root, scale_factor);
    }
}

pub(crate) fn reparent_subtree(
    element: &mut dyn ElementInternals,
    tree: &RetGuiAccessTree,
    parent: issho::AccessKey,
    root: issho::AccessKey,
    scale_factor: f64,
) {
    let key = {
        let data = element.element_data();
        assert!(
            data.access_tree.ptr_eq(tree),
            "elements from different accessibility trees cannot be reparented"
        );
        data.access_key.expect("element accessibility node was not created")
    };
    tree.append_child(parent, key);
    set_subtree_context(element, tree, root, scale_factor);
}

pub(crate) fn detach_subtree(element: &mut dyn ElementInternals) {
    let (tree, key) = {
        let data = element.element_data();
        (data.access_tree.clone(), data.access_key)
    };
    if let Some(key) = key {
        tree.detach_node(key);
        let scale_factor = element.element_data().access_scale_factor;
        set_subtree_context(element, &tree, key, scale_factor);
    }
}

#[cfg(test)]
mod tests {
    use retgui_primitives::geometry::{Point, Size};

    use crate::elements::{Container, ElementData as _, ElementInternals, Text, Window};

    #[test]
    fn text_name_changes_are_retained_immediately() {
        let text = Text::new("before");
        let detached_key = text.inner.borrow().element_data().access_key.unwrap();
        let detached_tree = text.inner.borrow().element_data().access_tree.clone();
        assert!(detached_tree.contains_node(detached_key));
        assert_eq!(detached_tree.get_node(detached_key).unwrap().name(), "before");

        let window = Window::new("Accessibility test");
        window.inner.borrow_mut().push(text.inner.clone());

        let node = text.inner.borrow().element_data().access_key.unwrap();
        assert_eq!(node, detached_key);
        let tree = window.inner.borrow().access_tree.clone();
        assert_eq!(tree.get_node(node).unwrap().name(), "before");

        text.inner.borrow_mut().set_text("after");

        assert_eq!(tree.get_node(node).unwrap().name(), "after");
    }

    #[test]
    fn reparenting_preserves_the_existing_accessibility_node() {
        let window = Window::new("Accessibility test");
        let text = Text::new("child");
        let child = text.inner.clone();
        window.inner.borrow_mut().push(child.clone());

        let node = child.borrow().element_data().access_key.unwrap();
        let tree = window.inner.borrow().access_tree.clone();
        assert!(tree.contains_node(node));

        let child = window.inner.borrow_mut().remove_child(child).unwrap();
        assert_eq!(child.borrow().element_data().access_key, Some(node));
        assert!(tree.contains_node(node));

        window.inner.borrow_mut().push(child);
        assert_eq!(text.inner.borrow().element_data().access_key, Some(node));
    }

    #[test]
    fn child_node_survives_its_detached_parent() {
        let parent = Container::new();
        let text = Text::new("child");
        parent.inner.borrow_mut().push(text.inner.clone());
        let key = text.inner.borrow().element_data().access_key.unwrap();
        let tree = text.inner.borrow().element_data().access_tree.clone();

        drop(parent);

        assert!(tree.contains_node(key));
        let new_parent = Container::new();
        new_parent.inner.borrow_mut().push(text.inner.clone());
        assert_eq!(text.inner.borrow().element_data().access_key, Some(key));
    }

    #[test]
    fn deep_clone_creates_a_distinct_retained_node() {
        let text = Text::new("clone me");
        let original_key = text.inner.borrow().element_data().access_key.unwrap();
        let clone = text.inner.borrow().deep_clone();
        let clone_key = clone.borrow().element_data().access_key.unwrap();
        let tree = text.inner.borrow().element_data().access_tree.clone();

        assert_ne!(clone_key, original_key);
        assert_eq!(tree.get_node(clone_key).unwrap().name(), "clone me");
    }

    #[test]
    fn semantic_changes_do_not_rebuild_layout_bounds() {
        let container = Container::new();
        let (tree, key) = {
            let mut inner = container.inner.borrow_mut();
            let data = inner.element_data_mut();
            data.layout.computed_box_transformed.position = Point::new(4.0, 6.0);
            data.layout.computed_box_transformed.size = Size::new(20.0, 30.0);
            data.set_accessibility_bounds_from_layout(2.0);
            (data.access_tree.clone(), data.access_key.unwrap())
        };

        let bounds = tree.get_node(key).unwrap().bounding_rect();
        assert_eq!(bounds, issho::AccessRect::new(8.0, 12.0, 40.0, 60.0));

        container
            .inner
            .borrow_mut()
            .element_data_mut()
            .set_accessibility_name("container");

        assert_eq!(tree.get_node(key).unwrap().bounding_rect(), bounds);
    }

    #[test]
    fn regaining_native_window_focus_restores_the_retained_focus() {
        let window = Window::new("Accessibility focus test");
        let root = window.inner.borrow().element_data().access_key.unwrap();
        let tree = window.inner.borrow().access_tree.clone();
        window.inner.borrow_mut().focus();

        window.on_focused(false);
        assert_eq!(tree.get_focus(root), None);

        window.on_focused(true);
        assert_eq!(tree.get_focus(root), Some(root));
    }
}
