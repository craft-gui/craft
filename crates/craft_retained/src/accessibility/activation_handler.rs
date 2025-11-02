use accesskit::{ActivationHandler, TreeUpdate};

pub(crate) struct CraftActivationHandler {
    tree_update: Option<TreeUpdate>,
}

impl ActivationHandler for CraftActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.tree_update.take().unwrap())
    }
}

impl CraftActivationHandler {
    pub fn new(tree_update: Option<TreeUpdate>) -> Self {
        CraftActivationHandler { tree_update }
    }
}
