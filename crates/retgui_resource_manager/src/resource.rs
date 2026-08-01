use crate::resource_type::ResourceType;
use chrono::{DateTime, Utc};
use std::any::Any;

#[derive(Debug)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub data: Box<dyn Any + Send>,
    pub expiration_time: Option<DateTime<Utc>>,
}

impl Resource {
    pub fn resource_type(&self) -> &ResourceType {
        &self.resource_type
    }

    pub fn expiration_time(&self) -> Option<DateTime<Utc>> {
        self.expiration_time
    }
}
