use crate::resource_manager::image::ImageResource;
use crate::resource_manager::tinyvg_resource::TinyVgResource;
use std::sync::Arc;
use image::EncodableLayout;
use crate::resource_manager::resource_data::ResourceData;

#[derive(Debug)]
pub enum Resource {
    Image(Arc<ImageResource>),
    Font(ResourceData),
    TinyVg(TinyVgResource),
}

impl Resource {
    pub fn data(&self) -> Option<&[u8]> {
        match self {
            Resource::Image(data) => data.common_data.data.as_deref(),
            Resource::Font(common_data) => common_data.data.as_ref().map(|d| d.as_bytes()),
            Resource::TinyVg(data) => data.common_data.data.as_deref(),
        }
    }

    pub fn common_data(&self) -> &ResourceData {
        match self {
            Resource::Image(data) => &data.common_data,
            Resource::Font(data) => &data,
            Resource::TinyVg(data) => &data.common_data,
        }
    }
}
