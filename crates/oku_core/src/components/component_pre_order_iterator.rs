use crate::reactive::tree::ComponentTreeNode;

pub struct ComponentTreePreOrderIterator<'a> {
    stack: Vec<&'a ComponentTreeNode>,
}

impl<'a> ComponentTreePreOrderIterator<'a> {
    fn new(root: &'a ComponentTreeNode) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for ComponentTreePreOrderIterator<'a> {
    type Item = &'a ComponentTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            for child in node.children.iter().rev() {
                self.stack.push(child);
            }
            Some(node)
        } else {
            None
        }
    }
}

impl ComponentTreeNode {
    pub fn pre_order_iter(&self) -> ComponentTreePreOrderIterator {
        ComponentTreePreOrderIterator::new(self)
    }
}
