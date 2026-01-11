use std::future::Future;
use std::pin::Pin;

pub trait CloneableCraftFn: 'static {
    fn call(&self) -> Pin<Box<dyn Future<Output = ()>>>;
    fn clone_box(&self) -> Box<dyn CloneableCraftFn>;
}

impl<F, Fut> CloneableCraftFn for F
where
    F: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output = ()> + 'static,
{
    fn call(&self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin((self)())
    }

    fn clone_box(&self) -> Box<dyn CloneableCraftFn> {
        Box::new(self.clone())
    }
}

pub struct CraftCallback(pub Box<dyn CloneableCraftFn>);

impl Clone for CraftCallback {
    fn clone(&self) -> Self {
        CraftCallback(self.0.clone_box())
    }
}

impl CraftCallback {
    pub fn call(&self) -> Pin<Box<dyn Future<Output = ()>>> {
        self.0.call()
    }
}
