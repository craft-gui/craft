use crate::resource_manager::image::ImageResource;
use std::sync::Arc;

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
