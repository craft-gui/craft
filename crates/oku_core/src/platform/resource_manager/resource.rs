use crate::platform::resource_manager::identifier::ResourceIdentifier;
use crate::platform::resource_manager::image::ImageResource;

pub enum Resource {
    Image(ImageResource),
}

impl Resource {
    pub fn resource_identifier(&self) -> ResourceIdentifier {
        match self {
            Resource::Image(data) => data.common_data.resource_identifier.clone(),
        }
    }

    pub fn data(&self) -> Option<&[u8]> {
        match self {
            Resource::Image(data) => data.common_data.data.as_deref(),
        }
    }
}
