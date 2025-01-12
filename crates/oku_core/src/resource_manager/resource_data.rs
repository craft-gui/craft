use crate::resource_manager::identifier::ResourceIdentifier;
use chrono::{DateTime, Utc};
use crate::resource_manager::resource_type::ResourceType;

#[allow(dead_code)]
pub struct ResourceData {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub(crate) data: Option<Vec<u8>>,
    pub(crate) resource_type: ResourceType,
    expiration_time: Option<DateTime<Utc>>,
}

impl ResourceData {
    pub(crate) fn new(
        resource_identifier: ResourceIdentifier,
        data: Vec<u8>,
        expiration_time: Option<DateTime<Utc>>,
        resource_type: ResourceType,
    ) -> Self {
        ResourceData {
            resource_identifier,
            expiration_time,
            data: Some(data),
            resource_type
        }
    }
}
