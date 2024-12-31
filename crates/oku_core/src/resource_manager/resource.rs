use crate::resource_manager::identifier::ResourceIdentifier;
use crate::resource_manager::image::ImageResource;

pub enum Resource {
    Image(ImageResource),
    Font(Vec<u8>),
}

impl Resource {

    pub fn data(&self) -> Option<&[u8]> {
        match self {
            Resource::Image(data) => data.common_data.data.as_deref(),
            Resource::Font(data) => Some(data),
        }
    }
}
