use std::any::Any;

use image::EncodableLayout;

use retgui_logging::info;

use tinyvg_rs::TinyVg;

use crate::image::ImageResource;

pub fn image_decoder(bytes: Vec<u8>) -> Box<dyn Any + Send + Sync> {
    info!("Image downloaded");

    let image = image::load_from_memory(bytes.as_bytes()).unwrap();
    let image = image.to_rgba8();

    Box::new(ImageResource { image })
}

pub fn tinyvg_decoder(bytes: Vec<u8>) -> Box<dyn Any + Send + Sync> {
    let tinyvg = TinyVg::from_bytes(bytes.as_bytes()).unwrap();

    Box::new(tinyvg)
}
