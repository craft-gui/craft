use crate::resource_manager::image::ImageResource;
use std::sync::Arc;

pub struct ImageAdapter {
    image: Arc<ImageResource>,
}

impl ImageAdapter {
    #[allow(dead_code)]
    pub fn new(image: Arc<ImageResource>) -> Self {
        Self { image }
    }
}

impl AsRef<[u8]> for ImageAdapter {
    fn as_ref(&self) -> &[u8] {
        self.image.image.as_ref()
    }
}
