use std::any::Any;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Resource {
    pub resource_type: String,
    pub data: Box<dyn Any + Send>,
    pub expiration_time: Option<DateTime<Utc>>,
}

impl Resource {
    pub fn expiration_time(&self) -> Option<DateTime<Utc>> {
        self.expiration_time
    }
}
