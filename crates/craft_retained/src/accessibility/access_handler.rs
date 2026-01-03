use accesskit::{ActionHandler, ActionRequest};

pub(crate) struct CraftAccessHandler {}

impl ActionHandler for CraftAccessHandler {
    fn do_action(&mut self, _request: ActionRequest) {}
}
