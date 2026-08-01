use std::future::Future;
use std::pin::Pin;

pub trait CloneableRetGuiFn: 'static {
    fn call(&self) -> Pin<Box<dyn Future<Output = ()>>>;
    fn clone_box(&self) -> Box<dyn CloneableRetGuiFn>;
}

impl<F, Fut> CloneableRetGuiFn for F
where
    F: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output = ()> + 'static,
{
    fn call(&self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin((self)())
    }

    fn clone_box(&self) -> Box<dyn CloneableRetGuiFn> {
        Box::new(self.clone())
    }
}

pub struct RetGuiCallback(pub Box<dyn CloneableRetGuiFn>);

impl Clone for RetGuiCallback {
    fn clone(&self) -> Self {
        RetGuiCallback(self.0.clone_box())
    }
}

impl RetGuiCallback {
    pub fn call(&self) -> Pin<Box<dyn Future<Output = ()>>> {
        self.0.call()
    }
}
