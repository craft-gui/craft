use crate::components::component::{UpdateFn, UpdateResult};
use crate::components::ComponentId;
use crate::components::props::Props;

pub struct UpdateQueueEntry {
    pub source_component: ComponentId,
    pub update_function: UpdateFn,
    pub update_result: UpdateResult,
    pub props: Props,
}

impl UpdateQueueEntry {
    pub fn new(
        source_component: u64,
        update_function: UpdateFn,
        update_result: UpdateResult,
        props: Props,
    ) -> Self {
        UpdateQueueEntry {
            source_component,
            update_function,
            update_result,
            props,
        }
    }
}
