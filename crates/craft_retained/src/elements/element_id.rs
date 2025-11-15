use std::cell::Cell;

thread_local! {
    static THREAD_LOCAL_ELEMENT_ID: Cell<u64> = const { Cell::new(0) };
}

pub fn create_unique_element_id() -> u64 {
    THREAD_LOCAL_ELEMENT_ID.with(|counter| {
        let id = counter.get();
        counter.set(id + 1);
        id
    })
}