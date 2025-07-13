use std::cell::RefCell;
use crate::components::{ComponentId, FocusAction};
use crate::elements::Element;
use crate::events::update_queue_entry::UpdateQueueEntry;
use crate::reactive::element_state_store::ElementStateStore;
use crate::reactive::state_store::StateStore;
use crate::reactive::tree::ComponentTreeNode;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use crate::animations::animation::AnimationFlags;
use crate::layout::layout_context::LayoutContext;
use crate::reactive::fiber_tree;
use crate::reactive::fiber_tree::FiberNode;

#[derive(Default)]
pub struct ReactiveTree {
    pub element_tree: Option<Box<dyn Element>>,
    pub(crate) component_tree: Option<ComponentTreeNode>,
    pub(crate) element_ids: HashSet<ComponentId>,
    pub(crate) component_ids: HashSet<ComponentId>,
    /// Stores a pointer device id and their pointer captured element.
    pub(crate) pointer_captures: HashMap<i64, ComponentId>,
    pub(crate) update_queue: VecDeque<UpdateQueueEntry>,
    pub(crate) user_state: StateStore,
    pub(crate) element_state: ElementStateStore,
    pub(crate) focus: Option<ComponentId>,
    pub(crate) previous_animation_flags: AnimationFlags,
    pub(crate) taffy_tree: Option<taffy::TaffyTree<LayoutContext>>,
}

impl ReactiveTree {
    pub(crate) fn as_fiber_tree(&self) -> Rc<RefCell<FiberNode>> {
        fiber_tree::new(self.component_tree.as_ref().unwrap(), self.element_tree.as_ref().unwrap().as_ref())
    }
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
