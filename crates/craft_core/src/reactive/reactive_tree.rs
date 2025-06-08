use crate::components::{ComponentId, Event, FocusAction};
use crate::elements::Element;
use crate::events::update_queue_entry::UpdateQueueEntry;
use crate::reactive::element_state_store::ElementStateStore;
use crate::reactive::state_store::StateStore;
use crate::reactive::tree::ComponentTreeNode;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Default)]
pub(crate) struct ReactiveTree {
    pub(crate) element_tree: Option<Box<dyn Element>>,
    pub(crate) component_tree: Option<ComponentTreeNode>,
    pub(crate) element_ids: HashSet<ComponentId>,
    pub(crate) component_ids: HashSet<ComponentId>,
    /// Stores a pointer device id and their pointer captured element.
    pub(crate) pointer_captures: HashMap<i64, ComponentId>,
    pub(crate) update_queue: VecDeque<UpdateQueueEntry>,
    pub(crate) user_state: StateStore,
    pub(crate) element_state: ElementStateStore,
    pub(crate) focus: Option<ComponentId>,
}

impl ReactiveTree {
    pub(crate) fn update_focus(&mut self, focus: FocusAction) {
        match focus {
            FocusAction::None => {}
            FocusAction::Set(id) => {
                self.focus = Some(id);
            }
            FocusAction::Unset => {
                self.focus = None;
            }
        }
    }
}
