mod identifier;
pub(crate) mod image;
pub mod resource;
pub mod resource_data;
pub mod resource_type;
pub(crate) mod tinyvg_resource;
mod lock_free_map;

#[cfg(target_arch = "wasm32")]
pub(crate) mod wasm_queue;

use crate::events::internal::InternalMessage;
use crate::events::resource_event::ResourceEvent;
pub use crate::resource_manager::identifier::ResourceIdentifier;
use crate::resource_manager::image::ImageResource;
use crate::resource_manager::resource::Resource;
use crate::resource_manager::resource_data::ResourceData;
use crate::resource_manager::resource_type::ResourceType;
use craft_logging::info;

use ::image::ImageReader;
use tokio::sync::mpsc::Sender;

use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;
use std::sync::Arc;
use crate::craft_runtime::CraftRuntimeHandle;
use crate::resource_manager::lock_free_map::LockFreeMap;
use crate::resource_manager::tinyvg_resource::TinyVgResource;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

pub struct ResourceManager {
    pub(crate) resources: LockFreeMap<ResourceIdentifier, Resource>,
    pub(crate) app_sender: Sender<InternalMessage>,
    pub(crate) runtime: CraftRuntimeHandle,
}

impl ResourceManager {
    pub(crate) fn new(app_sender: Sender<InternalMessage>, craft_runtime_handle: CraftRuntimeHandle) -> Self {
        Self {
            resources: LockFreeMap::new(),
            app_sender,
            runtime: craft_runtime_handle,
        }
    }

    pub fn async_download_resource_and_send_message_on_finish(
        &self,
        resource_identifier: ResourceIdentifier,
        resource_type: ResourceType,
        resources_collected: &HashMap<ResourceIdentifier, bool>
    ) {
        if !resources_collected.contains_key(&resource_identifier) {
            let resource_identifier_copy = resource_identifier.clone();
            let resource_type_copy = resource_type;

            match &resource_type_copy {
                ResourceType::Image => {
                    let resource_identifier = resource_identifier.clone();
                    let app_sender_copy = self.app_sender.clone();
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
                                .send(
                                    InternalMessage::ResourceEvent(ResourceEvent::Loaded(
                                        resource_identifier_copy,
                                        ResourceType::Image,
                                        resource,
                                    ),
                                ))
                                .await
                                .expect("Failed to send added resource event");
                        }
                    };
                    self.runtime.spawn(f);
                }
                ResourceType::Font => {
                    let resource = resource_identifier.clone();
                    let app_sender_copy = self.app_sender.clone();
                    let f = async move {
                        let bytes = resource.clone().fetch_data_from_resource_identifier().await;

                        if let Some(font_bytes) = bytes {
                            let resource = Resource::Font(font_bytes);

                            app_sender_copy
                                .send(
                                    InternalMessage::ResourceEvent(ResourceEvent::Loaded(
                                        resource_identifier_copy,
                                        ResourceType::Font,
                                        resource,
                                    ))
                                )
                                .await
                                .expect("Failed to send added resource event");
                        }
                    };
                    self.runtime.spawn(f);
                }
                ResourceType::TinyVg => {
                    let resource = resource_identifier.clone();
                    let app_sender_copy = self.app_sender.clone();
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

                            app_sender_copy
                                .send(
                                    InternalMessage::ResourceEvent(ResourceEvent::Loaded(
                                        resource_identifier_copy,
                                        ResourceType::TinyVg,
                                        resource,
                                    )),
                                ).await
                                .expect("Failed to send added resource event");
                        }
                    };
                    self.runtime.spawn(f);
                }
            }
        }
    }
}
