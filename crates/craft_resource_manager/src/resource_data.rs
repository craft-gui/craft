use chrono::{DateTime, Utc};

use crate::identifier::ResourceId;
use crate::resource_type::ResourceType;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ResourceData {
    pub(crate) resource_id: ResourceId,
    pub(crate) data: Option<Vec<u8>>,
    pub(crate) resource_type: ResourceType,
    pub(crate) expiration_time: Option<DateTime<Utc>>,
}

impl ResourceData {
    pub(crate) fn new(
        resource_id: ResourceId,
        data: Option<Vec<u8>>,
        expiration_time: Option<DateTime<Utc>>,
        resource_type: ResourceType,
    ) -> Self {
        ResourceData {
            resource_id,
            expiration_time,
            data,
            resource_type,
        }
    }

    pub fn expiration_time(&self) -> Option<DateTime<Utc>> {
        self.expiration_time
    }
}
