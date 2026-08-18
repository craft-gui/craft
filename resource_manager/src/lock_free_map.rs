use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

pub struct Node<K: Clone + Hash + PartialEq, V> {
    pub key: K,
    pub value: Arc<V>,
    pub next: *mut Node<K, V>,
}

pub struct Inner<K: Clone + Hash + PartialEq, V> {
    pub mask: usize,
    pub buckets: Box<[AtomicPtr<Node<K, V>>]>,
}

impl<K: Clone + Hash + PartialEq, V> Inner<K, V> {
    pub fn new(size: usize) -> Self {
        let size = size.next_power_of_two().max(1);
        let buckets = (0..size)
            .map(|_| AtomicPtr::new(ptr::null_mut()))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Inner {
            mask: size - 1,
            buckets,
        }
    }

    pub fn bucket(&self, key: &K) -> usize {
        let mut h = DefaultHasher::new();
        key.hash(&mut h);
        (h.finish() as usize) & self.mask
    }
}

/// A growable and lock-free map that can have many readers, but only one writer.
pub struct LockFreeMap<K: Clone + Hash + PartialEq, V> {
    pub root: AtomicPtr<Inner<K, V>>,
    pub count: AtomicUsize,
}

impl<K: Clone + Hash + PartialEq, V> Default for LockFreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone + Hash + PartialEq, V> LockFreeMap<K, V> {
    pub fn new() -> Self {
        let inner = Arc::new(Inner::<K, V>::new(1000));
        LockFreeMap {
            root: AtomicPtr::new(Arc::into_raw(inner) as *mut _),
            count: AtomicUsize::new(0),
        }
    }

    fn with_root<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Arc<Inner<K, V>>) -> R,
    {
        let ptr = self.root.load(Ordering::Acquire);
        unsafe {
            Arc::increment_strong_count(ptr);
            let arc = Arc::from_raw(ptr);
            let res = f(&arc);
            std::mem::forget(arc);
            res
        }
    }

    fn resize_if_needed(&self) {
        let count = self.count.load(Ordering::Relaxed);
        self.with_root(|inner| {
            let capacity = inner.mask + 1;
            // Resize when we reach 66% of the total capacity.
            let at_capacity_threshold = count * 3 > capacity * 2;

            if at_capacity_threshold {
                let new_inner = Arc::new(Inner::new(capacity * 2));
                for bucket in inner.buckets.iter() {
                    let mut cur = bucket.load(Ordering::Acquire);
                    while !cur.is_null() {
                        unsafe {
                            let node = &*cur;
                            let idx = new_inner.bucket(&node.key);
                            let head = new_inner.buckets[idx].load(Ordering::Acquire);
                            let new_node = Box::new(Node {
                                key: node.key.clone(),
                                value: node.value.clone(),
                                next: head,
                            });
                            new_inner.buckets[idx].store(Box::into_raw(new_node), Ordering::Release);
                            cur = node.next;
                        }
                    }
                }
                let new_ptr = Arc::into_raw(new_inner) as *mut _;
                let old = self.root.swap(new_ptr, Ordering::AcqRel);
                unsafe {
                    drop(Arc::from_raw(old));
                }
            }
        });
    }

    pub fn insert(&self, key: K, value: Arc<V>) {
        self.resize_if_needed();
        self.with_root(|inner| {
            let idx = inner.bucket(&key);
            let head = inner.buckets[idx].load(Ordering::Acquire);
            let node = Box::new(Node {
                key,
                value,
                next: head,
            });
            inner.buckets[idx].store(Box::into_raw(node), Ordering::Release);
        });
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        self.with_root(|inner| {
            let idx = inner.bucket(key);
            let mut cur = inner.buckets[idx].load(Ordering::Acquire);
            while !cur.is_null() {
                unsafe {
                    let node = &*cur;
                    if node.key == *key {
                        return Some(node.value.clone());
                    }
                    cur = node.next;
                }
            }
            None
        })
    }
}

impl<K: Clone + Hash + PartialEq, V> Drop for LockFreeMap<K, V> {
    fn drop(&mut self) {
        let ptr = self.root.load(Ordering::Relaxed);
        if !ptr.is_null() {
            unsafe {
                drop(Arc::from_raw(ptr));
            }
        }
    }
}

impl<K: Clone + Hash + PartialEq, V> LockFreeMap<K, V> {
    pub fn contains(&self, key: &K) -> bool {
        // Todo speedup with cache
        self.with_root(|inner| {
            let idx = inner.bucket(key);
            let mut cur = inner.buckets[idx].load(Ordering::Acquire);
            while !cur.is_null() {
                unsafe {
                    let node = &*cur;
                    if node.key == *key {
                        return true;
                    }
                    cur = node.next;
                }
            }
            false
        })
    }
}
