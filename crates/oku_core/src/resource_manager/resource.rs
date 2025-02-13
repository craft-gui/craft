use std::sync::Arc;
use crate::resource_manager::image::ImageResource;

#[derive(Debug)]
pub enum Resource {
    Image(Arc<ImageResource>),
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
