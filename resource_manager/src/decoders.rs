use std::any::Any;

use image::EncodableLayout;

use retgui_logging::info;

use tinyvg_rs::TinyVg;

use crate::ResourceError;
use crate::image::ImageResource;
use crate::resource_type::ResourceType;

pub fn image_decoder(bytes: Vec<u8>) -> Result<Box<dyn Any + Send + Sync>, ResourceError> {
    info!("Image downloaded");

    let image = image::load_from_memory(bytes.as_bytes())
        .map_err(|error| ResourceError::new(ResourceType::Image, error.to_string()))?;
    let image = image.to_rgba8();

    Ok(Box::new(ImageResource { image }))
}

pub fn tinyvg_decoder(bytes: Vec<u8>) -> Result<Box<dyn Any + Send + Sync>, ResourceError> {
    let tinyvg = TinyVg::from_bytes(bytes.as_bytes())
        .map_err(|error| ResourceError::new(ResourceType::TinyVg, format!("Invalid TinyVG: {error:?}")))?;

    Ok(Box::new(tinyvg))
}
