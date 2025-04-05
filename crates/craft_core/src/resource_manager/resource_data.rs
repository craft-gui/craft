use crate::resource_manager::identifier::ResourceIdentifier;
use crate::resource_manager::resource_type::ResourceType;
use chrono::{DateTime, Utc};

#[allow(dead_code)]
#[derive(Debug)]
pub struct ResourceData {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub(crate) data: Option<Vec<u8>>,
    pub(crate) resource_type: ResourceType,
    expiration_time: Option<DateTime<Utc>>,
}

impl ResourceData {
    pub(crate) fn new(
        resource_identifier: ResourceIdentifier,
        data: Option<Vec<u8>>,
        expiration_time: Option<DateTime<Utc>>,
        resource_type: ResourceType,
    ) -> Self {
        ResourceData {
            resource_identifier,
            expiration_time,
            data,
            resource_type,
        }
    }
}
