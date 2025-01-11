use crate::resource_manager::image::ImageResource;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::resource_data::ResourceData;
use crate::resource_manager::ResourceIdentifier::{File};

#[cfg(feature = "http_client")]
use crate::resource_manager::ResourceIdentifier::{Url};

use image::ImageReader;
use std::{fmt, fs};
use std::fmt::Display;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ResourceIdentifier {
    #[cfg(feature = "http_client")]
    Url(String),
    File(PathBuf),
}

impl Display for ResourceIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "http_client")]
            Url(url) => write!(f, "URL: {}", url),
            File(file_path) => write!(f, "File: {:?}", file_path.as_os_str().to_str()),
        }
    }
}

impl ResourceIdentifier {
    pub async fn fetch_data_from_resource_identifier(&self) -> Option<Vec<u8>> {
        match self {
            #[cfg(feature = "http_client")]
            Url(url) => {
                let res = reqwest::get(url).await;

                if let Ok(data) = res {
                    if !data.status().is_success() {
                        //tracing::warn!("Failed to fetch the resource from {}, Status: {}", url, data.status());
                        return None;
                    }

                    let bytes = data.bytes().await.ok();

                    bytes.map(|b| b.to_vec())
                } else {
                    None
                }
            }
            File(path) => {
                let file = fs::read(path);
                if let Ok(data) = file {
                    return Some(data);
                }
                //tracing::warn!("Failed to find the local file: {:?}", path.as_os_str().to_str());
                None
            }
        }
    }
}
