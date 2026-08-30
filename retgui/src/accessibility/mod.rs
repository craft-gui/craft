use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use issho::IsshoError;

use winit::window::Window;

use crate::elements::{DynElement, ElementNode, Elements};

#[derive(Clone)]
pub struct RetGuiAccessTree {
    tree: issho::AccessTree<Arc<dyn Window>, DynElement>,
    pending_events: Arc<Mutex<VecDeque<(DynElement, issho::AccessEvent)>>>,
}

impl RetGuiAccessTree {
    pub(crate) fn new() -> Self {
        let tree = issho::AccessTree::new();
        tree.set_framework_name("RetGui");
        tree.set_native_platform();
        let pending_events = Arc::new(Mutex::new(VecDeque::new()));
        let queue = pending_events.clone();
        tree.set_on_access_event(move |tree, node_id, event| -> Result<(), IsshoError> {
            let node = tree.get_node(node_id).ok_or(IsshoError::MissingAccessNode(node_id))?;
            let target = *node.context().ok_or(IsshoError::MissingAccessNode(node_id))?;
            queue
                .lock()
                .expect("accessibility event queue poisoned")
                .push_back((target, event));
            Ok(())
        });
        Self {
            tree,
            pending_events,
        }
    }

    pub(crate) fn pop_event(&self) -> Option<(DynElement, issho::AccessEvent)> {
        self.pending_events
            .lock()
            .expect("accessibility event queue poisoned")
            .pop_front()
    }
}

impl Deref for RetGuiAccessTree {
    type Target = issho::AccessTree<Arc<dyn Window>, DynElement>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

pub(crate) fn set_subtree_context(
    elements: &mut Elements,
    element: &mut dyn ElementNode,
    tree: &RetGuiAccessTree,
    root: issho::AccessKey,
    scale_factor: f64,
) {
    {
        let data = element.element_data_mut();
        data.access_tree = tree.clone();
        data.access_root = Some(root);
        data.access_scale_factor.set(scale_factor);
    }

    for child in element.element_data().children.clone() {
        elements.dispatch_mut(child, |child, elements| {
            set_subtree_context(elements, child, tree, root, scale_factor)
        });
    }
}

pub(crate) fn reparent_subtree(
    elements: &mut Elements,
    element: &mut dyn ElementNode,
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
    set_subtree_context(elements, element, tree, root, scale_factor);
}

pub(crate) fn detach_subtree(elements: &mut Elements, element: &mut dyn ElementNode) {
    let (tree, key) = {
        let data = element.element_data();
        (data.access_tree.clone(), data.access_key)
    };
    if let Some(key) = key {
        tree.detach_node(key);
        let scale_factor = element.element_data().access_scale_factor.get();
        set_subtree_context(elements, element, &tree, key, scale_factor);
    }
}

#[cfg(test)]
mod tests {
    use issho::AccessEvent;
    use retgui_primitives::geometry::{Affine, Size};

    use crate::elements::{Button, Container, Element as _, Elements, Text, Window};

    #[test]
    fn access_events_are_queued_with_slotmap_targets() {
        let mut elements = Elements::new();
        let button = Button::new(&mut elements);
        let data = elements.get(button.inner).element_data();
        let tree = data.access_tree.clone();
        let key = data.access_key.unwrap();
        assert!(tree.dispatch_access_event(key, AccessEvent::Invoke).is_ok());
        let (target, _event) = tree.pop_event().expect("access event should be retained");
        assert_eq!(target, button.inner);
        assert!(tree.pop_event().is_none());
    }

    #[test]
    fn accessibility_label_updates_the_retained_name() {
        let mut elements = Elements::new();
        let button = Button::new(&mut elements).accessibility_name(&mut elements, "Save changes");
        let (tree, key) = {
            let data = elements.get(button.inner).element_data();
            (data.access_tree.clone(), data.access_key.unwrap())
        };

        assert_eq!(tree.get_node(key).unwrap().name(), "Save changes");
        button.accessibility_name(&mut elements, "Submit");
        assert_eq!(tree.get_node(key).unwrap().name(), "Submit");
    }

    #[test]
    fn text_name_changes_are_retained_immediately() {
        let mut elements = Elements::new();
        let text = Text::new(&mut elements, "before");
        let data = elements.get(text.inner).element_data();
        let detached_key = data.access_key.unwrap();
        let detached_tree = data.access_tree.clone();
        assert!(detached_tree.contains_node(detached_key));
        assert_eq!(detached_tree.get_node(detached_key).unwrap().name(), "before");

        let window = Window::new(&mut elements, "Accessibility test").push(&mut elements, text);
        let node = elements.get(text.inner).element_data().access_key.unwrap();
        assert_eq!(node, detached_key);
        let tree = elements
            .get_as::<crate::elements::WindowNode>(window.inner)
            .access_tree
            .clone();
        assert_eq!(tree.get_node(node).unwrap().name(), "before");

        text.text(&mut elements, "after");
        assert_eq!(tree.get_node(node).unwrap().name(), "after");
    }

    #[test]
    fn reparenting_preserves_the_existing_accessibility_node() {
        let mut elements = Elements::new();
        let text = Text::new(&mut elements, "child");
        let window = Window::new(&mut elements, "Accessibility test").push(&mut elements, text);
        let node = elements.get(text.inner).element_data().access_key.unwrap();
        let tree = elements
            .get_as::<crate::elements::WindowNode>(window.inner)
            .access_tree
            .clone();
        assert!(tree.contains_node(node));

        let child = window.remove_child(&mut elements, text.inner).unwrap();
        assert_eq!(elements.get(child).element_data().access_key, Some(node));
        assert!(tree.contains_node(node));

        window.push(&mut elements, child);
        assert_eq!(elements.get(text.inner).element_data().access_key, Some(node));
    }

    #[test]
    fn child_node_survives_its_detached_parent() {
        let mut elements = Elements::new();
        let text = Text::new(&mut elements, "child");
        let parent = Container::new(&mut elements).push(&mut elements, text);
        let data = elements.get(text.inner).element_data();
        let key = data.access_key.unwrap();
        let tree = data.access_tree.clone();
        parent.remove_child(&mut elements, text.inner).unwrap();
        elements.remove(parent.inner);

        assert!(tree.contains_node(key));
        Container::new(&mut elements).push(&mut elements, text);
        assert_eq!(elements.get(text.inner).element_data().access_key, Some(key));
    }

    #[test]
    fn deep_clone_creates_a_distinct_retained_node() {
        let mut elements = Elements::new();
        let text = Text::new(&mut elements, "clone me");
        let original_key = elements.get(text.inner).element_data().access_key.unwrap();
        let clone = elements.dispatch_mut(text.inner, |text, elements| text.deep_clone(elements));
        let clone_key = elements.get(clone).element_data().access_key.unwrap();
        let tree = elements.get(text.inner).element_data().access_tree.clone();

        assert_ne!(clone_key, original_key);
        assert_eq!(tree.get_node(clone_key).unwrap().name(), "clone me");
    }

    #[test]
    fn semantic_changes_do_not_rebuild_layout_bounds() {
        let mut elements = Elements::new();
        let container = Container::new(&mut elements);
        let (tree, key) = {
            let data = elements.get_mut(container.inner).element_data_mut();
            data.layout.computed_box.size = Size::new(20.0, 30.0);
            data.layout.update_render_state(Affine::translate((4.0, 6.0)), None);
            data.set_accessibility_bounds_from_layout(2.0);
            (data.access_tree.clone(), data.access_key.unwrap())
        };

        let bounds = tree.get_node(key).unwrap().bounding_rect();
        assert_eq!(bounds, issho::AccessRect::new(8.0, 12.0, 40.0, 60.0));

        elements
            .get_mut(container.inner)
            .element_data_mut()
            .set_accessibility_name("container");

        assert_eq!(tree.get_node(key).unwrap().bounding_rect(), bounds);
    }

    #[test]
    fn regaining_native_window_focus_restores_the_retained_focus() {
        let mut elements = Elements::new();
        let window = Window::new(&mut elements, "Accessibility focus test");
        let root = elements.get(window.inner).element_data().access_key.unwrap();
        let tree = elements
            .get_as::<crate::elements::WindowNode>(window.inner)
            .access_tree
            .clone();
        elements.dispatch_mut(window.inner, |window, elements| window.focus(elements));

        window.on_focused(&elements, false);
        assert_eq!(tree.get_focus(root), None);

        window.on_focused(&elements, true);
        assert_eq!(tree.get_focus(root), Some(root));
    }
}
