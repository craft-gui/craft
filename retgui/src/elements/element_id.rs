use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ELEMENT_ID: AtomicU64 = AtomicU64::new(0);

pub fn create_unique_element_id() -> u64 {
    NEXT_ELEMENT_ID.fetch_add(1, Ordering::Relaxed)
}
