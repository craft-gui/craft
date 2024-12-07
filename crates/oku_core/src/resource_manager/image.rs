use crate::resource_manager::resource_data::ResourceData;
use image::RgbaImage;

pub struct ImageResource {
    pub common_data: ResourceData,
    pub width: u32,
    pub height: u32,
    pub image: RgbaImage,
}

impl ImageResource {
    pub(crate) fn new(width: u32, height: u32, mut data: ResourceData) -> Self {
        let image = image::load_from_memory(data.data.as_ref().unwrap()).unwrap();
        let image = image.to_rgba8();
        data.data = None;

        ImageResource {
            common_data: data,
            image,
            width,
            height,
        }
    }
}
