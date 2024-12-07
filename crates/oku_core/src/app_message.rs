use crate::events::internal::InternalMessage;

pub(crate) struct AppMessage {
    pub(crate) id: u64,
    pub(crate) blocking: bool,
    pub(crate) data: InternalMessage,
}

impl AppMessage {
    pub fn new(id: u64, content: InternalMessage) -> Self {
        AppMessage {
            id,
            blocking: false,
            data: content,
        }
    }

    #[allow(dead_code)]
    pub fn new_blocking(id: u64, content: InternalMessage) -> Self {
        AppMessage {
            id,
            blocking: true,
            data: content,
        }
    }
}
