/*use crate::reactive::tree::ElementTreeNode;

pub struct ElementTreePreOrderIterator<'a> {
    stack: Vec<&'a ElementTreeNode>,
}

impl<'a> ElementTreePreOrderIterator<'a> {
    fn new(root: &'a ElementTreeNode) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for ElementTreePreOrderIterator<'a> {
    type Item = &'a ElementTreeNode;

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

impl ElementTreeNode {
    pub fn pre_order_iter(&self) -> ElementTreePreOrderIterator {
        ElementTreePreOrderIterator::new(self)
    }
}
*/