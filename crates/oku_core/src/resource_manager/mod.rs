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
use oku_logging::info;

use ::image::ImageReader;

use cosmic_text::fontdb;
use tokio::sync::mpsc::Sender;

use std::any::Any;
use std::collections::{HashMap};
use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;
use crate::OkuRuntime;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

pub struct ResourceManager {
    pub(crate) resources: HashMap<ResourceIdentifier, Resource>,
    pub(crate) app_sender: Sender<AppMessage>,
}

impl ResourceManager {
    pub(crate) fn new(app_sender: Sender<AppMessage>) -> Self {
        Self {
            resources: HashMap::new(),
            app_sender,
        }
    }

    pub fn add_temporary_resource(&mut self, resource_identifier: ResourceIdentifier,
                                  resource_type: ResourceType) {
        if !self.resources.contains_key(&resource_identifier) {
            match resource_type {
                ResourceType::Image => {
                    let generic_resource = ResourceData::new(resource_identifier.clone(), None, None, ResourceType::Image);
                    self.resources
                        .insert(resource_identifier.clone(), Resource::Image(ImageResource::new(0, 0, generic_resource)));
                }
                ResourceType::Font => {
                    self.resources.insert(resource_identifier.clone(), Resource::Font(vec![]));
                }
            }   
        }
    }
    
    pub fn async_download_resource_and_send_message_on_finish(
        &self,
        resource_identifier: ResourceIdentifier,
        resource_type: ResourceType,
        font_db: Option<&mut fontdb::Database>,
    ) {
        if !self.resources.contains_key(&resource_identifier) {
            let resource_identifier_copy = resource_identifier.clone();
            let resource_type_copy = resource_type;

            match &resource_type_copy {
                ResourceType::Image => {

                    let f = || {
                        let resource_identifier = resource_identifier.clone();
                        let app_sender_copy = self.app_sender.clone();
                        async move {
                            let image = resource_identifier.fetch_data_from_resource_identifier().await;

                            if let Some(image_resource) = &image {
                                let bytes = image_resource;
                                let cursor = Cursor::new(&bytes);
                                let reader = ImageReader::new(cursor).with_guessed_format().expect("Failed to guess format");
                                let size = reader.into_dimensions().unwrap_or_default();
                                let generic_resource = ResourceData::new(resource_identifier.clone(), Some(bytes.to_vec()), None, ResourceType::Image);
                                info!("Image downloaded");

                                let resource = Resource::Image(ImageResource::new(size.0, size.1, generic_resource));
                                app_sender_copy
                                    .send(AppMessage::new(
                                        0,
                                        InternalMessage::ResourceEvent(ResourceEvent::Loaded(resource_identifier_copy, ResourceType::Image, resource)),
                                    ))
                                    .await
                                    .expect("Failed to send added resource event");
                            }
                        }
                    };
                    OkuRuntime::native_spawn(f());
                }
                ResourceType::Font => {
                    
                    let f = || {
                    let resource = resource_identifier.clone();
                              let app_sender_copy = self.app_sender.clone();
                        async move {
                            let bytes = resource.clone().fetch_data_from_resource_identifier().await;

                            if let Some(font_bytes) = bytes {
                                let resource = Resource::Font(font_bytes);

                                app_sender_copy
                                    .send(AppMessage::new(0, InternalMessage::ResourceEvent(ResourceEvent::Loaded(resource_identifier_copy, ResourceType::Font, resource))))
                                    .await
                                    .expect("Failed to send added resource event");
                            }
                        }
                    };
                    OkuRuntime::native_spawn(f());
                }
            }
        }
    }
}
