use image::RgbaImage;

#[derive(Debug, Clone)]
pub struct ImageResource {
    pub image: RgbaImage,
}

impl ImageResource {
    pub fn get_width(&self) -> u32 {
        self.image.width()
    }

    pub fn get_height(&self) -> u32 {
        self.image.height()
    }
}