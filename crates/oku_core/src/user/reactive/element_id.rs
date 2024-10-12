use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
static ATOMIC_ELEMENT_ID: AtomicU64 = AtomicU64::new(0);

pub fn get_current_element_id_counter() -> u64 {
    ATOMIC_ELEMENT_ID.load(Ordering::SeqCst)
}

pub fn create_unique_element_id(user_state: &mut HashMap<u64, Option<Box<dyn Any + Send>>>) -> u64 {
    let id = get_current_element_id_counter();
    user_state.entry(id).or_insert(None);

    ATOMIC_ELEMENT_ID.fetch_add(1, Ordering::SeqCst);

    let id = get_current_element_id_counter();
    user_state.insert(id, None);

    get_current_element_id_counter()
}

pub fn reset_unique_element_id() {
    ATOMIC_ELEMENT_ID.store(0, Ordering::SeqCst);
}
