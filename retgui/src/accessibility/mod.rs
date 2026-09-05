use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use issho::IsshoError;

use winit::window::Window;

use crate::elements::{DynElement, ElementInternals, Elements};

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
    element: &mut dyn ElementInternals,
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
    set_subtree_context(elements, element, tree, root, scale_factor);
}

pub(crate) fn detach_subtree(elements: &mut Elements, element: &mut dyn ElementInternals) {
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
