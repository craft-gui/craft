use crate::platform::resource_manager::identifier::ResourceIdentifier;
use chrono::{DateTime, Utc};

pub struct ResourceData {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub(crate) data: Option<Vec<u8>>,
    #[allow(dead_code)]
    expiration_time: Option<DateTime<Utc>>,
}

impl ResourceData {
    pub(crate) fn new(
        resource_identifier: ResourceIdentifier,
        data: Vec<u8>,
        expiration_time: Option<DateTime<Utc>>,
    ) -> Self {
        ResourceData {
            resource_identifier,
            expiration_time,
            data: Some(data),
        }
    }
}
