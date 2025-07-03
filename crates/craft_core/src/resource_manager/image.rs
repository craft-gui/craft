use crate::resource_manager::resource_data::ResourceData;
use image::{EncodableLayout, RgbaImage};

#[derive(Debug)]
pub struct ImageResource {
    pub common_data: ResourceData,
    pub width: u32,
    pub height: u32,
    pub image: RgbaImage,
}

impl ImageResource {
    pub(crate) fn new(width: u32, height: u32, mut data: ResourceData) -> Self {
        if let Some(image_data) = data.data.take() {
            let image = image::load_from_memory(image_data.as_bytes()).unwrap();
            let image = image.to_rgba8();

            ImageResource {
                common_data: data,
                image,
                width,
                height,
            }
        } else {
            let empty_image = RgbaImage::new(0, 0);
            data.data = None;

            ImageResource {
                common_data: data,
                image: empty_image,
                width,
                height,
            }
        }
    }
}
