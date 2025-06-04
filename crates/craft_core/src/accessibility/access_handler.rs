use crate::craft_runtime::CraftRuntimeHandle;
use crate::events::internal::InternalMessage;
use accesskit::{ActionHandler, ActionRequest};
use tokio::sync::mpsc::Sender;

pub(crate) struct CraftAccessHandler {
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    pub(crate) runtime_handle: CraftRuntimeHandle,
    #[allow(dead_code)]
    pub(crate) app_sender: Sender<InternalMessage>,
}

impl ActionHandler for CraftAccessHandler {
    fn do_action(&mut self, _request: ActionRequest) {
    }
}