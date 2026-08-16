use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use issho::IsshoError;

use winit::window::Window;

use crate::elements::ElementInternals;
use crate::events::EventDispatcher;
use crate::text::text_context::TextContext;

#[derive(Clone)]
pub struct RetGuiAccessTree {
    tree: issho::AccessTree<Arc<Window>, Weak<RefCell<dyn ElementInternals>>>,
    pub(crate) event_dispatcher: Rc<RefCell<EventDispatcher>>,
    pub(crate) text_context: Rc<RefCell<Option<TextContext>>>,
}

impl RetGuiAccessTree {
    pub(crate) fn new() -> Self {
        Self {
            tree: issho::AccessTree::new(),
            event_dispatcher: Rc::new(RefCell::new(EventDispatcher::new())),
            text_context: Rc::new(RefCell::new(None)),
        }
    }
}

impl Deref for RetGuiAccessTree {
    type Target = issho::AccessTree<Arc<Window>, Weak<RefCell<dyn ElementInternals>>>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

thread_local! {
    pub(crate) static ACCESS_TREE: RetGuiAccessTree = {
        let tree = RetGuiAccessTree::new();
        tree.set_framework_name("RetGui");
        tree.set_native_platform();
        let event_dispatcher = tree.event_dispatcher.clone();
        let text_context = tree.text_context.clone();
        tree.set_on_access_event(move |tree, node_id, event| -> Result<(), IsshoError> {
            let element: Rc<RefCell<dyn ElementInternals>> = {
                let Some(node) = tree.get_node(node_id) else {
                    return Err(IsshoError::MissingAccessNode(node_id));
                };
                let Some(context) = node.context() else {
                    return Err(IsshoError::MissingAccessNode(node_id));
                };
                context.upgrade().unwrap()
            };
            {
                let mut element = element.borrow_mut();
                crate::elements::scrollable::handle_accessibility_scroll_event(&mut *element, &event);
                element.on_access_event(event)?;
            }

            let mut event_dispatcher = event_dispatcher.borrow_mut();
            let mut text_context = text_context.borrow_mut();
            if let Some(text_context) = text_context.as_mut() {
                event_dispatcher.dispatch_queued_events(text_context);
            }
            Ok(())
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
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use issho::AccessEvent;
    use retgui_primitives::geometry::{Affine, Size};

    use crate::app::{dequeue_event, queue_event};
    use crate::elements::{Button, Container, Element as _, ElementData as _, ElementInternals, Text, Window};
    use crate::events::{Event, EventKind};
    use crate::text::text_context::TextContext;

    #[test]
    fn access_events_drain_every_queued_event() {
        while dequeue_event().is_some() {}

        let button = Button::new();
        let (tree, key) = {
            let button = button.inner.borrow();
            let data = button.element_data();
            (data.access_tree.clone(), data.access_key.unwrap())
        };
        *tree.text_context.borrow_mut() = Some(TextContext::default());

        let target: Rc<RefCell<dyn ElementInternals>> = button.inner.clone();
        let target = Rc::downgrade(&target);
        let click_count = Rc::new(Cell::new(0));
        button.inner.borrow_mut().on_click({
            let click_count = click_count.clone();
            Rc::new(move |_| {
                let next_count = click_count.get() + 1;
                click_count.set(next_count);
                if next_count == 1 {
                    let target = target.upgrade().expect("button should still be alive");
                    queue_event(Event::new(target), EventKind::Click());
                }
            })
        });

        assert!(tree.dispatch_access_event(key, AccessEvent::Invoke).is_ok());
        assert_eq!(click_count.get(), 2);
        assert!(dequeue_event().is_none());

        *tree.text_context.borrow_mut() = None;
    }

    #[test]
    fn accessibility_label_updates_the_retained_name() {
        let button = Button::new().accessibility_name("Save changes");
        let (tree, key) = {
            let button = button.inner.borrow();
            let data = button.element_data();
            (data.access_tree.clone(), data.access_key.unwrap())
        };

        assert_eq!(tree.get_node(key).unwrap().name(), "Save changes");

        let _button = button.accessibility_name("Submit");

        assert_eq!(tree.get_node(key).unwrap().name(), "Submit");
    }

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
            data.layout.computed_box.size = Size::new(20.0, 30.0);
            data.layout.update_render_state(Affine::translate((4.0, 6.0)), None);
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
