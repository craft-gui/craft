mod identifier;
mod image;
pub mod resource;
pub mod resource_data;
pub mod resource_type;

use crate::app_message::AppMessage;
use crate::events::internal::InternalMessage;
use crate::events::resource_event::ResourceEvent;
pub use crate::resource_manager::identifier::ResourceIdentifier;
use crate::resource_manager::resource::Resource;
use futures::channel::mpsc::Sender;
use futures::SinkExt;
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;
use ::image::ImageReader;
use log::info;
use crate::resource_manager::image::ImageResource;
use crate::resource_manager::resource_data::ResourceData;
use crate::resource_manager::resource_type::ResourceType;

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

    pub async fn add(&mut self, resource: ResourceIdentifier, resource_type: ResourceType) {
        if !self.resources.contains_key(&resource) {
            
            match resource_type {
                ResourceType::Image => {
                    let image = resource.fetch_data_from_resource_identifier().await;

                    if let Some(image_resource) = &image {
                        let resource_copy = resource.clone();
                        
                        let bytes = image_resource;
                        let cursor = Cursor::new(&bytes);
                        let reader = ImageReader::new(cursor).with_guessed_format().expect("Failed to guess format");
                        let size = reader.into_dimensions().unwrap_or_default();
                        let generic_resource = ResourceData::new(resource.clone(), bytes.to_vec(), None, ResourceType::Image);
                        info!("Image downloaded");

                        self.resources.insert(resource, Resource::Image(ImageResource::new(size.0, size.1, generic_resource)));

                        self.app_sender
                            .send(AppMessage::new(0, InternalMessage::ResourceEvent(ResourceEvent::Added(resource_copy))))
                            .await
                            .expect("Failed to send added resource event");
                    }       
                }
                ResourceType::Font => {
                    panic!("Implement font loading.");
                }
            }
        }
    }
}
