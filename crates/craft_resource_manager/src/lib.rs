mod identifier;
pub mod image;
mod lock_free_map;
pub mod resource;

pub mod resource_event;
pub mod decoders;
pub mod resource_type;

use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use craft_runtime::{CraftRuntimeHandle, Sender};
use crate::decoders::{image_decoder, tinyvg_decoder};
pub use crate::identifier::ResourceId;
use crate::lock_free_map::LockFreeMap;
use crate::resource::Resource;
use crate::resource_event::ResourceEvent;
use crate::resource_type::ResourceType;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

#[cfg(not(target_arch = "wasm32"))]
pub trait ResourceEventHandler: From<ResourceEvent> + Send + 'static {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: From<ResourceEvent> + Send + 'static> ResourceEventHandler for T {}

#[cfg(target_arch = "wasm32")]
pub trait ResourceEventHandler: From<ResourceEvent> + 'static {}

#[cfg(target_arch = "wasm32")]
impl<T: From<ResourceEvent> + 'static> ResourceEventHandler for T {}

pub struct ResourceManager {
    resources: LockFreeMap<ResourceId, Resource>,
    pub(crate) runtime: CraftRuntimeHandle,
    decoders: HashMap<ResourceType, fn(Vec<u8>) -> Box<dyn Any + Send>>
}

impl ResourceManager {
    pub fn new(craft_runtime_handle: CraftRuntimeHandle) -> Self {
        Self {
            resources: LockFreeMap::new(),
            runtime: craft_runtime_handle,
            decoders: HashMap::from(
                [
                    (ResourceType::Image, image_decoder as fn(Vec<u8>) -> Box<dyn Any + Send + 'static>),
                    (ResourceType::TinyVg, tinyvg_decoder as fn(Vec<u8>) -> Box<dyn Any + Send + 'static>)
                ]
            ),
        }
    }

    pub fn async_download_resource_and_send_message_on_finish<Message: ResourceEventHandler>(
        &self,
        app_sender: Sender<Message>,
        resource_id: ResourceId,
        resource_type: &ResourceType,
    ) {
        let resource_id_copy = resource_id.clone();

        let resource_id = resource_id.clone();
        let resource_type = resource_type.clone();
        let decoder_fn =  *self.decoders.get(&resource_type).unwrap();
        let app_sender_copy = app_sender.clone();
        let f = async move {
            let bytes = resource_id.fetch_data_from_resource_id().await;

            let resource = Resource {
                resource_type: resource_type.clone(),
                data: decoder_fn(bytes.unwrap()),
                expiration_time: None,
            };

            app_sender_copy
                .send(ResourceEvent::Loaded(resource_id_copy, resource_type, resource).into())
                .await
                .expect("Failed to send added resource event");
        };

        self.runtime.spawn(f);
    }

    pub fn contains(&self, resource_id: &ResourceId) -> bool {
        self.resources.contains(resource_id)
    }

    pub fn get(&self, resource_id: &ResourceId) -> Option<Arc<Resource>> {
        self.resources.get(resource_id)
    }

    pub fn insert(&self, resource_id: ResourceId, resource: Arc<Resource>) {
        self.resources.insert(resource_id, resource);
    }
}
