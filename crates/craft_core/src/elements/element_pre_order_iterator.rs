use crate::elements::element::Element;
pub struct ElementTreePreOrderIterator<'a> {
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
                self.stack.push(child.internal.as_ref());
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

#[cfg(test)]
mod tests {
    use std::any::Any;
    use crate::elements::element::ElementBoxed;
    use crate::elements::{Container, Text};
    use crate::events::update_queue_entry::UpdateQueueEntry;
    use crate::reactive::element_id::reset_unique_element_id;
    use crate::reactive::element_state_store::ElementStateStore;
    use crate::reactive::state_store::StateStore;
    use crate::reactive::tree::diff_trees;
    use crate::text::text_context::TextContext;
    use crate::window_context::WindowContext;
    use crate::GlobalState;
    use std::collections::VecDeque;

    #[test]
    fn pre_order_iter_ids_correct_order() {
        let mut text_context = TextContext::new();
        reset_unique_element_id();

        let initial_view = Container::new().id("1").component().push(Text::new("Foo").id("2").component()).push(
            Container::new()
                .id("3")
                .component()
                .push(Text::new("Bar").id("4").component())
                .push(Text::new("Baz").id("5").component()),
        );
        let root_element: ElementBoxed = Container::new().id("0").into();

        let mut user_state = StateStore::default();
        let mut element_state = ElementStateStore::default();
        let mut global_state = GlobalState::from(Box::new(()) as Box<dyn Any + Send>);
        let mut window_context = WindowContext::new();
        let mut update_queue: VecDeque<UpdateQueueEntry> = VecDeque::new();

        let initial_tree = diff_trees(
            initial_view,
            root_element.clone(),
            None,
            &mut user_state,
            &mut global_state,
            &mut element_state,
            false,
            &mut text_context,
            1.0,
            &mut window_context,
            &mut update_queue,
        );

        let mut iter = initial_tree.element_tree.internal.pre_order_iter();
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("0".into()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("1".into()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("2".into()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("3".into()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("4".into()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("5".into()));
        assert!(iter.next().is_none());
    }
}
