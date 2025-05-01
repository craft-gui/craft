use crate::elements::element::Element;
use crate::reactive::tree::ComponentTreeNode;
use std::collections::VecDeque;
use crate::elements::Overlay;

#[derive(Clone)]
pub(crate) struct FiberNode<'a> {
    pub(crate) element: Option<&'a dyn Element>,
    pub(crate) component: Option<&'a ComponentTreeNode>,
}

pub(crate) struct FiberNodePreOrderIterator<'a> {
    component_stack: Vec<&'a ComponentTreeNode>,
    element_stack: Vec<&'a dyn Element>,
}

impl<'a> FiberNode<'a> {
    pub fn new(component: Option<&'a ComponentTreeNode>, element: Option<&'a dyn Element>) -> Self {
        Self { element, component }
    }
}

impl<'a> FiberNodePreOrderIterator<'a> {
    fn new(fiber: &'a FiberNode<'a>) -> Self {
        let mut element_stack = vec![];
        if let Some(element) = fiber.element {
            element_stack.push(element);
        }
        let mut component_stack = vec![];
        if let Some(component) = fiber.component {
            component_stack.push(component);
        }
        Self {
            element_stack,
            component_stack,
        }
    }
}

impl<'a> FiberNodeLevelOrderIterator<'a> {
    fn new(fiber: &'a FiberNode<'a>) -> Self {
        let mut element_stack = VecDeque::new();
        if let Some(element) = fiber.element {
            element_stack.push_back(element);
        }
        let mut component_stack = VecDeque::new();
        if let Some(component) = fiber.component {
            component_stack.push_back(component);
        }
        Self {
            element_stack,
            component_stack,
        }
    }
}

impl<'a> Iterator for FiberNodePreOrderIterator<'a> {
    type Item = FiberNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.component_stack.pop() {
            for child in node.children.iter().rev() {
                self.component_stack.push(child);
            }
            if !self.element_stack.is_empty() {
                let first_id = self.element_stack[0].component_id();

                if first_id == node.id {
                    if let Some(element) = self.element_stack.pop() {
                        for child in element.children().iter().rev() {
                            self.element_stack.push(*child);
                        }
                        Some(FiberNode::new(Some(node), Some(element)))
                    } else {
                        Some(FiberNode::new(Some(node), None))
                    }
                } else {
                    Some(FiberNode::new(Some(node), None))
                }
            } else {
                Some(FiberNode::new(Some(node), None))
            }
        } else {
            self.element_stack.pop().map(|element| FiberNode::new(None, Some(element)))
        }
    }
}

impl<'a> FiberNode<'a> {
    #[allow(dead_code)]
    pub fn pre_order_iter(&'a self) -> FiberNodePreOrderIterator<'a> {
        FiberNodePreOrderIterator::new(self)
    }

    pub fn level_order_iter(&'a self) -> FiberNodeLevelOrderIterator<'a> {
        FiberNodeLevelOrderIterator::new(self)
    }

    pub fn dfs_with_overlay_depth(&'a self) -> FiberNodeDFSIterator<'a> {
        let mut stack = Vec::new();
        stack.push((self.clone(), 0));
        FiberNodeDFSIterator { stack }
    }
}

pub(crate) struct FiberNodeLevelOrderIterator<'a> {
    component_stack: VecDeque<&'a ComponentTreeNode>,
    element_stack: VecDeque<&'a dyn Element>,
}

impl<'a> Iterator for FiberNodeLevelOrderIterator<'a> {
    type Item = FiberNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Process the component stack first
        if let Some(node) = self.component_stack.pop_front() {
            // Add child components to the stack
            for child in &node.children {
                self.component_stack.push_back(child);
            }

            // Check if the current element matches the component
            if let Some(&element) = self.element_stack.front() {
                if element.component_id() == node.id {
                    self.element_stack.pop_front();

                    // Add child elements to the stack
                    for child in element.children() {
                        self.element_stack.push_back(child);
                    }

                    // Return a FiberNode with both node and element
                    return Some(FiberNode::new(Some(node), Some(element)));
                }
            }

            // Return a FiberNode with just the component node
            return Some(FiberNode::new(Some(node), None));
        }

        // If the component stack is empty, process the element stack
        if let Some(element) = self.element_stack.pop_front() {
            // Add child elements to the stack
            for child in element.children() {
                self.element_stack.push_back(child);
            }

            // Return a FiberNode with just the element
            return Some(FiberNode::new(None, Some(element)));
        }

        // If both stacks are empty, traversal is complete
        None
    }
}

pub(crate) struct FiberNodeDFSIterator<'a> {
    stack: Vec<(FiberNode<'a>, usize)>,
}

impl<'a> Iterator for FiberNodeDFSIterator<'a> {
    type Item = (FiberNode<'a>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, depth) = self.stack.pop()?;
        let mut depth = depth;
        let mut children = Vec::new();
        
        if let Some(comp) = node.component {
            for child_comp in comp.children.iter().rev() {
                children.push(FiberNode::new(Some(child_comp), None));
            }
        }
        
        if let Some(el) = node.element {
            if el.as_any().is::<Overlay>() {
                depth += 1;
            }
            
            for child_el in el.children().iter().rev() {
                children.push(FiberNode::new(None, Some(*child_el)));
            }
        }
        
        for child in children {
            self.stack.push((child, depth));
        }

        Some((node, depth))
    }
}