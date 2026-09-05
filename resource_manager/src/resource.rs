use std::any::Any;

use jiff::Timestamp;

use crate::resource_type::ResourceType;

#[derive(Debug)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub data: Box<dyn Any + Send + Sync>,
    pub expiration_time: Option<Timestamp>,
}

impl Resource {
    pub fn resource_type(&self) -> &ResourceType {
        &self.resource_type
    }

    pub fn expiration_time(&self) -> Option<Timestamp> {
        self.expiration_time
    }
}
