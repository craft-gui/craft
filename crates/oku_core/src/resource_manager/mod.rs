mod identifier;
mod image;
pub mod resource;
pub mod resource_data;
pub mod resource_type;

use crate::app_message::AppMessage;
use crate::events::internal::InternalMessage;
use crate::events::resource_event::ResourceEvent;
pub use crate::resource_manager::identifier::ResourceIdentifier;
use crate::resource_manager::image::ImageResource;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::resource_data::ResourceData;
use crate::resource_manager::resource_type::ResourceType;
use ::image::ImageReader;
use cosmic_text::fontdb;
use futures::channel::mpsc::Sender;
use futures::SinkExt;
use log::info;
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

pub struct ResourceManager {
    pub(crate) resource_jobs: VecDeque<ResourceFuture>,
    pub(crate) resources: HashMap<ResourceIdentifier, Resource>,
    pub(crate) app_sender: Sender<AppMessage>,
}

impl ResourceManager {
    pub(crate) fn new(app_sender: Sender<AppMessage>) -> Self {
        Self {
            resource_jobs: VecDeque::new(),
            resources: HashMap::new(),
            app_sender,
        }
    }

    pub async fn add(
        &mut self,
        resource: ResourceIdentifier,
        resource_type: ResourceType,
        font_db: Option<&mut fontdb::Database>,
    ) {
        if !self.resources.contains_key(&resource) {
            let resource_copy = resource.clone();

            match resource_type {
                ResourceType::Image => {
                    let image = resource.fetch_data_from_resource_identifier().await;

                    if let Some(image_resource) = &image {
                        let bytes = image_resource;
                        let cursor = Cursor::new(&bytes);
                        let reader = ImageReader::new(cursor).with_guessed_format().expect("Failed to guess format");
                        let size = reader.into_dimensions().unwrap_or_default();
                        let generic_resource =
                            ResourceData::new(resource.clone(), bytes.to_vec(), None, ResourceType::Image);
                        info!("Image downloaded");

                        self.resources
                            .insert(resource, Resource::Image(ImageResource::new(size.0, size.1, generic_resource)));

                        self.app_sender
                            .send(AppMessage::new(
                                0,
                                InternalMessage::ResourceEvent(ResourceEvent::Added((resource_copy, resource_type))),
                            ))
                            .await
                            .expect("Failed to send added resource event");
                    }
                }
                ResourceType::Font => {
                    let bytes = resource.fetch_data_from_resource_identifier().await;

                    if let Some(font_db) = font_db {
                        if let Some(font_bytes) = bytes {
                            font_db.load_font_data(font_bytes.clone());
                            self.resources.insert(resource, Resource::Font(font_bytes));
                        }
                    }

                    self.app_sender
                        .send(AppMessage::new(0, InternalMessage::ResourceEvent(ResourceEvent::Added((resource_copy, resource_type)))))
                        .await
                        .expect("Failed to send added resource event");
                }
            }
        }
    }
}
