use std::any::Any;

pub trait CloneableAny: Any {
    fn clone_box(&self) -> Box<dyn CloneableAny>;
}

impl<T> CloneableAny for T
where
    T: Any + Clone + 'static, // <-- ensure it's 'static
{
    fn clone_box(&self) -> Box<dyn CloneableAny> {
        Box::new(self.clone())
    }
}
