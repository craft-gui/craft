use crate::elements::element::Element;
use crate::elements::{Dropdown, Overlay};
use crate::reactive::tree::ComponentTreeNode;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use smallvec::SmallVec;

#[derive(Clone)]
/// Links the ComponentTree with the ElementTree.
///
/// This is needed because the component tree does not have a reference to the element tree.
pub(crate) struct FiberNode<'a> {
    /// The component tree node. Both "Components" and "Elements" are components.
    pub(crate) component: &'a ComponentTreeNode,
    /// If the node is an element, this will be Some(element).
    pub(crate) element: Option<&'a dyn Element>,
    /// The children of this node. This is a vector of FiberNode.
    pub(crate) children: SmallVec<[Rc<RefCell<FiberNode<'a>>>; 4]>,
    pub(crate) parent: Option<Weak<RefCell<FiberNode<'a>>>>,
    pub(crate) overlay_order: u32,
}

pub fn new<'a>(root_component: &'a ComponentTreeNode, root_element: &'a dyn Element) -> Rc<RefCell<FiberNode<'a>>> {
    // Dummy lets us treat the real root like any other child.
    let dummy_root = Rc::new(RefCell::new(FiberNode {
        component: root_component,
        element: None,
        children: SmallVec::new(),
        parent: None,
        overlay_order: 0,
    }));

    // ┌──────────────────┬────────────────────────┬──────────────────┐
    // │ component_stack  │ parent_fiber_stack     │ element_stack    │
    // └──────────────────┴────────────────────────┴──────────────────┘
    let mut component_stack: Vec<&'a ComponentTreeNode> = vec![root_component];
    let mut parent_fiber_stack: Vec<Rc<RefCell<FiberNode>>> = vec![Rc::clone(&dummy_root)];
    let mut element_stack: Vec<&'a dyn Element> = vec![root_element];

    while let (Some(component), Some(parent_fiber)) = (component_stack.pop(), parent_fiber_stack.pop()) {
        let mut overlay_order = parent_fiber.borrow().overlay_order;
        // If the component *is* an element, pop the matching element and
        // push its children so the two stacks stay aligned.
        let element = if component.is_element {
            let element = element_stack.pop().expect("component / element stacks out of sync");
            if element.as_any().is::<Overlay>() || element.as_any().is::<Dropdown>() {
                overlay_order += 1;
            }
            for child_element in element.children().iter().rev() {
                element_stack.push(child_element.internal.as_ref());
            }
            Some(element)
        } else {
            None
        };

        // Build the real fiber node and attach it to its parent.
        let this_fiber = Rc::new(RefCell::new(FiberNode {
            component,
            element,
            children: SmallVec::new(),
            parent: Some(Rc::downgrade(&parent_fiber)),
            overlay_order,
        }));
        parent_fiber.borrow_mut().children.push(Rc::clone(&this_fiber));

        // Sanity-check: the ID stored in the component must really point
        // to the parent we just attached it to.
        if component.id != 0 {
            debug_assert_eq!(
                component.parent_id,
                Some(parent_fiber.borrow().component.id),
                "component {} expects parent {:?}, but actual parent is {}",
                component.id,
                component.parent_id,
                parent_fiber.borrow().component.id,
            );
        }

        // Queue the component’s children and remember *this* fiber as
        // their parent.
        for child in component.children.iter().rev() {
            component_stack.push(child);
            parent_fiber_stack.push(Rc::clone(&this_fiber));
        }
    }

    // The dummy now has exactly one child: the tree’s true root.
    let root = Rc::clone(&dummy_root.borrow().children.first().expect("component tree was empty"));
    root.borrow_mut().parent = None;

    root
}
