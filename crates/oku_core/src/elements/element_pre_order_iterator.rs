use crate::elements::element::Element;
pub(crate) struct ElementTreePreOrderIterator<'a> {
    stack: Vec<&'a dyn Element>,
}

impl<'a> ElementTreePreOrderIterator<'a> {
    fn new(root: &'a dyn Element) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for ElementTreePreOrderIterator<'a> {
    type Item = &'a dyn Element;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            for child in node.children().iter().rev() {
                self.stack.push(*child);
            }
            Some(node)
        } else {
            None
        }
    }
}

impl dyn Element {
    pub fn pre_order_iter(&self) -> ElementTreePreOrderIterator {
        ElementTreePreOrderIterator::new(self)
    }
}

