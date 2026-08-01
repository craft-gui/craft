use crate::image::ImageResource;
use craft_logging::info;
use image::EncodableLayout;
use std::any::Any;
use tinyvg_rs::TinyVg;

pub fn image_decoder(bytes: Vec<u8>) -> Box<dyn Any + Send> {
    info!("Image downloaded");

    let image = image::load_from_memory(bytes.as_bytes()).unwrap();
    let image = image.to_rgba8();

    Box::new(ImageResource { image })
}

pub fn tinyvg_decoder(bytes: Vec<u8>) -> Box<dyn Any + Send> {
    let tinyvg = TinyVg::from_bytes(bytes.as_bytes()).unwrap();

    Box::new(tinyvg)
}
