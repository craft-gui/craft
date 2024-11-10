use crate::components::component::{UpdateFn, UpdateResult};
use crate::components::props::Props;

pub struct UpdateQueueEntry {
    pub source_component: u64,
    pub source_element: Option<String>,
    pub update_function: UpdateFn,
    pub update_result: UpdateResult,
    pub props: Option<Props>
}

impl UpdateQueueEntry {
    pub fn new(
        source_component: u64,
        source_element: Option<String>,
        update_function: UpdateFn,
        update_result: UpdateResult,
        props: Option<Props>
    ) -> Self {
        UpdateQueueEntry {
            source_component,
            source_element,
            update_function,
            update_result,
            props,
        }
    }
}
