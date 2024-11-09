use std::any::Any;
use std::sync::Arc;

/// A read-only block of data that can be shared between threads.
///
/// The `Props` struct allows for storing any type of data that implements
/// `Any`, `Send`, and `Sync`. The data is stored in an `Arc`, making it
/// safe for shared read-only access across multiple threads.
#[derive(Clone)]
pub struct Props {
    pub data: Arc<dyn Any + Send + Sync>,
}

impl Props {
    pub fn get_data<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }

    pub fn new<T>(data: T) -> Self
    where
        T: Any + Send + Sync,
    {
        Self {
            data: Arc::new(data),
        }
    }
}
