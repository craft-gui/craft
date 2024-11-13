use crate::platform::resource_manager::image::ImageResource;
use crate::platform::resource_manager::resource::Resource;
use crate::platform::resource_manager::resource_data::ResourceData;
use crate::platform::resource_manager::ResourceIdentifier::Url;
use image::ImageReader;
use log::{info, warn};
use std::fmt;
use std::fmt::Display;
use std::io::Cursor;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ResourceIdentifier {
    Url(String),
    File(String),
}

impl Display for ResourceIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceIdentifier::Url(url) => write!(f, "URL: {}", url),
            ResourceIdentifier::File(file_path) => write!(f, "File: {}", file_path),
        }
    }
}

impl ResourceIdentifier {
    pub async fn fetch_resource_from_resource_identifier(&self) -> Option<Resource> {
        match self {
            Url(url) => {
                let res = reqwest::get(url).await;

                if let Ok(data) = res {
                    if !data.status().is_success() {
                        warn!("Failed to fetch resource from {}, Status: {}", url, data.status());
                        return None;
                    }

                    let bytes = data.bytes().await.ok();

                    // Do error checking here.
                    let bytes = bytes?;
                    let cursor = Cursor::new(&bytes);
                    let reader = ImageReader::new(cursor).with_guessed_format().expect("Failed to guess format");
                    let size = reader.into_dimensions().unwrap_or_default();
                    let generic_resource = ResourceData::new(self.clone(), bytes.to_vec(), None);
                    info!("Image downloaded");
                    return Some(Resource::Image(ImageResource::new(size.0, size.1, generic_resource)));
                } else {
                    return None;
                }
            }
            // tracing::warn!(name: "ResourceIdentifier", warning = "Resource Identifier {} not supported.");
            _ => {}
        }

        None
    }
}
