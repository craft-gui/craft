mod identifier;
pub mod image;
mod lock_free_map;
pub mod resource;
pub mod resource_data;
pub mod resource_type;
pub(crate) mod tinyvg_resource;

pub mod resource_event;

use crate::resource_event::ResourceEvent;
pub use crate::identifier::ResourceIdentifier;
use crate::image::ImageResource;
use crate::resource::Resource;
use crate::resource_data::ResourceData;
use crate::resource_type::ResourceType;
use craft_logging::info;

use craft_runtime::Sender;

use craft_runtime::CraftRuntimeHandle;
use crate::lock_free_map::LockFreeMap;
use crate::tinyvg_resource::TinyVgResource;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;
use std::sync::Arc;
use ::image::ImageReader;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

pub struct ResourceManager {
    pub resources: LockFreeMap<ResourceIdentifier, Resource>,
    pub(crate) runtime: CraftRuntimeHandle,
}

impl ResourceManager {
    pub fn new(craft_runtime_handle: CraftRuntimeHandle) -> Self {
        Self {
            resources: LockFreeMap::new(),
            runtime: craft_runtime_handle,
        }
    }
    // TODO: FIx the duplicate code in this function.
    #[cfg(target_arch = "wasm32")]
    pub fn async_download_resource_and_send_message_on_finish<Message: From<ResourceEvent> + 'static>(
        &self,
        app_sender: Sender<Message>,
        resource_identifier: ResourceIdentifier,
        resource_type: ResourceType,
        resources_collected: &HashMap<ResourceIdentifier, bool>,
    ) {
        if !resources_collected.contains_key(&resource_identifier) {
            let resource_identifier_copy = resource_identifier.clone();
            let resource_type_copy = resource_type;

            match &resource_type_copy {
                ResourceType::Image => {
                    let resource_identifier = resource_identifier.clone();
                    let app_sender_copy = app_sender.clone();
                    let f = async move {
                        let image = resource_identifier.fetch_data_from_resource_identifier().await;

                        if let Some(image_resource) = &image {
                            let bytes = image_resource;
                            let cursor = Cursor::new(&bytes);
                            let reader =
                                ImageReader::new(cursor).with_guessed_format().expect("Failed to guess format");
                            let size = reader.into_dimensions().unwrap_or_default();
                            let generic_resource = ResourceData::new(
                                resource_identifier.clone(),
                                Some(bytes.to_vec()),
                                None,
                                ResourceType::Image,
                            );
                            info!("Image downloaded");

                            let resource =
                                Resource::Image(Arc::new(ImageResource::new(size.0, size.1, generic_resource)));
                            app_sender_copy
                                .send(ResourceEvent::Loaded(
                                    resource_identifier_copy,
                                    ResourceType::Image,
                                    resource,
                                ).into())
                                .await
                                .expect("Failed to send added resource event");
                        }
                    };
                    self.runtime.spawn(f);
                }
                ResourceType::Font => {
                    let resource = resource_identifier.clone();
                    let app_sender_copy = app_sender.clone();
                    let f = async move {
                        let mut bytes = resource.clone().fetch_data_from_resource_identifier().await;

                        if let Some(font_bytes) = bytes.take() {
                            let generic_resource = ResourceData::new(
                                resource_identifier.clone(),
                                Some(font_bytes),
                                None,
                                ResourceType::Font,
                            );
                            info!("Font downloaded");
                            let resource = Resource::Font(generic_resource);

                            app_sender_copy
                                .send(ResourceEvent::Loaded(
                                    resource_identifier_copy,
                                    ResourceType::Font,
                                    resource,
                                ).into())
                                .await
                                .expect("Failed to send added resource event");
                        }
                    };
                    self.runtime.spawn(f);
                }
                ResourceType::TinyVg => {
                    let resource = resource_identifier.clone();
                    let app_sender_copy = app_sender.clone();
                    let f = async move {
                        let bytes = resource.clone().fetch_data_from_resource_identifier().await;

                        if let Some(bytes) = bytes {
                            let generic_resource = ResourceData::new(
                                resource_identifier.clone(),
                                Some(bytes.to_vec()),
                                None,
                                ResourceType::TinyVg,
                            );
                            let resource = Resource::TinyVg(TinyVgResource::new(generic_resource));
                            info!("TinyVG downloaded");
                            app_sender_copy
                                .send(ResourceEvent::Loaded(
                                    resource_identifier_copy,
                                    ResourceType::TinyVg,
                                    resource,
                                ).into())
                                .await
                                .expect("Failed to send added resource event");
                        }
                    };
                    self.runtime.spawn(f);
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn async_download_resource_and_send_message_on_finish<Message: From<ResourceEvent> + Send + 'static>(
        &self,
        app_sender: Sender<Message>,
        resource_identifier: ResourceIdentifier,
        resource_type: ResourceType,
        resources_collected: &HashMap<ResourceIdentifier, bool>,
    ) {
        if !resources_collected.contains_key(&resource_identifier) {
            let resource_identifier_copy = resource_identifier.clone();
            let resource_type_copy = resource_type;

            match &resource_type_copy {
                ResourceType::Image => {
                    let resource_identifier = resource_identifier.clone();
                    let app_sender_copy = app_sender.clone();
                    let f = async move {
                        let image = resource_identifier.fetch_data_from_resource_identifier().await;

                        if let Some(image_resource) = &image {
                            let bytes = image_resource;
                            let cursor = Cursor::new(&bytes);
                            let reader =
                                ImageReader::new(cursor).with_guessed_format().expect("Failed to guess format");
                            let size = reader.into_dimensions().unwrap_or_default();
                            let generic_resource = ResourceData::new(
                                resource_identifier.clone(),
                                Some(bytes.to_vec()),
                                None,
                                ResourceType::Image,
                            );
                            info!("Image downloaded");

                            let resource =
                                Resource::Image(Arc::new(ImageResource::new(size.0, size.1, generic_resource)));
                            app_sender_copy
                                .send(ResourceEvent::Loaded(
                                    resource_identifier_copy,
                                    ResourceType::Image,
                                    resource,
                                ).into())
                                .await
                                .expect("Failed to send added resource event");
                        }
                    };
                    self.runtime.spawn(f);
                }
                ResourceType::Font => {
                    let resource = resource_identifier.clone();
                    let app_sender_copy = app_sender.clone();
                    let f = async move {
                        let mut bytes = resource.clone().fetch_data_from_resource_identifier().await;

                        if let Some(font_bytes) = bytes.take() {
                            let generic_resource = ResourceData::new(
                                resource_identifier.clone(),
                                Some(font_bytes),
                                None,
                                ResourceType::Font,
                            );
                            info!("Font downloaded");
                            let resource = Resource::Font(generic_resource);

                            app_sender_copy
                                .send(ResourceEvent::Loaded(
                                    resource_identifier_copy,
                                    ResourceType::Font,
                                    resource,
                                ).into())
                                .await
                                .expect("Failed to send added resource event");
                        }
                    };
                    self.runtime.spawn(f);
                }
                ResourceType::TinyVg => {
                    let resource = resource_identifier.clone();
                    let app_sender_copy = app_sender.clone();
                    let f = async move {
                        let bytes = resource.clone().fetch_data_from_resource_identifier().await;

                        if let Some(bytes) = bytes {
                            let generic_resource = ResourceData::new(
                                resource_identifier.clone(),
                                Some(bytes.to_vec()),
                                None,
                                ResourceType::TinyVg,
                            );
                            let resource = Resource::TinyVg(TinyVgResource::new(generic_resource));
                            info!("TinyVG downloaded");
                            app_sender_copy
                                .send(ResourceEvent::Loaded(
                                    resource_identifier_copy,
                                    ResourceType::TinyVg,
                                    resource,
                                ).into())
                                .await
                                .expect("Failed to send added resource event");
                        }
                    };
                    self.runtime.spawn(f);
                }
            }
        }
    }
}
