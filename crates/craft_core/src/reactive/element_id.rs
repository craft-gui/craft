use std::sync::atomic::{AtomicU64, Ordering};

static ATOMIC_ELEMENT_ID: AtomicU64 = AtomicU64::new(0);

pub fn get_current_element_id_counter() -> u64 {
    ATOMIC_ELEMENT_ID.load(Ordering::SeqCst)
}

pub fn create_unique_element_id() -> u64 {
    ATOMIC_ELEMENT_ID.fetch_add(1, Ordering::SeqCst);
    get_current_element_id_counter()
}

pub fn reset_unique_element_id() {
    ATOMIC_ELEMENT_ID.store(0, Ordering::SeqCst);
}
