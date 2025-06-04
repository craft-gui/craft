use crate::events::internal::InternalMessage;
use std::cell::RefCell;

const WASM_QUEUE_SIZE: usize = 100;

pub struct WasmQueue {
    queue: [Option<InternalMessage>; WASM_QUEUE_SIZE],
    next: usize,
}

impl WasmQueue {
    pub const fn new() -> Self {
        Self {
            queue: [const { None }; WASM_QUEUE_SIZE],
            next: 0,
        }
    }

    /// Push, overwriting the oldest entry if we’re full (ring buffer).
    pub fn push(&mut self, msg: InternalMessage) {
        self.queue[self.next] = Some(msg);
        self.next = (self.next + 1) % WASM_QUEUE_SIZE;
    }

    /// Drain all pending messages, calling `f` for each.
    pub fn drain<F: FnMut(InternalMessage)>(&mut self, mut f: F) {
        for slot in self.queue.iter_mut() {
            if let Some(msg) = slot.take() {
                f(msg);
            } else {
                break;
            }
        }
        self.next = 0;
    }

    pub fn len(&self) -> usize {
        self.next
    }
}

thread_local! {
    pub(crate) static WASM_QUEUE: RefCell<WasmQueue> = RefCell::new(WasmQueue::new());
}
